use std::sync::{Mutex, MutexGuard};

use bevy::prelude::*;
use rand::{make_rng, rngs::StdRng};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_rng);
}

#[derive(Resource)]
pub struct RandomSource(Mutex<StdRng>);

impl RandomSource {
    pub fn rng(&self) -> MutexGuard<'_, StdRng> {
        self.0.lock().unwrap()
    }
}

fn setup_rng(mut commands: Commands) {
    commands.insert_resource(RandomSource(Mutex::new(make_rng())));
}
