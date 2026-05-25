use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use rand::seq::IndexedRandom;
use serde_derive::Deserialize;

use crate::{
    followers::{Follower, FollowerCount, FollowerEffects, FollowersAsset, FollowersHandle},
    funds::{Expense, Funds, FundsAmount},
    new_game::NewGame,
    regions::{BasePlot, Region},
    rng::RandomSource,
    state::{GameState, MainSetupSet},
    suspicion::{SuspicionType, add_suspicion_change},
    tasks::{Task, TaskEffects, TasksAsset, TasksHandle},
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
    pub max_follower_count: usize,
    pub cost_per_day: FundsAmount,
    pub initial_cost: FundsAmount,
    #[serde(default)]
    pub suspicions: IndexMap<SuspicionType, f32>,
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
#[require(Save)]
pub struct Base(pub String);

fn new_game(
    mut commands: Commands,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    random_source: Res<RandomSource>,
    new_game: Res<NewGame>,
    regions: Query<(&Children, &Region)>,
    base_plots: Query<(), With<BasePlot>>,
) {
    info!("Creating starting base");
    let region = &new_game.region;
    let base_plots: Vec<Entity> = regions
        .iter()
        .find(|(_, r)| r.name == region.name)
        .unwrap()
        .0
        .iter()
        .filter(|c| base_plots.contains(*c))
        .collect();

    let base_plot = *base_plots.choose(&mut random_source.rng()).unwrap();

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
    let base_entity = commands
        .spawn((
            Base(base_type),
            Expense(base_type_settings.cost_per_day, "base".into(), 1),
            ChildOf(base_plot),
        ))
        .id();

    for (suspicion, amount) in &base_type_settings.suspicions {
        add_suspicion_change(&mut commands.entity(base_entity), *suspicion, *amount);
    }

    if !free {
        funds.0 -= base_type_settings.initial_cost;
    }

    for (follower, _) in &followers_asset.get(followers_handle.0.id()).unwrap().0 {
        let task = task_assets
            .get(task_handle.0.id())
            .unwrap()
            .0
            .iter()
            .find(|(_, settings)| settings.follower_types.contains(follower))
            .unwrap()
            .0;
        commands
            .spawn((
                ChildOf(base_entity),
                Follower(follower.clone()),
                Task(task.clone()),
            ))
            .insert(FollowerCount(0))
            .with_child(FollowerEffects)
            .with_child(TaskEffects);
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
    random_source: Res<RandomSource>,
) {
    let vacant_base_plots: Vec<Entity> = regions
        .get(region)
        .unwrap()
        .iter()
        .filter(|base_plot| base_plots.get(*base_plot) == Ok(false))
        .collect();
    let Some(base_plot) = vacant_base_plots.choose(&mut random_source.rng()) else {
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

#[allow(clippy::cast_possible_truncation)]
pub fn transfer_follower_costs(
    src_entity: Entity,
    dst_entity: Entity,
    _follower: String,
    count: usize,
    parents: Query<&ChildOf>,
) -> (FundsAmount, u32) {
    const INTRA_REGION_FUNDS_COST: FundsAmount = -1000;
    const INTER_REGION_FUNDS_COST: FundsAmount = -5000;
    // TODO: custom costs based on follower
    const INTEL_SUSPICION_COST: u32 = 5;

    // same region entity
    if parents.root_ancestor(src_entity) == parents.root_ancestor(dst_entity) {
        (
            INTRA_REGION_FUNDS_COST * count as FundsAmount,
            INTEL_SUSPICION_COST * count as u32,
        )
    } else {
        (
            INTER_REGION_FUNDS_COST * count as FundsAmount,
            INTEL_SUSPICION_COST * count as u32,
        )
    }
}

pub fn transfer_followers(
    In((dst_entity, src_follower_entity, follower, count)): In<(Entity, Entity, String, usize)>,
    mut commands: Commands,
    bases: Query<&Children, With<Base>>,
    followers: Query<(&Follower, &FollowerCount)>,
) {
    let mut follower_count = *followers.get(src_follower_entity).unwrap().1;
    follower_count.0 -= count;
    commands.entity(src_follower_entity).insert(follower_count);

    let children = bases.get(dst_entity).unwrap();
    let (mut dst_follower_count, dst_follower_entity) = children
        .iter()
        .find_map(|child| {
            followers
                .get(child)
                .ok()
                .and_then(|(f, c)| (f.0 == follower).then_some((*c, child)))
        })
        .unwrap();
    dst_follower_count.0 += count;
    commands
        .entity(dst_follower_entity)
        .insert(dst_follower_count);
}
