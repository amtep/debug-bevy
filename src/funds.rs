use bevy::prelude::*;
use moonshine_save::save::Save;

use crate::{
    modifiers::{ExpenseModifier, IncomeModifier, Modifier},
    new_game::NewGame,
    state::{GameState, MainSetupSet},
    time::GameDate,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Main),
        new_game
            .run_if(resource_exists::<NewGame>)
            .in_set(MainSetupSet::Default),
    )
    .add_systems(
        FixedUpdate,
        update_funds.run_if(resource_exists_and_changed::<GameDate>.and(in_state(GameState::Main))),
    )
    .add_systems(
        OnEnter(GameState::Main),
        setup_main.in_set(MainSetupSet::Late),
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
    })
    .add_observer(|_: On<Insert, IncomeModifier>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    })
    .add_observer(|_: On<Remove, IncomeModifier>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    })
    .add_observer(|_: On<Insert, ExpenseModifier>, mut commands: Commands| {
        commands.trigger(IncomeExpenseUpdatedEvent);
    })
    .add_observer(|_: On<Remove, ExpenseModifier>, mut commands: Commands| {
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

fn new_game(mut commands: Commands, new_game: Res<NewGame>) {
    commands.insert_resource(Funds(new_game.difficulty.starting_funds));
}

fn setup_main(mut commands: Commands) {
    commands.init_resource::<TotalIncome>();
    commands.init_resource::<TotalExpense>();
    // do NOT trigger on load with lots of insertion.
    commands
        .add_observer(on_income_expense_updated)
        .insert(DespawnOnExit(GameState::Main));
    commands.trigger(IncomeExpenseUpdatedEvent);
}

#[derive(Resource, Default, PartialEq, Eq)]
pub struct TotalIncome(pub FundsAmount);

#[derive(Resource, Default, PartialEq, Eq)]
pub struct TotalExpense(pub FundsAmount);

#[allow(clippy::cast_possible_truncation)]
fn on_income_expense_updated(
    _: On<IncomeExpenseUpdatedEvent>,
    mut total_income: ResMut<TotalIncome>,
    mut total_expense: ResMut<TotalExpense>,
    incomes: Query<(Entity, &Income)>,
    expenses: Query<(Entity, &Expense)>,
    m_i: Modifier<IncomeModifier>,
    m_e: Modifier<ExpenseModifier>,
) {
    let mut total_income_temp = 0;
    let mut total_expense_temp = 0;

    for (entity, Income(amount, _, count)) in &incomes {
        let income = (amount * (*count as FundsAmount)) as f64;
        total_income_temp += m_i.calc(income, entity) as FundsAmount;
    }
    for (entity, Expense(amount, _, count)) in &expenses {
        let expense = (amount * (*count as FundsAmount)) as f64;
        total_expense_temp += m_e.calc(expense, entity) as FundsAmount;
    }

    total_income.set_if_neq(TotalIncome(total_income_temp));
    total_expense.set_if_neq(TotalExpense(total_expense_temp));
}

fn update_funds(
    mut funds: ResMut<Funds>,
    total_income: Res<TotalIncome>,
    total_expense: Res<TotalExpense>,
) {
    funds.0 += total_income.0 - total_expense.0;
}
