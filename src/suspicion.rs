use bevy::prelude::*;
use rand::RngExt;
use rand_distr::Poisson;
use serde::Deserialize;

use crate::{
    bases::{Base, BasetypesAsset, BasetypesHandle},
    regions::{BasePlot, Region},
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

fn setup_main(mut commands: Commands) {
    commands.init_resource::<IntelligenceSuspicion>();
    commands.init_resource::<ScientificSuspicion>();
}

fn update_suspicion(
    mut intel_suspicion: ResMut<IntelligenceSuspicion>,
    mut scien_suspicion: ResMut<ScientificSuspicion>,
    mut regions: Query<(&mut PoliceSuspicion, &mut MediaSuspicion, &Children), With<Region>>,
    base_plots: Query<&Children, With<BasePlot>>,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    bases: Query<&Base>,
    mut random: ResMut<RandomSource>,
) {
    intel_suspicion.0 += random.0.sample(Poisson::new(1.0).unwrap()) as u32;
    scien_suspicion.0 += random.0.sample(Poisson::new(1.0).unwrap()) as u32;

    let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;

    for (mut police_suspicion, mut media_suspicion, children) in regions.iter_mut() {
        let mut police = 0;
        let mut media = 0;

        for child in children {
            if let Ok(children) = base_plots.get(*child) {
                for child in children {
                    let base_type = bases.get(*child).unwrap();
                    let settings = base_types.get(&base_type.0).unwrap();
                    police += settings.police_suspicion;
                    media += settings.media_suspicion;
                }
            }
        }

        if police != 0 {
            police_suspicion.0 += random.0.sample(Poisson::new(police as f32).unwrap()) as u32;
        }

        if media != 0 {
            media_suspicion.0 += random.0.sample(Poisson::new(media as f32).unwrap()) as u32;
        }
    }
}
