use bevy::prelude::*;
use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::{
    new_game::NewGame,
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
    .add_observer(on_force_pause_remove)
    .add_observer(on_game_speed_changed);
}

#[derive(Resource, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
#[reflect(Serialize)]
#[reflect(Deserialize)]
#[reflect(opaque)]
pub struct GameDate(pub NaiveDate);

/// A marker struct that makes the game pause when it's added and
/// prevents unpausing until it's removed or despawned.
#[derive(Component)]
pub struct ForcePause;

const START_DATE: NaiveDate = NaiveDate::from_ymd_opt(2026, 4, 15).unwrap();

impl Default for GameDate {
    fn default() -> Self {
        Self(START_DATE)
    }
}

impl GameDate {
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn days_since_start(&self) -> usize {
        (self.0 - START_DATE).num_days() as usize
    }
}

fn setup(mut commands: Commands, mut time: ResMut<Time<Virtual>>) {
    time.set_relative_speed(1.0);
    time.unpause();
    commands.insert_resource(Time::<Fixed>::from_seconds(1.0));
    commands.insert_resource(CurrentGameSpeed::default());
}

fn new_game(mut commands: Commands) {
    commands.insert_resource(GameDate::default());
}

/// The "source of truth" for game speed state.
/// The state of `Time<Virtual>` is derived from this resource.
#[derive(Resource, Default)]
pub struct CurrentGameSpeed {
    /// Whether the clock is currently forced to pause.
    /// This is distinct from `paused` because that is the user's desired setting.
    /// When `forced_paused` is over, we should return to the state indicated by `paused` and `speed`.
    pub forced_paused: bool,
    /// Whether the user elected to pause the game.
    pub paused: bool,
    /// Which speed the game should run at when not paused.
    /// This remains valid even when `paused` or `forced_paused`, so that the game can return
    /// to the user's desired speed setting when unpaused.
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

/// Process a game speed event, either from user input or internal UI logic.
fn on_game_speed_changed(
    event: On<GameSpeedChangedEvent>,
    mut time: ResMut<Time<Virtual>>,
    mut current_game_speed: ResMut<CurrentGameSpeed>,
) {
    match event.0 {
        GameSpeedAction::SetSpeed(speed) => {
            let s = speed.get();
            info!("Game speed to {s}");
            time.set_relative_speed(s);
            if !current_game_speed.forced_paused {
                time.unpause();
            }
            *current_game_speed = CurrentGameSpeed {
                forced_paused: current_game_speed.forced_paused,
                paused: false,
                speed,
            };
        }
        GameSpeedAction::TogglePause => {
            if current_game_speed.paused {
                info!("Unpausing");
                if !current_game_speed.forced_paused {
                    time.unpause();
                }
            } else {
                info!("Pausing");
                time.pause();
            }
            current_game_speed.paused = !current_game_speed.paused;
        }
    }
}

/// Pause the game if someone inserts a [`ForcePause`] component.
fn on_force_pause_insert(
    _: On<Insert, ForcePause>,
    mut current_game_speed: ResMut<CurrentGameSpeed>,
    mut time: ResMut<Time<Virtual>>,
) {
    current_game_speed.forced_paused = true;
    if !time.is_paused() {
        info!("Forced pausing");
        time.pause();
    }
}

/// Unpause the game if the last [`ForcePause`] component is removed.
fn on_force_pause_remove(
    _: On<Remove, ForcePause>,
    q: Query<(), With<ForcePause>>,
    mut current_game_speed: ResMut<CurrentGameSpeed>,
    mut time: ResMut<Time<Virtual>>,
) {
    // Check for count 1, because we're called just before the component is removed.
    if q.count() == 1 {
        current_game_speed.forced_paused = false;
        if !current_game_speed.paused {
            info!("Forced unpausing");
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
