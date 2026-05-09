use bevy::prelude::*;
use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::{
    main_menu::NewGame,
    state::{GameState, MainSetupSet},
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Main),
        setup.in_set(MainSetupSet::Default),
    )
    .add_systems(
        OnEnter(GameState::Main),
        new_game
            .run_if(resource_exists::<NewGame>)
            .in_set(MainSetupSet::Default),
    )
    .add_systems(
        FixedPreUpdate,
        fixed_pre_update.run_if(in_state(GameState::Main)),
    )
    .add_systems(Update, listen_speed_keys.run_if(in_state(GameState::Main)))
    .add_observer(on_force_pause_insert)
    .add_observer(on_force_pause_remove);
}

#[derive(Resource, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
#[reflect(Serialize)]
#[reflect(Deserialize)]
#[reflect(opaque)]
pub struct GameDate(pub NaiveDate);

/// A marker struct that makes the game pause when it's added and
/// prevents unpausing until it's removed or despawned.
#[derive(Component)]
pub struct ForcePause;

impl Default for GameDate {
    fn default() -> Self {
        Self(NaiveDate::from_ymd_opt(2026, 4, 15).unwrap())
    }
}

fn setup(mut commands: Commands, mut time: ResMut<Time<Virtual>>) {
    time.set_relative_speed(1.0);
    time.unpause();
    commands.insert_resource(Time::<Fixed>::from_seconds(1.0));
    commands.insert_resource(CurrentGameSpeed::default());
    commands.add_observer(on_game_speed_changed);
}

fn new_game(mut commands: Commands) {
    commands.init_resource::<GameDate>();
}

#[derive(Resource, Default)]
pub struct CurrentGameSpeed {
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
}

#[derive(Event)]
pub struct GameSpeedChangedEvent(pub GameSpeedAction);

fn on_game_speed_changed(
    event: On<GameSpeedChangedEvent>,
    mut time: ResMut<Time<Virtual>>,
    mut current_game_speed: ResMut<CurrentGameSpeed>,
    forced_pause: Query<(), With<ForcePause>>,
) {
    let can_unpause = forced_pause.is_empty();
    match event.0 {
        GameSpeedAction::SetSpeed(speed) if can_unpause => {
            let s = speed.get();
            info!("Game speed to {s}");
            time.set_relative_speed(s);
            time.unpause();
            *current_game_speed = CurrentGameSpeed {
                paused: false,
                speed,
            };
        }
        GameSpeedAction::TogglePause if can_unpause => {
            if current_game_speed.paused {
                info!("Unpausing");
                time.unpause();
            } else {
                info!("Pausing");
                time.pause();
            }
            current_game_speed.paused = !current_game_speed.paused;
        }
        _ => (),
    }
}

/// Pause the game if someone inserts a [`ForcePause`] component.
fn on_force_pause_insert(_: On<Insert, ForcePause>, mut time: ResMut<Time<Virtual>>) {
    if !time.is_paused() {
        info!("ForcePause");
        time.pause();
    }
}

/// Unpause the game if the last [`ForcePause`] component is removed.
fn on_force_pause_remove(
    _: On<Remove, ForcePause>,
    mut time: ResMut<Time<Virtual>>,
    q: Query<(), With<ForcePause>>,
    current_game_speed: ResMut<CurrentGameSpeed>,
) {
    // Check for count 1, because we're called just before the component is removed.
    if q.count() == 1 {
        info!("Un-ForcePause");
        if !current_game_speed.paused {
            time.set_relative_speed(current_game_speed.speed.get());
            time.unpause();
        }
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
