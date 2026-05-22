use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use rand::{RngExt, rngs::StdRng, seq::IndexedRandom};
use rand_distr::Poisson;
use serde::Deserialize;
use strum::Display;

use crate::{
    effects::Effect,
    modifiers::{
        IntelligenceSuspicionModifier, MediaSuspicionModifier, Modifier, PoliceSuspicionModifier,
        ScientificSuspicionModifier,
    },
    new_game::NewGame,
    regions::Region,
    rng::RandomSource,
    state::{GameState, MainSetupSet},
    time::GameDate,
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
#[derive(Resource, Default, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct IntelligenceSuspicion(pub u32);

#[derive(Resource, Default, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct ScientificSuspicion(pub u32);

// regional
#[derive(Component, Default, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct PoliceSuspicion(pub u32);

#[derive(Component, Default, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct MediaSuspicion(pub u32);

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
    suspicion_events_handle: Res<SuspicionEventsHandle>,
    suspicion_events_asset: Res<Assets<SuspicionEventsAsset>>,
) {
    #[expect(clippy::cast_possible_truncation, reason = "it's random values anyway")]
    #[expect(clippy::cast_sign_loss, reason = "it's random values anyway")]
    fn update_suspicion_inner<
        T: DetectChangesMut + std::ops::DerefMut<Target = U>,
        U: std::ops::DerefMut<Target = u32>,
    >(
        mut commands: Commands,
        entity: Option<Entity>,
        mut value: T,
        amount: f64,
        suspicion_type: SuspicionType,
        suspicion_events: &IndexMap<String, SuspicionEventSettings>,
        rng: &mut StdRng,
    ) {
        if amount <= 0.0 {
            return;
        }
        let change = rng.sample(Poisson::new(amount).unwrap()) as u32;
        let after = **value + change;

        if after == **value {
            return;
        }

        let mut spawn_event = |major| {
            let events: Vec<_> = suspicion_events
                .iter()
                .filter(|(_, v)| v.major == major && v.suspicion_type == suspicion_type)
                .map(|(k, _)| k)
                .collect();
            let event = (*events.choose(rng).unwrap()).clone();
            if let Some(entity) = entity {
                commands.entity(entity).insert(SuspicionEvent(event));
            } else {
                commands.spawn(SuspicionEvent(event));
            }
        };

        if **value < LOWER_LIMIT && after >= LOWER_LIMIT
            || **value < MIDDLE_LIMIT && after >= MIDDLE_LIMIT
        {
            **value = after;
            spawn_event(false);
        } else if **value < UPPER_LIMIT && after >= UPPER_LIMIT {
            **value = after - UPPER_LIMIT;
            spawn_event(true);
        } else {
            **value = after;
        }
    }

    let suspicion_events = &suspicion_events_asset
        .get(suspicion_events_handle.0.id())
        .unwrap()
        .0;

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
        intel,
        SuspicionType::Intelligence,
        suspicion_events,
        &mut random.rng(),
    );
    update_suspicion_inner(
        commands.reborrow(),
        None,
        scien_suspicion,
        scien,
        SuspicionType::Scientific,
        suspicion_events,
        &mut random.rng(),
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
            police,
            SuspicionType::Police,
            suspicion_events,
            &mut random.rng(),
        );
        update_suspicion_inner(
            commands.reborrow(),
            Some(entity),
            media_suspicion,
            media,
            SuspicionType::Media,
            suspicion_events,
            &mut random.rng(),
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
    In((region_entity, suspicion, amount)): In<(Entity, SuspicionType, i32)>,
    mut intel_suspicion: ResMut<IntelligenceSuspicion>,
    mut scien_suspicion: ResMut<ScientificSuspicion>,
    mut regions: Query<(&mut PoliceSuspicion, &mut MediaSuspicion), With<Region>>,
) {
    match suspicion {
        SuspicionType::Intelligence => {
            intel_suspicion.0 = intel_suspicion.0.saturating_add_signed(amount);
        }
        SuspicionType::Scientific => {
            scien_suspicion.0 = scien_suspicion.0.saturating_add_signed(amount);
        }
        SuspicionType::Police => {
            if let Ok(mut region) = regions.get_mut(region_entity) {
                region.0.0 = region.0.0.saturating_add_signed(amount);
            }
        }
        SuspicionType::Media => {
            if let Ok(mut region) = regions.get_mut(region_entity) {
                region.1.0 = region.1.0.saturating_add_signed(amount);
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
