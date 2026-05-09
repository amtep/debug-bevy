use bevy::prelude::*;
use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::state::{GameState, MainSetupSet};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Main),
        setup.in_set(MainSetupSet::Default),
    )
    .add_systems(
        FixedPreUpdate,
        fixed_pre_update.run_if(in_state(GameState::Main)),
    )
    .add_systems(Update, listen_speed_keys.run_if(in_state(GameState::Main)));
}

#[derive(Resource, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
#[reflect(Serialize)]
#[reflect(Deserialize)]
#[reflect(opaque)]
pub struct GameDate(pub NaiveDate);

impl Default for GameDate {
    fn default() -> Self {
        Self(NaiveDate::from_ymd_opt(2026, 4, 15).unwrap())
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Time::<Fixed>::from_seconds(1.0));
    commands.insert_resource(GameDate::default());
    commands.insert_resource(CurrentGameSpeed::default());
    commands.add_observer(on_game_speed_changed);
}

#[derive(Resource, Default)]
pub struct CurrentGameSpeed {
    pub dialog_open: u32,
    pub paused: bool,
    pub speed: GameSpeed,
}

fn fixed_pre_update(mut date: ResMut<GameDate>) {
    // We don't expect to reach 262000 AD
    date.0 = date.0 + Days::new(1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameSpeed {
    #[default]
    Normal,
    Fast,
    Faster,
}

impl GameSpeed {
    const fn get(self) -> f32 {
        match self {
            GameSpeed::Normal => 1.0,
            GameSpeed::Fast => 2.0,
            GameSpeed::Faster => 5.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum GameSpeedAction {
    SetSpeed(GameSpeed),
    TogglePause,
    UiOpen,
    UiClose,
}

#[derive(Event)]
pub struct GameSpeedChangedEvent(pub GameSpeedAction);

fn on_game_speed_changed(
    event: On<GameSpeedChangedEvent>,
    mut time: ResMut<Time<Virtual>>,
    mut current_game_speed: ResMut<CurrentGameSpeed>,
) {
    match event.0 {
        GameSpeedAction::SetSpeed(speed) if current_game_speed.dialog_open == 0 => {
            let s = speed.get();
            info!("Game speed to {s}");
            time.set_relative_speed(s);
            time.unpause();
            *current_game_speed = CurrentGameSpeed {
                dialog_open: 0,
                paused: false,
                speed,
            };
        }
        GameSpeedAction::TogglePause if current_game_speed.dialog_open == 0 => {
            if current_game_speed.paused {
                info!("Unpausing");
                time.unpause();
            } else {
                info!("Pausing");
                time.pause();
            }
            current_game_speed.paused = !current_game_speed.paused;
        }
        GameSpeedAction::UiOpen => {
            current_game_speed.dialog_open += 1;
            if !time.is_paused() {
                time.pause();
            }
        }
        GameSpeedAction::UiClose => {
            current_game_speed.dialog_open -= 1;
            if current_game_speed.dialog_open == 0 && !current_game_speed.paused {
                time.set_relative_speed(current_game_speed.speed.get());
                time.unpause();
            }
        }
        _ => (),
    }
}

fn listen_speed_keys(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    let action = if keys.just_pressed(KeyCode::Digit1) {
        GameSpeedAction::SetSpeed(GameSpeed::Normal)
    } else if keys.just_pressed(KeyCode::Digit2) {
        GameSpeedAction::SetSpeed(GameSpeed::Fast)
    } else if keys.just_pressed(KeyCode::Digit3) {
        GameSpeedAction::SetSpeed(GameSpeed::Faster)
    } else if keys.just_pressed(KeyCode::Space) {
        GameSpeedAction::TogglePause
    } else {
        return;
    };

    commands.trigger(GameSpeedChangedEvent(action));
}
