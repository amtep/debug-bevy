use bevy::prelude::*;
use chrono::NaiveDate;
use serde::Deserialize;

use crate::{
    funds::{Expense, FundsAmount, Income},
    modifiers::ModifierValue,
    suspicion::SuspicionType,
};

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CultName(pub String);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CultSymbol(pub usize);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Difficulty(pub String);

#[derive(Resource, Default)]
pub struct Dev;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Unlocked;

#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Component)]
#[reflect(opaque)]
pub struct EndDate(pub NaiveDate);

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Effect {
    Funds(FundsAmount),
    Income {
        amount: Income,
        duration: Option<u32>,
    },
    Expense {
        amount: Expense,
        duration: Option<u32>,
    },
    Secrets(i32),
    Discovery(String),
    SpawnBase(String),
    DestroyBase,
    Suspicion {
        suspicion: SuspicionType,
        amount: i32,
    },
    SuspicionChange {
        suspicion: SuspicionType,
        amount: f32,
        duration: Option<u32>,
    },
    Follower {
        name: String,
        count: isize,
    },
    FollowerBusy {
        name: String,
        count: isize,
        duration: u32,
    },
    Modifier(ModifierValue),
}
