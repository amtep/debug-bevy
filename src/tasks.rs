use bevy::prelude::*;
use bevy::reflect::Is;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use serde_derive::Deserialize;

use crate::{
    discoveries::ResearchPoints,
    followers::FollowerCount,
    funds::{Expense, FundsAmount, Income},
    state::GameState,
    suspicion::SuspicionType,
};

const TASKS_ASSET_PATH: &str = "data/define.tasks.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<TasksAsset>::new(&["tasks.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
        .add_systems(FixedUpdate, research.run_if(in_state(GameState::Main)))
        .add_observer(on_task_changed::<Task>)
        .add_observer(on_task_changed::<FollowerCount>);
}

#[derive(Deserialize, Asset, TypePath)]
pub struct TasksAsset(pub IndexMap<String, TaskSettings>);

#[derive(Resource)]
pub struct TasksHandle(pub Handle<TasksAsset>);

#[derive(Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "kebab-case")]
pub struct TaskSettings {
    pub follower_types: Vec<String>,
    #[serde(default)]
    pub requires_discovery: Option<String>,
    pub income_per_day: Option<(FundsAmount, String)>,
    pub expense_per_day: Option<(FundsAmount, String)>,
    #[serde(default)]
    pub suspicion: IndexMap<SuspicionType, u32>,
    #[serde(default)]
    pub recruit_progress: f64,
    #[serde(default)]
    pub research: usize,
    #[serde(default)]
    pub security: usize,
}

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TasksHandle(asset_server.load(TASKS_ASSET_PATH)));
}

/// A component added as a child of a Follower entity, to mark this as a task those followers are doing.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save)]
#[component(immutable)]
pub struct Task(pub String);

// update both on follower count change and task change
fn on_task_changed<C: Component>(
    insert: On<Insert, C>,
    mut commands: Commands,
    state: Res<State<GameState>>,
    task_handle: Res<TasksHandle>,
    task_assets: Res<Assets<TasksAsset>>,
    tasks: Query<(Entity, &ChildOf, &Task)>,
    followers: Query<&FollowerCount>,
    follower_children: Query<&Children, With<FollowerCount>>,
) {
    if *state != GameState::Main {
        return;
    }

    let task_settings = &task_assets.get(task_handle.0.id()).unwrap().0;

    let entity = if C::is::<Task>() {
        insert.entity
    } else if let Ok(children) = follower_children.get(insert.entity)
        && let Some(task_entity) = children.iter().find(|e| tasks.contains(*e))
    {
        task_entity
    } else {
        return;
    };

    let (entity, ChildOf(parent), Task(task)) = tasks.get(entity).unwrap();

    let Some(settings) = task_settings.get(task) else {
        error!("Task {task} not known");
        return;
    };

    let Ok(count) = followers.get(*parent) else {
        error!("Task without Follower parent");
        return;
    };

    // Handle task income/expense
    if let Some((income, category)) = &settings.income_per_day {
        commands
            .entity(entity)
            .insert(Income(*income, category.clone(), count.0));
    } else {
        commands.entity(entity).try_remove::<Income>();
    }

    if let Some((expense, category)) = &settings.expense_per_day {
        commands
            .entity(entity)
            .insert(Expense(*expense, category.clone(), count.0));
    } else {
        commands.entity(entity).try_remove::<Expense>();
    }

    // TODO: handle suspicions
    // TODO: handle research
}

fn research(
    tasks: Query<(&ChildOf, &Task)>,
    followers: Query<&FollowerCount>,
    task_handle: Res<TasksHandle>,
    task_assets: Res<Assets<TasksAsset>>,
    mut points: ResMut<ResearchPoints>,
) {
    let task_types = &task_assets.get(task_handle.0.id()).unwrap().0;
    for (ChildOf(follower_e), Task(task)) in tasks {
        let Some(task_settings) = task_types.get(task) else {
            error!("Unknown task '{task}'");
            continue;
        };
        if task_settings.research > 0 {
            let Ok(count) = followers.get(*follower_e) else {
                error!("Task without followers");
                continue;
            };
            points.0 += task_settings.research * **count;
        }
    }
}
