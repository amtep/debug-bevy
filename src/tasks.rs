use std::collections::HashMap;

use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use moonshine_save::save::Save;
use serde_derive::Deserialize;

use crate::{
    followers::FollowerCount,
    funds::{FundsAmount, Income, IncomeCategory},
    state::GameState,
    suspicion::SuspicionType,
};

const TASKS_ASSET_PATH: &str = "data/define.tasks.toml";

pub const DEFAULT_TASK: &str = "gig-work";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<TasksAsset>::new(&["tasks.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
        .add_systems(Update, switch_tasks);
}

#[derive(Deserialize, Asset, TypePath)]
struct TasksAsset(pub HashMap<String, TaskSettings>);

#[derive(Resource)]
struct TasksHandle(pub Handle<TasksAsset>);

#[derive(Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "kebab-case")]
struct TaskSettings {
    #[serde(default)]
    priests_allowed: bool,
    #[serde(default)]
    minions_allowed: bool,
    #[serde(default)]
    profit_per_day: FundsAmount,
    #[serde(default)]
    profit_category: Option<IncomeCategory>,
    #[serde(default)]
    suspicion: HashMap<SuspicionType, u32>,
    #[serde(default)]
    recruit_progress: usize,
    #[serde(default)]
    research: usize,
}

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TasksHandle(asset_server.load(TASKS_ASSET_PATH)));
}

/// A component added as a child of a Follower entity, to mark this as a task those followers are doing.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save)]
pub struct Task(pub String);

fn switch_tasks(
    mut commands: Commands,
    tasks: Res<TasksHandle>,
    asset: Res<Assets<TasksAsset>>,
    changed: Populated<(Entity, &ChildOf, &Task), Changed<Task>>,
    followers: Query<&FollowerCount>,
) {
    // SAFETY: followers are only spawned after all assets are loaded.
    let settings_hash = asset.get(tasks.0.id()).unwrap();
    for (task_e, ChildOf(parent), Task(task)) in changed {
        let Some(settings) = settings_hash.0.get(task) else {
            error!("Task {task} not known");
            continue;
        };
        let Ok(count) = followers.get(*parent) else {
            error!("Task without Follower parent");
            continue;
        };
        // Handle task income
        if let Some(cat) = settings.profit_category
            && settings.profit_per_day > 0
        {
            commands
                .entity(task_e)
                .insert(Income(settings.profit_per_day, cat, count.0));
        } else {
            commands.entity(task_e).remove::<Income>();
        }
        // TODO: handle recruitment
        // TODO: handle research
    }
}
