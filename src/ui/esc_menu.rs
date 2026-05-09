use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    constants::ui::*,
    save_load::SaveDirective,
    state::GameState,
    text::TextKey,
    time::ForcePause,
    ui::{EmojiFontHandle, FontHandle},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, listen_esc_key.run_if(in_state(GameState::Main)));
}

#[derive(Component)]
struct EscMenuRoot;

fn listen_esc_key(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    font_handle: Res<FontHandle>,
    emoji_font_handle: Res<EmojiFontHandle>,
    menu: Option<Single<Entity, With<EscMenuRoot>>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Some(menu) = menu {
            commands.entity(*menu).despawn();
        } else {
            open_esc_menu(commands.reborrow(), font_handle, emoji_font_handle);
        }
    }
}

fn open_esc_menu(
    mut commands: Commands,
    font_handle: Res<FontHandle>,
    emoji_font_handle: Res<EmojiFontHandle>,
) {
    let button = |key, is_save| {
        (
            Button,
            Node {
                width: percent(100),
                padding: UiRect::axes(px(30), px(15)),
                border: px(4).all(),
                border_radius: BorderRadius::all(px(20)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(10),
                ..default()
            },
            BorderColor::all(BORDER),
            BackgroundColor::from(BUTTON_BACKGROUND.with_alpha(OVERLAY_ALPHA)),
            children![
                (
                    TextFont::from_font_size(40.0).with_font(font_handle.clone()),
                    TextKey::new(key),
                ),
                (
                    TextFont::from_font_size(32.0).with_font(emoji_font_handle.clone()),
                    Text::new(if is_save { "💾" } else { "" })
                )
            ],
        )
    };
    let root = commands
        .spawn((
            EscMenuRoot,
            ForcePause,
            GlobalZIndex(ZINDEX_ESC_MENU),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(DARK_OVERLAY.into()),
            FocusPolicy::Block,
        ))
        .id();

    let menu = commands
        .spawn((
            ChildOf(root),
            Node {
                width: percent(35),
                flex_direction: FlexDirection::Column,
                row_gap: px(20),
                border: px(6).all(),
                border_radius: BorderRadius::all(px(20)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .id();

    commands
        .spawn((ChildOf(menu), button("esc-menu-button-resume", false)))
        .observe(move |click: On<Pointer<Click>>, mut commands: Commands| {
            if click.button == PointerButton::Primary {
                commands.entity(root).despawn();
            }
        });

    commands
        .spawn((ChildOf(menu), button("esc-menu-button-save-game", true)))
        .observe(move |click: On<Pointer<Click>>, mut commands: Commands| {
            if click.button == PointerButton::Primary {
                commands.trigger(SaveDirective);
                commands.entity(root).despawn();
            }
        });

    commands
        .spawn((ChildOf(menu), button("esc-menu-button-to-main-menu", true)))
        .observe(
            move |click: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut next_state: ResMut<NextState<GameState>>| {
                if click.button == PointerButton::Primary {
                    commands.trigger(SaveDirective);
                    next_state.set(GameState::MainMenu);
                    commands.entity(root).despawn();
                }
            },
        );

    commands
        .spawn((ChildOf(menu), button("esc-menu-button-quit", true)))
        .observe(
            move |click: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut exit: MessageWriter<AppExit>| {
                if click.button == PointerButton::Primary {
                    commands.trigger(SaveDirective);
                    exit.write(AppExit::Success);
                }
            },
        );
}
