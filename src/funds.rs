use bevy::prelude::*;
use moonshine_save::save::Save;

use crate::{
    modifiers::{GlobalExpenseModifier, GlobalIncomeModifier, Modifier},
    new_game::NewGame,
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
    )
    .add_observer(|_: On<Insert, Income>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    })
    .add_observer(|_: On<Remove, Income>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    })
    .add_observer(|_: On<Insert, Expense>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    })
    .add_observer(|_: On<Remove, Expense>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    });
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
pub struct Expense(pub FundsAmount, pub String, pub usize);

/// The third field is the number of budget entries represented by this component,
/// to be shown in the funds tooltip.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Save)]
#[component(immutable)]
pub struct Income(pub FundsAmount, pub String, pub usize);

#[derive(Debug, Event, Clone, Copy)]
pub struct IncomeExpenseUpdatedEvent;

fn setup_funds(mut commands: Commands, new_game: Res<NewGame>) {
    commands.insert_resource(Funds(new_game.difficulty.starting_funds));
}

#[expect(clippy::cast_possible_truncation, reason = "funds won't go that high")]
fn update_funds(
    mut funds: ResMut<Funds>,
    incomes: Query<&Income>,
    expenses: Query<&Expense>,
    m_i: Modifier<GlobalIncomeModifier>,
    m_e: Modifier<GlobalExpenseModifier>,
) {
    let mut income = 0;
    let mut expense = 0;

    for Income(amount, _, count) in incomes {
        income += amount * (*count as FundsAmount);
    }
    for Expense(amount, _, count) in expenses {
        expense += amount * (*count as FundsAmount);
    }

    funds.0 += (m_i.calc(income as f64) - m_e.calc(expense as f64)).round() as i64;
}
