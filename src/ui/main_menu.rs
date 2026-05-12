use bevy::prelude::*;
use bevy_ui_text_input::{TextInputBuffer, TextInputMode, TextInputNode, TextInputPrompt};

use crate::{
    common::{CultName, CultSymbol},
    constants::{
        files::{CULT_SYMBOL_PATH, CULT_SYMBOLS, TEXTURE_EARTH_BACKGROUND},
        ui::{colors::*, fonts::*},
    },
    new_game::{DifficultiesAsset, DifficultiesHandle, NewGame},
    save_load::any_save_file_exists,
    state::GameState,
    text::TextKey,
    ui::{
        DisplayFontHandle, FontHandle,
        dialog::{Dialog, DialogCancelled, DialogConfirm, DialogConfirmed},
        save_load::{load_most_recent_game, open_load_game_popup},
    },
};

#[derive(Component)]
struct CultSym(usize);

#[derive(Event)]
struct CultSymbolChanged(usize);

#[derive(Resource)]
struct DifficultySelected(String);

#[derive(Component)]
struct Difficulty(String);

#[derive(Event)]
struct DifficultyChanged(String);

pub fn setup_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    font_handle: Res<FontHandle>,
    display_font_handle: Res<DisplayFontHandle>,
) {
    let button = |key| {
        (
            Button,
            Node {
                width: percent(100),
                padding: UiRect::axes(px(30), px(15)),
                border: UiRect::all(px(4)),
                border_radius: BorderRadius::all(px(20)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderColor::all(WHITE),
            BackgroundColor::from(BUTTON_BACKGROUND.with_alpha(OVERLAY_ALPHA)),
            children![(
                TextFont::from_font_size(40.0).with_font(font_handle.clone()),
                TextKey::new(key),
            )],
        )
    };

    commands
        .spawn((
            DespawnOnExit(GameState::MainMenu),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                flex_direction: FlexDirection::Column,
                row_gap: px(20),
                ..default()
            },
            ImageNode {
                image: asset_server.load(TEXTURE_EARTH_BACKGROUND),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: percent(100),
                    height: px(200),
                    padding: UiRect::all(px(10)),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(
                    TextKey::new("main-menu-title"),
                    TextFont::from_font_size(150.0).with_font(display_font_handle.clone()),
                    TextShadow::default(),
                )],
            ));
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Center,
                    row_gap: px(20),
                    ..default()
                })
                .with_children(|parent| {
                    let any_save = any_save_file_exists();
                    if any_save {
                        parent.spawn(button("main-menu-button-continue-game")).observe(
                            |click: On<Pointer<Click>>, mut commands: Commands, next_state: ResMut<NextState<GameState>>| {
                                if click.button == PointerButton::Primary {
                                    load_most_recent_game(commands.reborrow(), next_state);
                                }
                            },
                        );
                    }

                    parent.spawn(button("main-menu-button-new-game")).observe(
                        move |click: On<Pointer<Click>>,
                         mut commands: Commands,
                         asset_server: Res<AssetServer>,
                         font_handle: Res<FontHandle>| {
                            if click.button == PointerButton::Primary {
                                let cult_symbol_observer = commands
                                    .add_observer(
                                        |event: On<CultSymbolChanged>, mut cult_symbols: Query<(&mut ImageNode, &CultSym)>| {
                                            for (mut image_node, sym) in &mut cult_symbols {
                                                if sym.0 == event.0 {
                                                    image_node.color = BLUE.into();
                                                } else {
                                                    image_node.color = WHITE.into();
                                                }
                                            }
                                        },
                                    )
                                    .id();

                                let mut entity_commands = commands.spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    width: percent(100),
                                    ..Default::default()
                                });
                                let entity = entity_commands.id();
                                entity_commands.with_child(
                                    (
                                        TextInputNode {
                                            mode: TextInputMode::SingleLine,
                                            justification: Justify::Center,
                                            clear_on_submit: false,
                                            unfocus_on_submit: false,
                                            ..Default::default()
                                        },
                                        TextInputPrompt {
                                            text: "Cult Name".into(),
                                        ..Default::default()
                                        },
                                        Node {
                                            width: percent(75),
                                            height: px(28),
                                            margin: UiRect::all(px(10.0)),
                                            ..Default::default()
                                        },
                                        TextFont::from_font_size(SUB_HEADING).with_font(font_handle.clone()),
                                        TextColor::from(TEXT_NEUTRAL),
                                        BackgroundColor::from(BLACK),
                                    )
                                )
                                .with_children(|parent| {
                                    parent.spawn(Node {
                                        display: Display::Grid,
                                        grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
                                        grid_template_rows: RepeatedGridTrack::flex(2, 1.0),
                                        row_gap: px(16),
                                        column_gap: px(16),
                                        margin: UiRect::vertical(px(20)),
                                        ..default()
                                    }).with_children(|parent| {
                                        for (symbol_nr, symbol) in CULT_SYMBOLS.iter().enumerate() {
                                            let handle = asset_server.load(format!("{CULT_SYMBOL_PATH}/{symbol}"));
                                            parent.spawn((
                                                Node {
                                                    width: px(96),
                                                    height: px(96),
                                                    border: UiRect::all(px(4)),
                                                    border_radius: BorderRadius::all(px(4)),
                                                    align_items: AlignItems::Center,
                                                    justify_content: JustifyContent::Center,
                                                    ..default()
                                                },
                                                Button,
                                                BorderColor::all(BORDER),
                                                BackgroundColor::from(BUTTON_BACKGROUND),
                                                children![
                                                    (
                                                        Node {
                                                            width: percent(100),
                                                            height: percent(100),
                                                            ..default()
                                                        },
                                                        ImageNode {
                                                            image: handle.clone(),
                                                            color: WHITE.into(),
                                                            ..default()
                                                        },
                                                        CultSym(symbol_nr),
                                                    )
                                                ]
                                            )).observe(move |click: On<Pointer<Click>>, mut commands: Commands| {
                                                if click.button == PointerButton::Primary {
                                                    commands.insert_resource(CultSymbol(symbol_nr));
                                                    commands.trigger(CultSymbolChanged(symbol_nr));
                                                    commands.entity(entity).insert(DialogConfirm(true));
                                                }
                                            });
                                        }
                                    });
                                });

                                commands.spawn(Dialog::new()
                                    .with_title("main-menu-new-game-cult-title")
                                    .with_entity_body(entity)
                                    .with_cancel()
                                    .with_confirm_label("main-menu-new-game-cult-confirm")
                                    .with_confirm_disabled("main-menu-new-game-cult-confirm-tooltip"))
                                .observe(
                                        move |_: On<Add, DialogConfirmed>,
                                         mut commands: Commands,
                                         text_input_buffer: Single<&TextInputBuffer>,
                                        | {
                                             let text = text_input_buffer.get_text();
                                             let text = if text.is_empty() { "Nameless".into() } else { text };
                                             commands.insert_resource(CultName(text));
                                             commands.entity(cult_symbol_observer).despawn();
                                             commands.run_system_cached(setup_difficulties_dialog);
                                        }
                                    )
                                .observe(
                                        move |_: On<Add, DialogCancelled>,
                                              mut commands: Commands,
                                        | {
                                            commands.remove_resource::<CultSymbol>();
                                        }
                                    );
                            }
                        },
                    );
                    if any_save {
                        parent.spawn(button("main-menu-button-load-game")).observe(
                            |click: On<Pointer<Click>>,
                            mut commands: Commands,
                            asset_server: Res<AssetServer>,
                            font_handle: Res<FontHandle>| {
                                if click.button == PointerButton::Primary {
                                    open_load_game_popup(commands.reborrow(), asset_server, font_handle.clone());
                                }
                            },
                        );
                    }
                    parent.spawn(button("main-menu-button-quit")).observe(
                        |click: On<Pointer<Click>>, mut exit: MessageWriter<AppExit>| {
                            if click.button == PointerButton::Primary {
                                exit.write(AppExit::Success);
                            }
                        },
                    );
                });
        });
}

fn setup_difficulties_dialog(
    mut commands: Commands,
    display_font_handle: Res<DisplayFontHandle>,
    font_handle: Res<FontHandle>,
    difficulties_handle: Res<DifficultiesHandle>,
    difficulties_assets: Res<Assets<DifficultiesAsset>>,
) {
    let difficulty_observer = commands
        .add_observer(
            |event: On<DifficultyChanged>,
             mut commands: Commands,
             mut difficulties: Query<(&mut TextColor, &Difficulty)>| {
                for (mut text_color, diff) in &mut difficulties {
                    if diff.0 == event.0 {
                        text_color.0 = BLUE.into();
                    } else {
                        text_color.0 = WHITE.into();
                    }
                }
                commands.insert_resource(DifficultySelected(event.0.clone()));
            },
        )
        .id();

    let entity = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(px(20)),
            column_gap: px(20),
            ..default()
        })
        .with_children(|parent| {
            for (name, settings) in &difficulties_assets
                .get(difficulties_handle.0.id())
                .unwrap()
                .0
            {
                let name = name.clone();
                parent
                    .spawn((
                        Node {
                            height: px(350),
                            width: px(200),
                            border: UiRect::all(px(4)),
                            border_radius: BorderRadius::all(px(16)),
                            padding: UiRect::all(px(5)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(BORDER),
                        Button,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextKey::new(format!("main-menu-difficulty-{name}")),
                            TextColor::from(TEXT),
                            TextFont::from_font_size(HEADING)
                                .with_font(display_font_handle.clone()),
                            Difficulty(name.clone()),
                        ));

                        parent.spawn((
                            Node {
                                height: px(1),
                                width: percent(100),
                                margin: UiRect::top(px(5)).with_bottom(px(10)),
                                ..default()
                            },
                            BackgroundColor::from(BORDER),
                        ));

                        let condition = |text_key, vert| {
                            (
                                Node {
                                    margin: UiRect::vertical(px(vert)),
                                    ..default()
                                },
                                text_key,
                                TextColor::from(TEXT),
                                TextLayout::new_with_no_wrap(),
                                TextFont::from_font_size(NORMAL).with_font(font_handle.clone()),
                            )
                        };

                        parent.spawn(condition(
                            TextKey::new("main-menu-new-game-difficulty-starting-funds")
                                .add_arg("funds", settings.starting_funds),
                            10,
                        ));
                        parent
                            .spawn(Node {
                                height: px(75),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                margin: UiRect::vertical(px(8)),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn(condition(
                                    TextKey::new(
                                        "main-menu-new-game-difficulty-starting-followers",
                                    ),
                                    2,
                                ));
                                for (follower, count) in &settings.starting_followers {
                                    parent.spawn(condition(
                                        TextKey::new("follower-list-tooltip")
                                            .add_arg("count", *count as f64)
                                            .add_arg("follower-type", follower.as_str()),
                                        2,
                                    ));
                                }
                            });
                    })
                    .observe(move |click: On<Pointer<Click>>, mut commands: Commands| {
                        if click.button == PointerButton::Primary {
                            commands.trigger(DifficultyChanged(name.clone()));
                        }
                    });
            }
        })
        .id();

    let default_difficulty_name = difficulties_assets
        .get(difficulties_handle.0.id())
        .unwrap()
        .0
        .iter()
        .find(|(_, settings)| settings.default)
        .unwrap()
        .0;

    commands.trigger(DifficultyChanged(default_difficulty_name.clone()));

    commands
        .spawn(
            Dialog::new()
                .with_title("main-menu-new-game-difficulty-title")
                .with_entity_body(entity)
                .with_cancel()
                .with_max_width(percent(75))
                .with_confirm_label("main-menu-new-game-difficulty-confirm"),
        )
        .observe(
            move |_: On<Add, DialogConfirmed>,
                  mut commands: Commands,
                  mut game_state: ResMut<NextState<GameState>>,
                  difficulty_name: Res<DifficultySelected>,
                  difficulties_handle: Res<DifficultiesHandle>,
                  difficulties_assets: Res<Assets<DifficultiesAsset>>| {
                commands.entity(difficulty_observer).despawn();
                let difficulty = difficulties_assets
                    .get(difficulties_handle.0.id())
                    .unwrap()
                    .0
                    .get(&difficulty_name.0)
                    .unwrap()
                    .clone();
                commands.insert_resource(crate::common::Difficulty(difficulty_name.0.clone()));
                commands.insert_resource(NewGame { difficulty });
                commands.remove_resource::<DifficultySelected>();
                game_state.set(GameState::Main);
            },
        )
        .observe(move |_: On<Add, DialogCancelled>, mut commands: Commands| {
            commands.remove_resource::<DifficultySelected>();
        });
}
