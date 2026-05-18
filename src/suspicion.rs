use bevy::prelude::*;
use moonshine_save::save::Save;
use rand::RngExt;
use rand_distr::Poisson;
use serde::Deserialize;
use strum::Display;

use crate::{
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

pub fn plugin(app: &mut App) {
    app.add_systems(
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
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct IntelligenceSuspicion(pub u32);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ScientificSuspicion(pub u32);

// regional
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct PoliceSuspicion(pub u32);

#[derive(Component, Default, Reflect)]
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
        IntelligenceSuspicionChange(1.0),
        ScientificSuspicionChange(1.0),
        Save,
    ));
}

#[expect(clippy::cast_possible_truncation, reason = "it's random values anyway")]
#[expect(clippy::cast_sign_loss, reason = "it's random values anyway")]
fn update_suspicion(
    mut intel_suspicion: ResMut<IntelligenceSuspicion>,
    mut scien_suspicion: ResMut<ScientificSuspicion>,
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
    mut random: ResMut<RandomSource>,
) {
    let intel_suspicion_change = intel_suspicion_changes
        .iter()
        .map(|(entity, change)| m_i.calc(change.0 as f64, entity))
        .sum::<f64>();
    let scien_suspicion_change = scien_suspicion_changes
        .iter()
        .map(|(entity, change)| m_s.calc(change.0 as f64, entity))
        .sum::<f64>();

    if intel_suspicion_change > 0.0 {
        intel_suspicion.0 += random
            .0
            .sample(Poisson::new(intel_suspicion_change).unwrap())
            as u32;
    }
    if scien_suspicion_change > 0.0 {
        scien_suspicion.0 += random
            .0
            .sample(Poisson::new(scien_suspicion_change).unwrap())
            as u32;
    }

    for (entity, mut police_suspicion, mut media_suspicion) in &mut regions {
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

        if police > 0.0 {
            police_suspicion.0 += random.0.sample(Poisson::new(police).unwrap()) as u32;
        }

        if media > 0.0 {
            media_suspicion.0 += random.0.sample(Poisson::new(media).unwrap()) as u32;
        }
    }
}

pub fn add_suspicion_changes(
    mut commands: Commands,
    entity: Entity,
    count: usize,
    suspicions: impl IntoIterator<Item = (SuspicionType, f32)>,
) {
    for (suspicion, amount) in suspicions {
        #[allow(clippy::cast_possible_truncation)]
        let amount = count as f32 * amount;
        match suspicion {
            SuspicionType::Intelligence => commands
                .entity(entity)
                .insert(IntelligenceSuspicionChange(amount)),
            SuspicionType::Scientific => commands
                .entity(entity)
                .insert(ScientificSuspicionChange(amount)),
            SuspicionType::Police => commands
                .entity(entity)
                .insert(PoliceSuspicionChange(amount)),
            SuspicionType::Media => commands.entity(entity).insert(MediaSuspicionChange(amount)),
        };
    }
}

pub fn add_suspicions(
    In((region_entity, count, suspicions)): In<(
        Entity,
        usize,
        impl IntoIterator<Item = (SuspicionType, u32)>,
    )>,
    mut intel_suspicion: ResMut<IntelligenceSuspicion>,
    mut scien_suspicion: ResMut<ScientificSuspicion>,
    mut regions: Query<(&mut PoliceSuspicion, &mut MediaSuspicion), With<Region>>,
) {
    for (suspicion, amount) in suspicions {
        #[allow(clippy::cast_possible_truncation)]
        let amount = count as u32 * amount;
        match suspicion {
            SuspicionType::Intelligence => intel_suspicion.0 += amount,
            SuspicionType::Scientific => scien_suspicion.0 += amount,
            SuspicionType::Police => regions.get_mut(region_entity).unwrap().0.0 += amount,
            SuspicionType::Media => regions.get_mut(region_entity).unwrap().1.0 += amount,
        }
    }
}
