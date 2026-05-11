use std::sync::Arc;
use std::{borrow::Cow, string::FromUtf8Error};

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedFolder, RecursiveDependencyLoadState, io::Reader},
    prelude::*,
    ui::UiSystems,
};
use chrono::{Datelike, NaiveDate, Timelike, Utc};
use fluent::types::FluentNumber;
use fluent::{FluentArgs, FluentResource, FluentValue, concurrent::FluentBundle};
use fluent_datetime::{BundleExt, FluentDateTime, length};
use icu::{
    calendar::{Date, Iso},
    time::{DateTime, Hour, Minute, Nanosecond, Second, Time},
};
use line_numbers::LinePositions;
use thiserror::Error;
use unic_langid::langid;

use crate::funds::FundsAmount;
use crate::state::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Load), setup)
        .add_systems(Update, update.run_if(in_state(GameState::Load)))
        .add_systems(OnExit(GameState::Load), cleanup)
        .add_systems(
            FixedUpdate,
            (
                reload.run_if(not(in_state(GameState::Load))),
                recalc_all_texts
                    .run_if(resource_changed::<FluentBundleWrapper>)
                    .after(reload),
            ),
        )
        .add_systems(
            PostUpdate,
            recalc_changed_texts
                .run_if(not(in_state(GameState::Load)))
                .before(UiSystems::Prepare),
        )
        .init_asset::<FluentResourceAsset>()
        .register_asset_loader(FluentResourceAssetLoader);
}

/// This type exists because `FluentValue` is not [`Sync`] and therefore
/// can't be used in a [`Component`].
/// There is the option of using `SyncCell` but I couldn't get that to work.
#[derive(Debug, Clone)]
pub enum TextArgValue {
    String(String),
    /// This uses `f64` because that's what `FluentValue` uses.
    /// It's unfortunate, because we use `i64` internally.
    Number(f64),
    Datetime(DateTime<Iso>),
}

impl TextArgValue {
    fn fluent(&self) -> FluentValue<'_> {
        match self {
            TextArgValue::String(s) => s.into(),
            TextArgValue::Number(n) => n.into(),
            TextArgValue::Datetime(d) => {
                let mut d: FluentDateTime = (*d).into();
                d.options.set_date_style(Some(length::Date::Long));
                d.into()
            }
        }
    }
}

impl From<&str> for TextArgValue {
    fn from(value: &str) -> Self {
        TextArgValue::String(value.into())
    }
}

impl From<String> for TextArgValue {
    fn from(value: String) -> Self {
        TextArgValue::String(value)
    }
}

impl From<f64> for TextArgValue {
    fn from(value: f64) -> Self {
        TextArgValue::Number(value)
    }
}

impl From<FundsAmount> for TextArgValue {
    fn from(value: FundsAmount) -> Self {
        TextArgValue::Number(value as f64)
    }
}

#[expect(clippy::fallible_impl_from, reason = "valid dates won't fail")]
impl From<NaiveDate> for TextArgValue {
    fn from(value: NaiveDate) -> Self {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "not a problem for valid dates"
        )]
        if let Ok(date) = Date::try_new_iso(value.year(), value.month() as u8, value.day() as u8) {
            TextArgValue::Datetime(DateTime {
                date,
                time: Time::start_of_day(),
            })
        } else {
            warn!("Invalid date: {value}");
            TextArgValue::Datetime(DateTime {
                date: Date::try_new_iso(2000, 1, 1).unwrap(),
                time: Time::start_of_day(),
            })
        }
    }
}

#[expect(clippy::fallible_impl_from, reason = "valid dates won't fail")]
impl From<chrono::DateTime<Utc>> for TextArgValue {
    fn from(value: chrono::DateTime<Utc>) -> Self {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "not a problem for valid dates"
        )]
        if let Ok(date) = Date::try_new_iso(value.year(), value.month() as u8, value.day() as u8) {
            TextArgValue::Datetime(DateTime {
                date,
                time: Time {
                    hour: Hour::try_from(value.hour() as usize).unwrap(),
                    minute: Minute::try_from(value.minute() as usize).unwrap(),
                    second: Second::try_from(value.second() as usize).unwrap(),
                    subsecond: Nanosecond::try_from(value.nanosecond() as usize).unwrap(),
                },
            })
        } else {
            warn!("Invalid date: {value}");
            TextArgValue::Datetime(DateTime {
                date: Date::try_new_iso(2000, 1, 1).unwrap(),
                time: Time::start_of_day(),
            })
        }
    }
}

impl From<DateTime<Iso>> for TextArgValue {
    fn from(value: DateTime<Iso>) -> Self {
        TextArgValue::Datetime(value)
    }
}

/// This component represents localized UI text.
/// It is the source of truth for the accompanying `Text` component.
#[derive(Component, Debug, Clone)]
#[require(Text)]
pub struct TextKey(pub String, pub Vec<(&'static str, TextArgValue)>);

impl TextKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into(), Vec::new())
    }

    pub fn add_arg(mut self, arg: &'static str, value: impl Into<TextArgValue>) -> Self {
        self.1.push((arg, value.into()));
        self
    }

    pub fn replace_arg(&mut self, arg: &'static str, value: impl Into<TextArgValue>) -> &mut Self {
        for (a, v) in &mut self.1 {
            if *a == arg {
                *v = value.into();
                break;
            }
        }
        self
    }
}

impl From<&str> for TextKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for TextKey {
    fn from(value: String) -> Self {
        Self(value, Vec::new())
    }
}

#[derive(Debug, Error)]
enum FluentResourceLoaderError {
    #[error("read error: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("invalid utf-8: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),
}

#[derive(TypePath)]
struct FluentResourceAssetLoader;

impl AssetLoader for FluentResourceAssetLoader {
    type Asset = FluentResourceAsset;
    type Settings = ();
    type Error = FluentResourceLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(FluentResourceAsset(Arc::new(
            match FluentResource::try_new(String::from_utf8(bytes)?) {
                Ok(resource) => resource,
                Err((resource, errs)) => {
                    let line_positions = LinePositions::from(resource.source());
                    for err in errs {
                        let (line_num, column) = line_positions.from_offset(err.pos.start);
                        error!(
                            "{}:{}:{column}: {}",
                            load_context.path(),
                            line_num.display(),
                            err.kind
                        );
                    }
                    resource
                }
            },
        )))
    }

    fn extensions(&self) -> &[&str] {
        &["ftl"]
    }
}

#[derive(Asset, TypePath)]
struct FluentResourceAsset(Arc<FluentResource>);

#[derive(Resource)]
struct FluentBundleWrapper(FluentBundle<Arc<FluentResource>>, bool);

impl FluentBundleWrapper {
    pub fn get(&self, key: &str, args: &[(&str, TextArgValue)]) -> String {
        let args = if args.is_empty() {
            None
        } else {
            Some(&args.iter().map(|(k, v)| (*k, v.fluent())).collect())
        };

        let pattern = if let Some((key, attribute)) = key.split_once('.') {
            let Some(msg) = self.0.get_message(key) else {
                error!("no message with key {key} exists");
                return String::new();
            };
            let Some(attr) = msg.get_attribute(attribute) else {
                error!("message {key} has no attribute {attribute}");
                return String::new();
            };
            attr.value()
        } else {
            let Some(msg) = self.0.get_message(key) else {
                error!("no message with key {key} exists");
                return String::new();
            };
            let Some(pattern) = msg.value() else {
                error!("message {key} has no value");
                return String::new();
            };
            pattern
        };

        let mut errors = vec![];
        let value = self.0.format_pattern(pattern, args, &mut errors);

        for e in errors {
            error!("message {key} formatting error: {e}");
        }

        value.into_owned()
    }
}

#[derive(Resource)]
struct FluentFolder(Handle<LoadedFolder>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FluentBundleWrapper(
        FluentBundle::new_concurrent(vec![langid!("en-US")]),
        false,
    ));
    commands.insert_resource(FluentFolder(asset_server.load_folder("text/en-US")));
}

// TODO: localize decimal sign
fn format_funds(mut f: f64) -> String {
    let sign = if f < 0.0 {
        f = -f;
        "-"
    } else {
        ""
    };
    if f < 100_000.0 {
        format!("{sign}€{f:.0}")
    } else {
        let magnifiers = &["", "k", "M", "B", "T", "Q"];
        let mut i = 0;
        while f >= 1000.0 && i + 1 < magnifiers.len() {
            f /= 1000.0;
            i += 1;
        }
        // Keep 3 significant digits, unless f is way over the Q range
        #[expect(clippy::bool_to_int_with_if)]
        let precision = if f < 10.0 {
            2
        } else if f < 100.0 {
            1
        } else {
            0
        };
        format!("{sign}€{1:.0$}{2}", precision, f, magnifiers[i])
    }
}

fn fluent_funds<'a>(positional: &[FluentValue<'a>], _named: &FluentArgs) -> FluentValue<'a> {
    match positional.first() {
        Some(FluentValue::Number(FluentNumber { value: f, .. })) => {
            FluentValue::String(Cow::Owned(format_funds(*f)))
        }
        Some(FluentValue::String(s)) => {
            if let Ok(f) = s.parse::<f64>() {
                FluentValue::String(Cow::Owned(format_funds(f)))
            } else {
                FluentValue::Error
            }
        }
        _ => FluentValue::Error,
    }
}

fn new_bundle<'a, I: Iterator<Item = &'a Arc<FluentResource>>>(
    bundle_resource: &FluentBundleWrapper,
    resource_iter: I,
) -> Option<FluentBundle<Arc<FluentResource>>> {
    let mut new_bundle = FluentBundle::new_concurrent(bundle_resource.0.locales.clone());
    if let Err(e) = new_bundle.add_builtins() {
        error!("could not add NUMBER to fluent bundle: {e}");
        return None;
    }
    if let Err(e) = new_bundle.add_datetime_support() {
        error!("could not add DATETIME to fluent bundle: {e}");
        return None;
    }
    if let Err(e) = new_bundle.add_function("FUNDS", fluent_funds) {
        error!("could not add FUNDS to fluent bundle: {e}");
        return None;
    }

    for resource in resource_iter {
        if let Err(err) = new_bundle.add_resource(Arc::clone(resource)) {
            for e in err {
                warn!("failed to add to fluent bundle: {e}");
            }
        }
    }

    Some(new_bundle)
}

fn update(
    asset_server: Res<AssetServer>,
    folder: Res<FluentFolder>,
    fluent_resource_assets: Res<Assets<FluentResourceAsset>>,
    mut bundle: ResMut<FluentBundleWrapper>,
) {
    if !bundle.1
        && matches!(
            asset_server.recursive_dependency_load_state(folder.0.id()),
            RecursiveDependencyLoadState::Loaded
        )
    {
        info!("fluent folder loaded");

        let Some(new_bundle) = new_bundle(
            &bundle,
            fluent_resource_assets.iter().map(|(_, res)| &res.0),
        ) else {
            return;
        };

        bundle.0 = new_bundle;
        bundle.1 = true;
    }
}

fn cleanup(mut messages: ResMut<Messages<AssetEvent<FluentResourceAsset>>>) {
    messages.clear();
}

fn reload(
    mut reader: MessageReader<AssetEvent<FluentResourceAsset>>,
    fluent_resource_assets: Res<Assets<FluentResourceAsset>>,
    mut bundle: ResMut<FluentBundleWrapper>,
) {
    if bundle.1 && !reader.is_empty() {
        info!("fluent bundle reloaded");
        let Some(new_bundle) = new_bundle(
            &bundle,
            fluent_resource_assets.iter().map(|(_, res)| &res.0),
        ) else {
            return;
        };
        bundle.0 = new_bundle;
        reader.clear();
    }
}

fn recalc_all_texts(bundle: Res<FluentBundleWrapper>, mut q: Query<(&mut Text, &TextKey)>) {
    for (mut text, TextKey(key, args)) in &mut q {
        text.set_if_neq(Text(bundle.get(key, args)));
    }
}

// TODO: change to an On<Insert, TextKey>?
fn recalc_changed_texts(
    bundle: Res<FluentBundleWrapper>,
    mut q: Query<(&mut Text, &TextKey), Changed<TextKey>>,
) {
    for (mut text, TextKey(key, args)) in &mut q {
        text.set_if_neq(Text(bundle.get(key, args)));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn funds_low() {
        assert_eq!(format_funds(50.0), "€50");
        assert_eq!(format_funds(5000.0), "€5000");
        assert_eq!(format_funds(12_345.0), "€12345");
    }

    #[test]
    fn funds_high() {
        assert_eq!(format_funds(123_456.0), "€123k");
        assert_eq!(format_funds(12_345_678.0), "€12.3M");
        assert_eq!(format_funds(1_234_567_891.0), "€1.23B");
        assert_eq!(format_funds(1_234_567_891_000.0), "€1.23T");
    }

    #[test]
    fn funds_very_high() {
        assert_eq!(format_funds(1_234_567_891_000_000.0), "€1.23Q");
        assert_eq!(format_funds(1_234_567_891_000_000_000.0), "€1235Q");
    }

    #[test]
    fn funds_negative() {
        assert_eq!(format_funds(-50.0), "-€50");
        assert_eq!(format_funds(-5000.0), "-€5000");
        assert_eq!(format_funds(-12_345.0), "-€12345");
        assert_eq!(format_funds(-1_234_567_891.0), "-€1.23B");
    }
}
