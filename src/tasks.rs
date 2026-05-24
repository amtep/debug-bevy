use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use serde_derive::Deserialize;

use crate::{
    effects::{Effect, apply_effect},
    followers::{FollowerCount, RecruitProgress},
    modifiers::Source,
    state::GameState,
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

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TaskSettings {
    pub follower_types: Vec<String>,
    pub requires_discovery: Option<String>,
    pub effects: Vec<Effect>,
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
    task_handle: Res<TasksHandle>,
    task_assets: Res<Assets<TasksAsset>>,
    followers: Query<(&FollowerCount, &Task)>,
) {
    let Ok((FollowerCount(count), Task(task))) = followers.get(insert.entity) else {
        return;
    };
    let task_settings = &task_assets.get(task_handle.0.id()).unwrap().0;

    let Some(settings) = task_settings.get(task) else {
        error!("Task {task} not known");
        return;
    };

    commands.entity(insert.entity).despawn_children();

    for effect in &settings.effects {
        // FIXME: any modifier applied is currently acting on the task, but not the base on a whole.
        commands.run_system_cached_with(
            apply_effect,
            (
                Some(insert.entity),
                Some(*count),
                effect.clone(),
                Source::Task(task.clone()),
            ),
        );
    }
}
