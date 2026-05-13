use bevy::prelude::*;
use rand::RngExt;
use rand_distr::Poisson;
use serde::Deserialize;

use crate::{
    regions::Region,
    rng::RandomSource,
    state::{GameState, MainSetupSet},
    time::GameDate,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Main),
        setup_main.in_set(MainSetupSet::Default),
    )
    .add_systems(
        FixedUpdate,
        update_suspicion
            .run_if(resource_exists_and_changed::<GameDate>.and(in_state(GameState::Main))),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Deserialize)]
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
    commands.init_resource::<IntelligenceSuspicion>();
    commands.init_resource::<ScientificSuspicion>();
}

#[expect(clippy::cast_possible_truncation, reason = "it's random values anyway")]
#[expect(clippy::cast_sign_loss, reason = "it's random values anyway")]
fn update_suspicion(
    mut intel_suspicion: ResMut<IntelligenceSuspicion>,
    mut scien_suspicion: ResMut<ScientificSuspicion>,
    mut regions: Query<(Entity, &mut PoliceSuspicion, &mut MediaSuspicion), With<Region>>,
    intel_suspicion_changes: Query<&IntelligenceSuspicionChange>,
    scien_suspicion_changes: Query<&ScientificSuspicionChange>,
    police_suspicion_changes: Query<&PoliceSuspicionChange>,
    media_suspicion_changes: Query<&MediaSuspicionChange>,
    children: Query<&Children>,
    mut random: ResMut<RandomSource>,
) {
    let intel_suspicion_change = 1.0 + intel_suspicion_changes.iter().map(|s| s.0).sum::<f32>();
    let scien_suspicion_change = 1.0 + scien_suspicion_changes.iter().map(|s| s.0).sum::<f32>();

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
                police += police_suspicion_change.0;
            }
            if let Ok(media_suspicion_change) = media_suspicion_changes.get(desc) {
                media += media_suspicion_change.0;
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
