use std::collections::HashMap;

use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use moonshine_save::save::Save;
use serde_derive::Deserialize;

use crate::{
    followers::Follower,
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

/// A component to be added to a base entity, representing what the priests in that base are doing.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save)]
pub struct PriestsTask(pub String);

/// A component to be added to a base entity, representing what the minions in that base are doing.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save)]
pub struct MinionsTask(pub String);

fn switch_tasks(
    mut commands: Commands,
    tasks: Res<TasksHandle>,
    asset: Res<Assets<TasksAsset>>,
    changed_priests: Query<(Entity, &PriestsTask, &Children), Changed<PriestsTask>>,
    changed_minions: Query<(Entity, &MinionsTask, &Children), Changed<MinionsTask>>,
    followers: Query<&Follower>,
) {
    // SAFETY: followers are only spawned after all assets are loaded.
    let settings_hash = asset.get(tasks.0.id()).unwrap();
    for (base, PriestsTask(task), children) in changed_priests {
        let Some(settings) = settings_hash.0.get(task) else {
            error!("Priests task {task} not known");
            continue;
        };
        switch_task(
            commands.reborrow(),
            base,
            task,
            settings,
            children,
            followers,
            Follower::Priest,
        );
    }
    for (base, MinionsTask(task), children) in changed_minions {
        let Some(settings) = settings_hash.0.get(task) else {
            error!("Minions task {task} not known");
            continue;
        };
        switch_task(
            commands.reborrow(),
            base,
            task,
            settings,
            children,
            followers,
            Follower::Minion,
        );
    }
}

fn switch_task(
    mut commands: Commands,
    _base: Entity,
    _task: &String,
    settings: &TaskSettings,
    children: &[Entity],
    followers: Query<&Follower>,
    ftype: Follower,
) {
    // Handle task income
    for child in children {
        if let Ok(follower) = followers.get(*child)
            && *follower == ftype
        {
            if let Some(cat) = settings.profit_category
                && settings.profit_per_day > 0
            {
                commands
                    .entity(*child)
                    .insert(Income(settings.profit_per_day, cat));
            } else {
                commands.entity(*child).remove::<Income>();
            }
        }
    }
    // TODO: handle recruitment
    // TODO: handle research
}
