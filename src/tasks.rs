use bevy::prelude::*;
use bevy::reflect::Is;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use serde_derive::Deserialize;

use crate::{
    achievements::AchievedEvent,
    discoveries::Research,
    followers::{FollowerCount, Recruit, RecruitProgress},
    funds::{Expense, FundsAmount, Income},
    state::GameState,
    suspicion::{
        IntelligenceSuspicionChange, MediaSuspicionChange, PoliceSuspicionChange,
        ScientificSuspicionChange, SuspicionType, add_suspicion_change,
    },
};

const TASKS_ASSET_PATH: &str = "data/define.tasks.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<TasksAsset>::new(&["tasks.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
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
    pub requires_discovery: Option<String>,
    pub income_per_day: Option<(FundsAmount, String)>,
    pub expense_per_day: Option<(FundsAmount, String)>,
    #[serde(default)]
    pub suspicions: IndexMap<SuspicionType, f32>,
    pub recruit_progress: Option<(String, f32)>,
    #[serde(default)]
    pub research: u32,
    #[serde(default)]
    pub security: u32,
    pub achievement: Option<String>,
}

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TasksHandle(asset_server.load(TASKS_ASSET_PATH)));
}

/// A component added as a child of a Follower entity, to mark this as a task those followers are doing.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save, RecruitProgress)]
#[component(immutable)]
pub struct Task(pub String);

// update both on follower count change and task change
fn on_task_changed<C: Component>(
    insert: On<Insert, C>,
    mut commands: Commands,
    state: Res<State<GameState>>,
    task_handle: Res<TasksHandle>,
    task_assets: Res<Assets<TasksAsset>>,
    tasks: Query<(&ChildOf, &Task)>,
    followers: Query<&FollowerCount>,
    follower_children: Query<&Children, With<FollowerCount>>,
) {
    if *state != GameState::Main {
        return;
    }

    let task_settings = &task_assets.get(task_handle.0.id()).unwrap().0;

    let task_entity = if C::is::<Task>() {
        insert.entity
    } else if let Ok(children) = follower_children.get(insert.entity)
        && let Some(task_entity) = children.iter().find(|e| tasks.contains(*e))
    {
        task_entity
    } else {
        return;
    };

    let (ChildOf(follower_entity), Task(task)) = tasks.get(task_entity).unwrap();

    let Some(settings) = task_settings.get(task) else {
        error!("Task {task} not known");
        return;
    };

    if let Some(achievement) = settings.achievement.as_ref() {
        commands.trigger(AchievedEvent {
            achievement: achievement.clone(),
        });
    }

    let Ok(count) = followers.get(*follower_entity) else {
        error!("Task without Follower parent");
        return;
    };

    // Handle task income/expense
    if let Some((income, category)) = &settings.income_per_day {
        commands
            .entity(task_entity)
            .insert(Income(*income, category.clone(), count.0));
    } else {
        commands.entity(task_entity).try_remove::<Income>();
    }

    if let Some((expense, category)) = &settings.expense_per_day {
        commands
            .entity(task_entity)
            .insert(Expense(*expense, category.clone(), count.0));
    } else {
        commands.entity(task_entity).try_remove::<Expense>();
    }

    commands.entity(task_entity).try_remove::<(
        IntelligenceSuspicionChange,
        ScientificSuspicionChange,
        PoliceSuspicionChange,
        MediaSuspicionChange,
    )>();

    for (suspicion, amount) in &settings.suspicions {
        add_suspicion_change(
            &mut commands.entity(task_entity),
            *suspicion,
            *amount * count.0 as f32,
        );
    }

    if settings.research != 0 {
        commands
            .entity(task_entity)
            .insert(Research(settings.research));
    } else {
        commands.entity(task_entity).try_remove::<Research>();
    }

    if let Some((follower, recruit_progress)) = &settings.recruit_progress {
        commands
            .entity(task_entity)
            .insert(Recruit(follower.clone(), *recruit_progress));
    } else {
        commands.entity(task_entity).try_remove::<Recruit>();
    }
}
