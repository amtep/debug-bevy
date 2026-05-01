use bevy::prelude::*;

use crate::funds::Funds;
use crate::state::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, listen_dev_keys.run_if(in_state(GameState::Main)));
}

#[cfg(feature = "dev")]
fn listen_dev_keys(keys: Res<ButtonInput<KeyCode>>, mut funds: ResMut<Funds>) {
    if keys.just_pressed(KeyCode::KeyF) {
        if keys.pressed(KeyCode::AltLeft) {
            // Alt + F
            funds.0 += 100000;
        }
    }
}
