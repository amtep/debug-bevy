use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use rand::{RngExt, seq::IndexedRandom};
use serde_derive::Deserialize;

use crate::{
    followers::{Follower, FollowerCount, FollowersAsset, FollowersHandle},
    funds::{Expense, Funds, FundsAmount},
    new_game::NewGame,
    regions::{BasePlot, Region},
    rng::RandomSource,
    state::{GameState, MainSetupSet},
    suspicion::{MediaSuspicionChange, PoliceSuspicionChange},
    tasks::{RecruitMinionProgress, Task, TasksAsset, TasksHandle},
};

const BASETYPES_ASSET_PATH: &str = "data/define.basetypes.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<BasetypesAsset>::new(&["basetypes.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
        .add_systems(
            OnEnter(GameState::Main),
            new_game
                .run_if(resource_exists::<NewGame>)
                .in_set(MainSetupSet::Bases),
        );
}

#[derive(Deserialize, Asset, TypePath)]
pub struct BasetypesAsset(pub IndexMap<String, BasetypeSettings>);

#[derive(Resource)]
pub struct BasetypesHandle(pub Handle<BasetypesAsset>);

#[derive(Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "kebab-case")]
pub struct BasetypeSettings {
    pub max_pop: usize,
    pub cost_per_day: FundsAmount,
    pub initial_cost: FundsAmount,
    pub police_suspicion: f32,
    pub media_suspicion: f32,
    #[serde(default)]
    pub regions: Vec<String>,
    pub requires_discovery: Option<String>,
    pub color: String,
}

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BasetypesHandle(asset_server.load(BASETYPES_ASSET_PATH)));
}

/// Basetype name
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Save, RecruitMinionProgress)]
pub struct Base(pub String);

fn new_game(
    mut commands: Commands,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    mut random_source: ResMut<RandomSource>,
    base_plots: Query<Entity, With<BasePlot>>,
) {
    info!("Creating starting base");
    let i = random_source.0.random_range(0..base_plots.count());
    let base_plot = base_plots.iter().nth(i).unwrap();

    let (base_type, _) = base_types_asset
        .get(base_types_handle.0.id())
        .unwrap()
        .0
        .first()
        .unwrap();

    commands.run_system_cached_with(spawn_base_inner, (base_plot, base_type.clone(), true));
}

fn spawn_base_inner(
    In((base_plot, base_type, free)): In<(Entity, String, bool)>,
    mut commands: Commands,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    followers_handle: Res<FollowersHandle>,
    followers_asset: Res<Assets<FollowersAsset>>,
    task_handle: Res<TasksHandle>,
    task_assets: Res<Assets<TasksAsset>>,
    mut funds: ResMut<Funds>,
) {
    let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;
    let base_type_settings = base_types.get(&base_type).unwrap();
    let mut entity_command = commands.spawn((
        Base(base_type),
        Expense(base_type_settings.cost_per_day, "base".into(), 1),
        ChildOf(base_plot),
    ));
    if base_type_settings.media_suspicion != 0.0 {
        entity_command.insert(MediaSuspicionChange(base_type_settings.media_suspicion));
    }
    if base_type_settings.police_suspicion != 0.0 {
        entity_command.insert(PoliceSuspicionChange(base_type_settings.police_suspicion));
    }
    let base = entity_command.id();

    if !free {
        funds.0 -= base_type_settings.initial_cost;
    }

    for (follower, settings) in &followers_asset.get(followers_handle.0.id()).unwrap().0 {
        let follower_entity = commands
            .spawn((
                ChildOf(base),
                Follower(follower.clone()),
                Expense(settings.cost_per_day, follower.clone(), 0),
            ))
            .insert(FollowerCount(0))
            .id();
        let task = task_assets
            .get(task_handle.0.id())
            .unwrap()
            .0
            .iter()
            .find(|(_, settings)| settings.follower_types.contains(follower))
            .unwrap()
            .0;
        commands.spawn((Task(task.clone()), ChildOf(follower_entity)));
    }
}

pub fn spawn_base(
    In((region, base_type)): In<(Entity, String)>,
    mut commands: Commands,
    funds: ResMut<Funds>,
    regions: Query<&Children, With<Region>>,
    base_plots: Query<Has<Children>, With<BasePlot>>,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    mut random_source: ResMut<RandomSource>,
) {
    let vacant_base_plots: Vec<Entity> = regions
        .get(region)
        .unwrap()
        .iter()
        .filter(|base_plot| base_plots.get(*base_plot) == Ok(false))
        .collect();
    let Some(base_plot) = vacant_base_plots.choose(&mut random_source.0) else {
        warn!("expected at least one vacant base plot");
        return;
    };

    let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;
    let base_type_settings = base_types.get(&base_type).unwrap();

    if funds.0 < base_type_settings.initial_cost {
        warn!(
            "not enough funds to acquire {base_type} base ({} < {})",
            funds.0, base_type_settings.initial_cost
        );
        return;
    }

    commands.run_system_cached_with(spawn_base_inner, (*base_plot, base_type, false));
}
