use std::collections::HashMap;

use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use serde::Deserialize;

use crate::{
    achievements::AchievedEvent,
    bases::{Base, BasetypesAsset, BasetypesHandle},
    funds::{Expense, FundsAmount},
    modifiers::{Modifier, RecruitmentBy, RecruitmentByOf, RecruitmentOf},
    new_game::NewGame,
    state::{GameState, MainSetupSet},
};

const FOLLOWERS_ASSET_PATH: &str = "data/define.followers.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<FollowersAsset>::new(&["followers.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
        .add_systems(
            OnEnter(GameState::Main),
            new_game
                .run_if(resource_exists::<NewGame>)
                .in_set(MainSetupSet::Followers),
        )
        .add_systems(FixedUpdate, recruit.run_if(in_state(GameState::Main)));
}

#[derive(Deserialize, Asset, TypePath)]
pub struct FollowersAsset(pub IndexMap<String, FollowerSettings>);

#[derive(Resource)]
pub struct FollowersHandle(pub Handle<FollowersAsset>);

/// These are the general settings for all follower types.
/// Once there are also specific follower settings, there
/// will need to be an enum to distinguish them.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct FollowerSettings {
    pub cost_per_day: FundsAmount,
    pub symbol: char,
    pub first_recruit_achievement: Option<String>,
}

#[derive(Component, Reflect, Clone, Deserialize, Debug, Deref, PartialEq, Eq)]
#[reflect(Component)]
#[require(Save)]
pub struct Follower(pub String);

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, DerefMut, Reflect,
)]
#[reflect(Component)]
#[component(immutable)]
pub struct FollowerCount(pub usize);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Recruit(pub String, pub f32);

/// A component to track recruitment of new minions.
/// A new minion is spawned if this rolls over the [`NEW_MINION_PROGRESS`] constant.
/// Progress does not advance if the base is full.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RecruitProgress(HashMap<String, f32>);

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FollowersHandle(asset_server.load(FOLLOWERS_ASSET_PATH)));
}

/// Create the starting priest for the cult.
fn new_game(
    mut commands: Commands,
    base: Single<&Children, With<Base>>,
    mut followers: Query<(&Follower, &FollowerCount, &Expense)>,
    new_game: Res<NewGame>,
) {
    info!("Creating starting follower");

    let starting_followers = &new_game.difficulty.starting_followers;

    for child in base.iter() {
        if let Ok((follower, follower_count, expense)) = followers.get_mut(child)
            && let Some(count) = starting_followers.get(&follower.0)
        {
            let mut follower_count = *follower_count;
            follower_count.0 = *count;
            let mut expense = expense.clone();
            expense.2 = *count;
            commands.entity(child).insert(follower_count);
            commands.entity(child).insert(expense);
        }
    }
}

const RECRUIT_PROGRESS: f32 = 100.0;

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::suspicious_operation_groupings)]
fn recruit(
    mut commands: Commands,
    bases: Query<(&Base, &Children)>,
    followers: Query<(&ChildOf, &Follower, &FollowerCount)>,
    mut recruits: Query<(Entity, &ChildOf, &Recruit, &mut RecruitProgress)>,
    m_by: Modifier<RecruitmentBy>,
    m_of: Modifier<RecruitmentOf>,
    m_by_of: Modifier<RecruitmentByOf>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    base_types_handle: Res<BasetypesHandle>,
    followers_asset: Res<Assets<FollowersAsset>>,
    followers_handle: Res<FollowersHandle>,
) {
    let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;
    let followers_types = &followers_asset.get(followers_handle.0.id()).unwrap().0;

    for (entity, follower_entity, recruit, mut recruit_progress) in &mut recruits {
        let (ChildOf(base_entity), follower, FollowerCount(follower_count)) =
            followers.get(follower_entity.0).unwrap();
        let (base, children) = bases.get(*base_entity).unwrap();
        let max_follower_count = base_types.get(&base.0).unwrap().max_follower_count;

        let total_followers = children
            .iter()
            .filter_map(|c| followers.get(c).ok().map(|(_, _, c)| c.0))
            .sum::<usize>();

        // TODO: automatically switch to the default task if the base is full
        if total_followers >= max_follower_count {
            continue;
        }

        let mut base = recruit.1 as f64;
        base = m_by.calc_with(base, entity, |f| f.0 == follower.0);
        base = m_of.calc_with(base, entity, |f| f.0 == recruit.0);
        base = m_by_of.calc_with(base, entity, |f| f.0 == follower.0 && f.1 == recruit.0);

        let recruit_progress = recruit_progress.0.entry(recruit.0.clone()).or_default();
        *recruit_progress += (base as f32) * (*follower_count as f32);

        let additional_followers = ((*recruit_progress / RECRUIT_PROGRESS) as usize)
            .min(max_follower_count - total_followers);
        *recruit_progress -= (additional_followers as f32) * RECRUIT_PROGRESS;

        if additional_followers != 0 {
            let (follower_entity, mut follower_count) = children
                .iter()
                .filter_map(|c| followers.get(c).ok().map(|f| (c, f)))
                .find(|(_, (_, f, _))| f.0 == recruit.0)
                .map(|(e, (_, _, c))| (e, *c))
                .unwrap();
            follower_count.0 += additional_followers;
            commands.entity(follower_entity).insert(follower_count);
            if let Some(achievement) = followers_types
                .get(&recruit.0)
                .and_then(|f| f.first_recruit_achievement.as_ref())
            {
                commands.trigger(AchievedEvent {
                    achievement: achievement.clone(),
                });
            }
        }
    }
}
