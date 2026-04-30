use std::collections::HashMap;

use bevy::{input_focus::InputFocus, prelude::*, ui::UiSystems, window::WindowResized};
use pyri_tooltip::{
    Tooltip, TooltipActivation, TooltipContent, TooltipDismissal, TooltipPlacement, TooltipTransfer,
};
use strum::IntoEnumIterator;

use crate::{
    common::{CultName, CultSymbol},
    constants::ui::*,
    funds::{Expense, ExpenseCategory, Funds, FundsAmount, Income, IncomeCategory},
    main_menu::NewGame,
    state::{GameState, MainSetupSet},
    suspicion::{IntelligenceSuspicion, ScientificSuspicion},
    text::TextKey,
    time::{CurrentGameSpeed, GameDate, GameSpeed, GameSpeedAction, GameSpeedChangedEvent},
    ui::{
        buttons::setup_observe_buttons,
        dialog::{Dialog, setup_observe_dialogs},
        main_menu::setup_main_menu,
        menu::setup_observe_menus,
    },
};

mod buttons;
mod dialog;
mod main_menu;
mod menu;
mod regions;
pub mod save_load;
mod scroll;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Load), setup_fonts)
        .init_resource::<UiScale>()
        .init_resource::<InputFocus>()
        .add_systems(
            OnExit(GameState::Load),
            (
                setup_observe_buttons,
                setup_observe_dialogs,
                setup_observe_menus,
            ),
        )
        .add_systems(Update, read_window_resized_messages)
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        .add_systems(
            OnEnter(GameState::Main),
            (setup_map, regions::setup, setup_intro)
                .chain()
                .in_set(MainSetupSet::Ui),
        )
        .add_systems(
            Update,
            regions::update_regional_suspicion.run_if(in_state(GameState::Main)),
        )
        .add_systems(
            Update,
            update_game_date
                .run_if(resource_exists_and_changed::<GameDate>.and(in_state(GameState::Main))),
        )
        .add_systems(
            Update,
            (update_funds_tooltip, update_funds)
                .run_if(resource_exists_and_changed::<Funds>.and(in_state(GameState::Main))),
        )
        .add_systems(
            Update,
            update_suspicion.run_if(
                (resource_exists_and_changed::<IntelligenceSuspicion>
                    .or(resource_exists_and_changed::<ScientificSuspicion>))
                .and(in_state(GameState::Main)),
            ),
        )
        .add_systems(
            Update,
            update_game_speed_state.run_if(
                resource_exists_and_changed::<CurrentGameSpeed>.and(in_state(GameState::Main)),
            ),
        )
        .add_systems(
            PostUpdate,
            update_meter_display::<u32>
                .run_if(in_state(GameState::Main))
                .before(UiSystems::Prepare),
        );
}

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
#[relationship(relationship_target = Views)]
struct ViewOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ViewOf, linked_spawn)]
struct Views(Vec<Entity>);

#[derive(Resource)]
pub struct FontHandle(pub Handle<Font>);

#[derive(Resource)]
pub struct DisplayFontHandle(pub Handle<Font>);

#[derive(Resource)]
pub struct UnicodeFontHandle(pub Handle<Font>);

#[derive(Component)]
struct MapUi;

#[derive(Component)]
struct GameDateUi;

#[derive(Component)]
struct FundsUi;

#[derive(Component)]
struct FundsTooltip;

#[derive(Component)]
#[require(Text, TextColor)]
struct MeterDisplay<T: PartialOrd + ToString + Send + Sync + 'static> {
    value: T,
    // positive | mixed
    low_threshold: T,
    // mixed | high_threshold
    high_threshold: T,
}

#[derive(Component)]
struct IntelligenceSuspicionUi;

#[derive(Component)]
struct ScientificSuspicionUi;

#[derive(Component)]
struct BaseUi;

#[derive(Component)]
struct FollowerList;

fn setup_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FontHandle(asset_server.load(FONT_PATH)));
    commands.insert_resource(DisplayFontHandle(asset_server.load(FONT_DISPLAY_PATH)));
    commands.insert_resource(UnicodeFontHandle(asset_server.load(UNICODE_FONT_PATH)));
}

fn read_window_resized_messages(
    mut reader: MessageReader<WindowResized>,
    mut ui_scale: ResMut<UiScale>,
) {
    if let Some(WindowResized { height, .. }) = reader.read().last() {
        info!("window resized; height: {height}");
        ui_scale.0 = height / 720.0;
    }
}

fn setup_map(
    mut commands: Commands,
    font_handle: Res<FontHandle>,
    unicode_font_handle: Res<UnicodeFontHandle>,
    asset_server: Res<AssetServer>,
    game_date: Res<GameDate>,
    cult_name: Res<CultName>,
    cult_symbol: Res<CultSymbol>,
) {
    let tooltip_content = commands
        .spawn((
            FundsTooltip,
            Node {
                flex_direction: FlexDirection::Column,
                border: UiRect::all(px(2)),
                padding: UiRect::all(px(3)),
                ..default()
            },
            BorderColor::all(BORDER_HIGHLIGHT),
            BackgroundColor::from(BUTTON_BACKGROUND),
            Visibility::Hidden,
            ZIndex(1),
        ))
        .id();

    let text_font = TextFont::from_font_size(SUB_HEADING).with_font(font_handle.0.clone());
    let unicode_text_font =
        TextFont::from_font_size(SUB_HEADING).with_font(unicode_font_handle.0.clone());

    commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            width: percent(100.0),
            height: percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
            // Top status bar
            parent
                .spawn((
                    Node {
                        width: percent(100.0),
                        border: UiRect::vertical(px(2)),
                        align_items: AlignItems::FlexEnd,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderColor::all(BORDER),
                    BackgroundColor::from(BUTTON_BACKGROUND),
                ))
                .with_children(|parent| {
                    // Cult symbol
                    parent.spawn((
                        Node {
                            margin: UiRect::right(px(5)),
                            ..default()
                        },
                        Text::new(cult_symbol.0),
                        TextColor::from(TEXT),
                        unicode_text_font.clone(),
                    ));
                    // Funds counter
                    parent.spawn((
                        Node {
                            min_width: px(75),
                            ..default()
                        },
                        text_font.clone(),
                        // will be updated by funds_changed
                        TextKey::new("funds-display").add_arg("funds", 0),
                        TextColor::from(TEXT),
                        FundsUi,
                        Tooltip {
                            content: TooltipContent::Custom(tooltip_content),
                            placement: TooltipPlacement::CURSOR,
                            activation: TooltipActivation::default(),
                            dismissal: TooltipDismissal {
                                // Not sure what units these are
                                on_distance: 400.0,
                                on_click: false,
                            },
                            transfer: TooltipTransfer::default(),
                        },
                    ));
                    // Game date display
                    parent.spawn((
                        Node {
                            min_width: px(125),
                            ..default()
                        },
                        text_font.clone(),
                        TextColor::from(TEXT),
                        // will be updated by update_game_date
                        TextKey::new("game-date-display").add_arg("date", game_date.0),
                        GameDateUi,
                    ));
                    // Suspicion meters
                    parent.spawn((
                        Node {
                            min_width: px(50),
                            ..default()
                        },
                        text_font.clone(),
                        TextLayout::new_with_justify(Justify::Right),
                        MeterDisplay::<u32> {
                            value: 0,
                            low_threshold: 34,
                            high_threshold: 67,
                        },
                        IntelligenceSuspicionUi,
                    ));
                    parent.spawn((
                        Node {
                            min_width: px(50),
                            ..default()
                        },
                        text_font.clone(),
                        TextLayout::new_with_justify(Justify::Right),
                        MeterDisplay::<u32> {
                            value: 0,
                            low_threshold: 34,
                            high_threshold: 67,
                        },
                        ScientificSuspicionUi,
                    ));
                    // Separate left-aligned and right-aligned status fields
                    parent.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    });
                    parent.spawn((
                        Node {
                            margin: UiRect::right(px(10)),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        Text::new(&cult_name.0),
                        TextColor::from(TEXT),
                        TextFont::from_font_size(SMALL).with_font(font_handle.0.clone()),
                    ));
                    parent
                        .spawn((
                            Button,
                            GameSpeedAction::TogglePause,
                            Node {
                                width: px(25),
                                ..default()
                            },
                            ImageNode {
                                image: asset_server.load(ICON_PAUSE),
                                color: TEXT.into(),
                                ..default()
                            },
                        ))
                        .observe(on_game_speed_clicked);
                    parent
                        .spawn((
                            Button,
                            GameSpeedAction::SetSpeed(GameSpeed::Normal),
                            Node {
                                width: px(25),
                                ..default()
                            },
                            // RIGHTWARDS ARROW
                            Text("\u{2192}".to_string()),
                            TextColor::from(TEXT_HIGHLIGHT),
                            unicode_text_font.clone(),
                            TextLayout::new_with_justify(Justify::Center),
                        ))
                        .observe(on_game_speed_clicked);
                    parent
                        .spawn((
                            Button,
                            GameSpeedAction::SetSpeed(GameSpeed::Fast),
                            Node {
                                width: px(25),
                                ..default()
                            },
                            // RIGHTWARDS PAIRED ARROWS
                            Text("\u{21C9}".to_string()),
                            TextColor::from(TEXT),
                            unicode_text_font.clone(),
                            TextLayout::new_with_justify(Justify::Center),
                        ))
                        .observe(on_game_speed_clicked);
                    parent
                        .spawn((
                            Button,
                            GameSpeedAction::SetSpeed(GameSpeed::Faster),
                            Node {
                                width: px(25),
                                ..default()
                            },
                            // THREE RIGHTWARDS ARROWS
                            Text("\u{21F6}".to_string()),
                            TextColor::from(TEXT),
                            unicode_text_font.clone(),
                            TextLayout::new_with_justify(Justify::Center),
                        ))
                        .observe(on_game_speed_clicked);
                });
            parent.spawn((
                ImageNode {
                    image: asset_server.load(TEXTURE_EARTH_BACKGROUND),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Node {
                    width: percent(100.0),
                    flex_grow: 1.0,
                    ..default()
                },
                MapUi,
            ));
        });
}

fn update_game_date(
    game_date: Res<GameDate>,
    mut text_key: Single<&mut TextKey, With<GameDateUi>>,
) {
    text_key.replace_arg("date", game_date.0);
}

fn update_funds(funds: Res<Funds>, mut text_key: Single<&mut TextKey, With<FundsUi>>) {
    text_key.replace_arg("funds", funds.0);
}

fn update_suspicion(
    intel_suspicion: Res<IntelligenceSuspicion>,
    scien_suspicion: Res<ScientificSuspicion>,
    mut intel_suspicion_ui: Single<
        &mut MeterDisplay<u32>,
        (
            With<IntelligenceSuspicionUi>,
            Without<ScientificSuspicionUi>,
        ),
    >,
    mut scien_suspicion_ui: Single<
        &mut MeterDisplay<u32>,
        (
            With<ScientificSuspicionUi>,
            Without<IntelligenceSuspicionUi>,
        ),
    >,
) {
    intel_suspicion_ui.value = intel_suspicion.0;
    scien_suspicion_ui.value = scien_suspicion.0;
}

fn update_meter_display<T: PartialOrd + ToString + Send + Sync + 'static>(
    mut meters: Query<(&mut Text, &mut TextColor, &MeterDisplay<T>), Changed<MeterDisplay<T>>>,
) {
    for (mut text, mut text_color, meter) in meters.iter_mut() {
        text.0 = meter.value.to_string();

        if meter.low_threshold < meter.high_threshold {
            // POS | MIX | NEG
            *text_color = if meter.value < meter.low_threshold {
                TEXT_POSITIVE
            } else if meter.value >= meter.low_threshold && meter.value < meter.high_threshold {
                TEXT_MIXED
            } else {
                TEXT_NEGATIVE
            }
            .into();
        } else {
            // NEG | MIX | POS
            *text_color = if meter.value < meter.high_threshold {
                TEXT_POSITIVE
            } else if meter.value >= meter.high_threshold && meter.value < meter.low_threshold {
                TEXT_MIXED
            } else {
                TEXT_NEGATIVE
            }
            .into();
        }
    }
}

fn on_game_speed_clicked(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    game_speed_actions: Query<&GameSpeedAction>,
) {
    if click.button == PointerButton::Primary {
        let game_speed_action = *game_speed_actions.get(click.entity).unwrap();
        commands.trigger(GameSpeedChangedEvent(game_speed_action));
    }
}

fn on_label_over(
    event: On<Pointer<Over>>,
    mut label_colors: Query<(&mut BackgroundColor, &mut BorderColor)>,
) {
    let (mut background_color, mut border_color) = label_colors.get_mut(event.entity).unwrap();
    border_color.set_all(BORDER_HIGHLIGHT);
    background_color.0.set_alpha(1.0);
}

fn on_label_out(
    event: On<Pointer<Out>>,
    mut label_colors: Query<(&mut BackgroundColor, &mut BorderColor)>,
) {
    let (mut background_color, mut border_color) = label_colors.get_mut(event.entity).unwrap();
    border_color.set_all(BORDER);
    background_color.0.set_alpha(0.75);
}

fn update_game_speed_state(
    current_game_speed: Res<CurrentGameSpeed>,
    mut game_speed_buttons: Query<(
        Option<&mut TextColor>,
        Option<&mut ImageNode>,
        &GameSpeedAction,
    )>,
) {
    for (mut text_color, mut image, &speed_action) in game_speed_buttons.iter_mut() {
        let is_active = speed_action == GameSpeedAction::TogglePause && current_game_speed.paused
            || speed_action == GameSpeedAction::SetSpeed(current_game_speed.speed)
                && !current_game_speed.paused;
        if is_active {
            if let Some(text_color) = text_color.as_mut() {
                **text_color = TEXT_HIGHLIGHT.into();
            }
            if let Some(image) = image.as_mut() {
                image.color = TEXT_HIGHLIGHT.into();
            }
        } else {
            if let Some(text_color) = text_color.as_mut() {
                **text_color = TEXT.into();
            }
            if let Some(image) = image.as_mut() {
                image.color = TEXT.into();
            }
        }
    }
}

fn update_funds_tooltip(
    mut commands: Commands,
    incomes: Query<&Income>,
    expenses: Query<&Expense>,
    tooltip: Single<Entity, With<FundsTooltip>>,
    font_handle: Res<FontHandle>,
) {
    fn income_expense_row(
        mut commands: Commands,
        parent: Entity,
        text_font: &TextFont,
        category: String,
        count: usize,
        funds: FundsAmount,
    ) {
        commands
            .spawn((
                // Node to represent the row
                Node::default(),
                ChildOf(parent),
            ))
            .with_children(|parent| {
                parent.spawn((Text::new(format!("{count}x ")), text_font.clone()));
                parent.spawn((TextKey::new(category), text_font.clone()));
                parent.spawn(Node {
                    flex_grow: 1.0,
                    padding: UiRect::left(px(5)),
                    ..default()
                });
                parent.spawn((
                    TextKey::new("funds").add_arg("funds", funds),
                    text_font.clone(),
                ));
            });
    }

    let tooltip = tooltip.entity();
    commands.entity(tooltip).despawn_children();

    // Completely refresh the tooltip contents
    let text_font = TextFont::from_font_size(NORMAL).with_font(font_handle.0.clone());
    let hrule = (
        Node {
            min_width: percent(80),
            min_height: px(1),
            margin: UiRect::vertical(px(5)),
            ..default()
        },
        BackgroundColor::from(YELLOW),
        ChildOf(tooltip),
    );
    commands.spawn((
        TextKey::new("income-tooltip-header"),
        text_font.clone(),
        ChildOf(tooltip),
    ));
    commands.spawn(hrule.clone());

    let mut income_ledger: HashMap<IncomeCategory, (FundsAmount, usize)> = HashMap::default();
    for Income(amount, category) in incomes {
        let (funds, count) = income_ledger.entry(*category).or_default();
        *funds += amount;
        *count += 1
    }
    for category in IncomeCategory::iter() {
        if let Some((funds, count)) = income_ledger.get(&category) {
            let category: &str = category.into();
            let category = format!("income-category-{category}");
            income_expense_row(
                commands.reborrow(),
                tooltip,
                &text_font,
                category,
                *count,
                *funds,
            );
        }
    }
    commands.spawn((
        TextKey::new("expense-tooltip-header"),
        text_font.clone(),
        ChildOf(tooltip),
    ));
    commands.spawn(hrule);
    let mut expense_ledger: HashMap<ExpenseCategory, (FundsAmount, usize)> = HashMap::default();
    for Expense(amount, category) in expenses {
        let (funds, count) = expense_ledger.entry(*category).or_default();
        *funds += amount;
        *count += 1
    }
    for category in ExpenseCategory::iter() {
        if let Some((funds, count)) = expense_ledger.get(&category) {
            let category: &str = category.into();
            let category = format!("expense-category-{category}");
            income_expense_row(
                commands.reborrow(),
                tooltip,
                &text_font,
                category,
                *count,
                *funds,
            );
        }
    }
}

fn setup_intro(mut commands: Commands, new_game: Option<Res<NewGame>>) {
    if new_game.is_some() {
        commands.spawn(
            Dialog::new()
                .with_pause()
                .with_text_body("new-game-intro-body")
                .with_confirm_label("new-game-confirm"),
        );
    }
}
