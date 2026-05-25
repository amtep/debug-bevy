use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use either::Either;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use rand::{RngExt, rngs::StdRng, seq::IndexedRandom};
use rand_distr::Poisson;
use serde::Deserialize;
use strum::Display;

use crate::{
    effects::{Effect, apply_effect},
    modifiers::{
        IntelligenceSuspicionModifier, MediaSuspicionModifier, Modifier, PoliceSuspicionModifier,
        ScientificSuspicionModifier, Source,
    },
    new_game::NewGame,
    regions::Region,
    rng::RandomSource,
    state::{GameState, MainSetupSet},
    time::{EndDate, GameDate},
};

const LOWER_LIMIT: u32 = 334;
const MIDDLE_LIMIT: u32 = 667;
const UPPER_LIMIT: u32 = 1000;

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<SuspicionEventsAsset>::new(&[
        "suspicion-events.toml",
    ]))
    .add_systems(OnEnter(GameState::Load), setup_load)
    .add_systems(
        OnEnter(GameState::Main),
        (setup_main, new_game.run_if(resource_exists::<NewGame>)).in_set(MainSetupSet::Default),
    )
    .add_systems(
        FixedUpdate,
        update_suspicion
            .run_if(resource_exists_and_changed::<GameDate>.and(in_state(GameState::Main))),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Deserialize, Display)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum SuspicionType {
    Intelligence,
    Scientific,
    Police,
    Media,
}

// global
#[derive(Resource, Default, Reflect, Deref)]
#[reflect(Resource)]
pub struct IntelligenceSuspicion(u32);

#[derive(Resource, Default, Reflect, Deref)]
#[reflect(Resource)]
pub struct ScientificSuspicion(u32);

// regional
#[derive(Component, Default, Reflect, Deref)]
#[reflect(Component)]
pub struct PoliceSuspicion(u32);

#[derive(Component, Default, Reflect, Deref)]
#[reflect(Component)]
pub struct MediaSuspicion(u32);

// custom private trait to prevent DerefMut from another module accessing the inner part
// without add_suspicion.
trait Suspicion {
    fn get(&self) -> u32;
    fn get_mut(&mut self) -> &mut u32;
}

macro_rules! suspicion {
    ($s: ty) => {
        impl Suspicion for $s {
            fn get(&self) -> u32 {
                self.0
            }

            fn get_mut(&mut self) -> &mut u32 {
                &mut self.0
            }
        }
    };
}

suspicion!(ResMut<'_, IntelligenceSuspicion>);
suspicion!(ResMut<'_, ScientificSuspicion>);
suspicion!(Mut<'_, PoliceSuspicion>);
suspicion!(Mut<'_, MediaSuspicion>);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct IntelligenceSuspicionChange(pub f32);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ScientificSuspicionChange(pub f32);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct PoliceSuspicionChange(pub f32);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct MediaSuspicionChange(pub f32);

fn setup_main(mut commands: Commands) {
    commands.insert_resource(IntelligenceSuspicion::default());
    commands.insert_resource(ScientificSuspicion::default());
}

fn new_game(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::Main),
        IntelligenceSuspicionChange(0.5),
        ScientificSuspicionChange(0.5),
        Save,
    ));
}

#[expect(clippy::cast_possible_truncation, reason = "it's random values anyway")]
#[expect(clippy::cast_sign_loss, reason = "it's random values anyway")]
fn update_suspicion_inner<T: DetectChangesMut + Suspicion>(
    mut commands: Commands,
    entity: Option<Entity>,
    mut value: T,
    suspicion_type: SuspicionType,
    amount: Either<i32, (f64, &mut StdRng)>,
) {
    let before = value.get();
    let after = match amount {
        Either::Left(amount) => before.saturating_add_signed(amount),
        Either::Right((amount, random)) => {
            if amount <= 0.0 {
                return;
            }
            before + random.sample(Poisson::new(amount).unwrap()) as u32
        }
    };

    if after <= before {
        return;
    }

    if before < LOWER_LIMIT && after >= LOWER_LIMIT
        || before < MIDDLE_LIMIT && after >= MIDDLE_LIMIT
    {
        *value.get_mut() = after;
        commands.run_system_cached_with(spawn_suspicion_event, (suspicion_type, false, entity));
    } else if before < UPPER_LIMIT && after >= UPPER_LIMIT {
        *value.get_mut() = after - UPPER_LIMIT;
        commands.run_system_cached_with(spawn_suspicion_event, (suspicion_type, true, entity));
    } else {
        *value.get_mut() = after;
    }
}

fn update_suspicion(
    mut commands: Commands,
    intel_suspicion: ResMut<IntelligenceSuspicion>,
    scien_suspicion: ResMut<ScientificSuspicion>,
    mut regions: Query<(Entity, &mut PoliceSuspicion, &mut MediaSuspicion), With<Region>>,
    intel_suspicion_changes: Query<(Entity, &IntelligenceSuspicionChange)>,
    scien_suspicion_changes: Query<(Entity, &ScientificSuspicionChange)>,
    m_i: Modifier<IntelligenceSuspicionModifier>,
    m_s: Modifier<ScientificSuspicionModifier>,
    m_p: Modifier<PoliceSuspicionModifier>,
    m_m: Modifier<MediaSuspicionModifier>,
    police_suspicion_changes: Query<&PoliceSuspicionChange>,
    media_suspicion_changes: Query<&MediaSuspicionChange>,
    children: Query<&Children>,
    random: Res<RandomSource>,
) {
    let intel = intel_suspicion_changes
        .iter()
        .map(|(entity, change)| m_i.calc(change.0 as f64, entity))
        .sum::<f64>();
    let scien = scien_suspicion_changes
        .iter()
        .map(|(entity, change)| m_s.calc(change.0 as f64, entity))
        .sum::<f64>();

    update_suspicion_inner(
        commands.reborrow(),
        None,
        intel_suspicion,
        SuspicionType::Intelligence,
        Either::Right((intel, &mut random.rng())),
    );
    update_suspicion_inner(
        commands.reborrow(),
        None,
        scien_suspicion,
        SuspicionType::Scientific,
        Either::Right((scien, &mut random.rng())),
    );

    for (entity, police_suspicion, media_suspicion) in &mut regions {
        let mut police = 0.0;
        let mut media = 0.0;

        for desc in children.iter_descendants(entity) {
            if let Ok(police_suspicion_change) = police_suspicion_changes.get(desc) {
                police += m_p.calc(police_suspicion_change.0 as f64, desc);
            }
            if let Ok(media_suspicion_change) = media_suspicion_changes.get(desc) {
                media += m_m.calc(media_suspicion_change.0 as f64, desc);
            }
        }

        update_suspicion_inner(
            commands.reborrow(),
            Some(entity),
            police_suspicion,
            SuspicionType::Police,
            Either::Right((police, &mut random.rng())),
        );
        update_suspicion_inner(
            commands.reborrow(),
            Some(entity),
            media_suspicion,
            SuspicionType::Media,
            Either::Right((media, &mut random.rng())),
        );
    }
}

pub fn add_suspicion_change(
    entity_commands: &mut EntityCommands,
    suspicion: SuspicionType,
    amount: f32,
) {
    match suspicion {
        SuspicionType::Intelligence => entity_commands.insert(IntelligenceSuspicionChange(amount)),
        SuspicionType::Scientific => entity_commands.insert(ScientificSuspicionChange(amount)),
        SuspicionType::Police => entity_commands.insert(PoliceSuspicionChange(amount)),
        SuspicionType::Media => entity_commands.insert(MediaSuspicionChange(amount)),
    };
}

pub fn add_suspicion(
    In((region_entity, suspicion, amount)): In<(Option<Entity>, SuspicionType, i32)>,
    mut commands: Commands,
    intel_suspicion: ResMut<IntelligenceSuspicion>,
    scien_suspicion: ResMut<ScientificSuspicion>,
    mut regions: Query<(&mut PoliceSuspicion, &mut MediaSuspicion), With<Region>>,
) {
    match suspicion {
        SuspicionType::Intelligence => {
            update_suspicion_inner(
                commands.reborrow(),
                None,
                intel_suspicion,
                suspicion,
                Either::Left(amount),
            );
        }
        SuspicionType::Scientific => {
            update_suspicion_inner(
                commands.reborrow(),
                None,
                scien_suspicion,
                suspicion,
                Either::Left(amount),
            );
        }
        SuspicionType::Police => {
            if let Some(region_entity) = region_entity
                && let Ok((police_suspicion, _)) = regions.get_mut(region_entity)
            {
                update_suspicion_inner(
                    commands.reborrow(),
                    Some(region_entity),
                    police_suspicion,
                    suspicion,
                    Either::Left(amount),
                );
            } else {
                error!("wrong suspicion entity");
            }
        }
        SuspicionType::Media => {
            if let Some(region_entity) = region_entity
                && let Ok((_, media_suspicion)) = regions.get_mut(region_entity)
            {
                update_suspicion_inner(
                    commands.reborrow(),
                    Some(region_entity),
                    media_suspicion,
                    suspicion,
                    Either::Left(amount),
                );
            } else {
                error!("wrong suspicion entity");
            }
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save, DespawnOnExit::<GameState>(GameState::Main))]
pub struct SuspicionEvent(pub String);

#[derive(Deserialize, Asset, TypePath)]
pub struct SuspicionEventsAsset(pub IndexMap<String, SuspicionEventSettings>);

#[derive(Resource)]
pub struct SuspicionEventsHandle(pub Handle<SuspicionEventsAsset>);

#[derive(Deserialize, Clone)]
pub struct SuspicionEventChoice {
    name: String,
    delay: Option<u32>,
    effects: Vec<Effect>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SuspicionEventSettings {
    pub major: bool,
    pub suspicion_type: SuspicionType,
    pub delay: Option<u32>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub choices: Vec<SuspicionEventChoice>,
}

const SUSPICION_EVENTS_ASSET_PATH: &str = "data/define.suspicion-events.toml";

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SuspicionEventsHandle(
        asset_server.load(SUSPICION_EVENTS_ASSET_PATH),
    ));
}

fn spawn_suspicion_event(
    In((suspicion_type, major, entity)): In<(SuspicionType, bool, Option<Entity>)>,
    mut commands: Commands,
    date: Res<GameDate>,
    suspicion_events_handle: Res<SuspicionEventsHandle>,
    suspicion_events_asset: Res<Assets<SuspicionEventsAsset>>,
    random: Res<RandomSource>,
) {
    let suspicion_events = suspicion_events_asset
        .get(suspicion_events_handle.0.id())
        .unwrap()
        .0
        .iter()
        .filter(|(_, s)| s.major == major && s.suspicion_type == suspicion_type)
        .collect::<Vec<_>>();

    let (name, setting) = *suspicion_events.choose(&mut random.rng()).unwrap();
    let source = Source::SuspicionEvent(name.clone());

    info!("suspicion event fired: {name}");

    if let Some(delay) = setting.delay {
        let effects = setting.effects.clone();
        commands.spawn(EndDate::new(date.0, delay)).observe(
            move |_: On<Despawn, EndDate>, mut commands: Commands| {
                warn!("delayed");
                for effect in effects.clone() {
                    commands.run_system_cached_with(
                        apply_effect,
                        (entity, None, effect.clone(), Some(source.clone())),
                    );
                }
            },
        );
    } else {
        for effect in &setting.effects {
            commands.run_system_cached_with(
                apply_effect,
                (entity, None, effect.clone(), Some(source.clone())),
            );
        }
    }
}
