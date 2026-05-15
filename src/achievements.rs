use bevy::{platform::collections::HashSet, prelude::*};

use crate::{constants::achievements::*, new_game::NewGame, state::GameState, ui::dialog::Dialog};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Main), new_game)
        .add_systems(
            FixedUpdate,
            announce_achievements.run_if(in_state(GameState::Main)),
        )
        .add_observer(on_achieved_event);
}

/// A list of achievements achieved during this game.
/// The strings double as message keys, in the form `achievement-{key}`.
/// The message keys should have attributes `.title` and `.desc`.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Achievements(HashSet<String>);

impl Achievements {
    #[expect(dead_code)]
    const ALL: &[&str] = &[STARTED_RECRUITING, FIRST_MINION_RECRUIT];

    pub fn achieved(&self, achievement: &str) -> bool {
        self.0.contains(achievement)
    }
}

/// Achievements that have been achieved but not yet announced to the player.
/// These will pop up one by one.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct RecentlyAchieved(pub Vec<String>);

/// A marker component to check if there is currently qan achievement pop-up open.
#[derive(Component)]
struct AchievementDialog;

/// A convenience event to be sent when an achievement can be gained.
/// The sender does not have to worry about whether it was already achieved.
#[derive(Event)]
pub struct AchievedEvent {
    pub achievement: String,
}

fn new_game(mut commands: Commands, _: If<Res<NewGame>>) {
    commands.insert_resource(Achievements::default());
    commands.insert_resource(RecentlyAchieved::default());
}

fn on_achieved_event(
    ev: On<AchievedEvent>,
    achievements: Res<Achievements>,
    mut recent: ResMut<RecentlyAchieved>,
) {
    if !achievements.achieved(&ev.achievement) && !recent.0.contains(&ev.achievement) {
        recent.0.push(ev.achievement.clone());
    }
}

fn announce_achievements(
    mut commands: Commands,
    mut achievements: ResMut<Achievements>,
    mut recent: ResMut<RecentlyAchieved>,
    open: Query<&AchievementDialog>,
) {
    // Only one achievement pop-up at a time.
    if !open.is_empty() {
        return;
    }

    while !recent.0.is_empty() {
        let achievement = recent.0.remove(0);
        if !achievements.0.insert(achievement.clone()) {
            continue;
        }

        commands.spawn((
            Dialog::new()
                .with_title(format!("achievement-{achievement}.title"))
                .with_text_body(format!("achievement-{achievement}.desc")),
            AchievementDialog,
        ));

        return;
    }
}
