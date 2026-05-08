use bevy::prelude::*;
use moonshine_save::save::Save;
use serde::Deserialize;
use strum::{EnumIter, IntoStaticStr};

use crate::{
    constants::STARTING_FUNDS,
    main_menu::NewGame,
    state::{GameState, MainSetupSet},
    time::GameDate,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Main),
        setup_funds
            .run_if(resource_exists::<NewGame>)
            .in_set(MainSetupSet::Default),
    )
    .add_systems(
        FixedUpdate,
        update_funds.run_if(resource_exists_and_changed::<GameDate>.and(in_state(GameState::Main))),
    );
}

pub type FundsAmount = i64;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Funds(pub FundsAmount);

/// The third field is the number of budget entries represented by this component,
/// to be shown in the funds tooltip.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Save)]
#[component(immutable)]
pub struct Expense(pub FundsAmount, pub ExpenseCategory, pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, IntoStaticStr, Reflect)]
#[strum(serialize_all = "lowercase")]
pub enum ExpenseCategory {
    Followers,
    Bases,
}

/// The third field is the number of budget entries represented by this component,
/// to be shown in the funds tooltip.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Save)]
#[component(immutable)]
pub struct Income(pub FundsAmount, pub IncomeCategory, pub usize);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, IntoStaticStr, Reflect, Deserialize,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum IncomeCategory {
    Jobs,
    Crime,
}

#[derive(Debug, Event, Clone, Copy)]
pub struct IncomeExpenseUpdatedEvent;

fn setup_funds(mut commands: Commands) {
    commands.insert_resource(Funds(STARTING_FUNDS));

    commands.add_observer(|_: On<Insert, Income>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    });
    commands.add_observer(|_: On<Remove, Income>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    });
    commands.add_observer(|_: On<Insert, Expense>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    });
    commands.add_observer(|_: On<Remove, Expense>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    });
}

fn update_funds(mut funds: ResMut<Funds>, incomes: Query<&Income>, expenses: Query<&Expense>) {
    for Income(amount, _, count) in incomes {
        funds.0 += amount * (*count as FundsAmount);
    }
    for Expense(amount, _, count) in expenses {
        funds.0 -= amount * (*count as FundsAmount);
    }
}
