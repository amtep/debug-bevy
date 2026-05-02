use bevy::prelude::*;
use bevy_ui_text_input::{TextInputBuffer, TextInputMode, TextInputNode, TextInputPrompt};

use crate::{
    common::{CultName, CultSymbol},
    constants::ui::*,
    main_menu::NewGame,
    save_load::any_save_file_exists,
    state::GameState,
    text::TextKey,
    ui::{
        DisplayFontHandle, FontHandle, UnicodeFontHandle,
        dialog::{Dialog, DialogConfirm, DialogConfirmed},
        save_load::{load_most_recent_game, open_load_game_popup},
    },
};

#[derive(Component)]
struct CultSym(char);

#[derive(Event)]
struct CultSymbolChanged(char);

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
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderColor::all(WHITE),
            BackgroundColor::from(BUTTON_BACKGROUND),
            children![(
                TextFont::from_font_size(40.0).with_font(font_handle.0.clone()),
                TextKey::new(key),
            )],
        )
    };

    let cult_symbol_observer = commands
        .add_observer(
            |event: On<CultSymbolChanged>,
             mut commands: Commands,
             mut cult_symbols: Query<(&mut TextColor, &CultSym)>| {
                for (mut text_color, sym) in cult_symbols.iter_mut() {
                    if sym.0 == event.0 {
                        text_color.0 = TEXT_NEUTRAL.into();
                    } else {
                        text_color.0 = TEXT.into();
                    }
                }
                commands.insert_resource(CultSymbol(event.0));
            },
        )
        .id();

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
                    TextFont::from_font_size(150.0).with_font(display_font_handle.0.clone()),
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
                         font_handle: Res<FontHandle>,
                         unicode_font_handle: Res<UnicodeFontHandle>| {
                            if click.button == PointerButton::Primary {

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
                                            max_chars: Some(20),
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
                                        TextFont::from_font_size(SUB_HEADING).with_font(font_handle.0.clone()),
                                        TextColor::from(TEXT_NEUTRAL),
                                        BackgroundColor::from(BLACK),
                                    )
                                )
                                .with_children(|parent| {
                                    parent.spawn(Node {
                                        display: Display::Grid,
                                        align_items: AlignItems::Stretch,
                                        justify_items: JustifyItems::Stretch,
                                        grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
                                        grid_template_rows: RepeatedGridTrack::flex(2, 1.0),
                                        row_gap: px(10),
                                        column_gap: px(10),
                                        margin: UiRect::all(px(10.0)),
                                        ..default()
                                    }).with_children(|parent| {
                                        for symbol in ['✭', '✥', '✯', '❂', '♔', '♛', '❤', '⚜'] {
                                            parent.spawn((
                                                Node {
                                                    width: px(120),
                                                    height: px(120),
                                                    border: UiRect::all(px(5)),
                                                    border_radius: BorderRadius::all(px(10)),
                                                    align_items: AlignItems::Center,
                                                    justify_content: JustifyContent::Center,
                                                    ..default()
                                                },
                                                Button,
                                                BorderColor::all(BORDER),
                                                BackgroundColor::from(BUTTON_BACKGROUND),
                                                children![
                                                    (
                                                        Text::new(symbol),
                                                        CultSym(symbol),
                                                        TextColor::from(TEXT),
                                                        TextFont::from_font_size(72.0).with_font(unicode_font_handle.0.clone()),
                                                    )
                                                ]
                                            )).observe(move |click: On<Pointer<Click>>, mut commands: Commands| {
                                                if click.button == PointerButton::Primary {
                                                    commands.trigger(CultSymbolChanged(symbol));
                                                    commands.entity(entity).insert(DialogConfirm(true));
                                                }
                                            });
                                        }
                                    });
                                });

                                commands.spawn(Dialog::new()
                                    .with_title("main-menu-button-new-game")
                                    .with_entity_body(entity)
                                    .with_cancel()
                                    .with_confirm_disabled("main-menu-new-game-confirm-tooltip"))
                                .observe(
                                        move |_: On<Add, DialogConfirmed>,
                                         mut commands: Commands,
                                         mut game_state: ResMut<NextState<GameState>>,
                                         text_input_buffer: Single<&TextInputBuffer>,
                                        | {
                                             let text = text_input_buffer.get_text();
                                             let text = if text.is_empty() { "Nameless".into() } else { text };
                                             commands.insert_resource(CultName(text));
                                             commands.init_resource::<NewGame>();
                                             commands.entity(cult_symbol_observer).despawn();
                                             game_state.set(GameState::Main);
                                        }
                                    );
                            }
                        },
                    );
                    if any_save {
                        parent.spawn(button("main-menu-button-load-game")).observe(
                            |click: On<Pointer<Click>>,
                            mut commands: Commands,
                            font: Res<FontHandle>, unicode_font: Res<UnicodeFontHandle>| {
                                if click.button == PointerButton::Primary {
                                    open_load_game_popup(commands.reborrow(), font.0.clone(), unicode_font.0.clone());
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
