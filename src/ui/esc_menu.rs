use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    constants::ui::*,
    save_load::SaveDirective,
    state::GameState,
    text::TextKey,
    time::{GameSpeedAction, GameSpeedChangedEvent},
    ui::FontHandle,
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
    menu: Option<Single<Entity, With<EscMenuRoot>>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Some(menu) = menu {
            commands.trigger(GameSpeedChangedEvent(GameSpeedAction::UiClose));
            commands.entity(*menu).despawn();
        } else {
            commands.trigger(GameSpeedChangedEvent(GameSpeedAction::UiOpen));
            open_esc_menu(commands.reborrow(), font_handle);
        }
    }
}

fn open_esc_menu(mut commands: Commands, font_handle: Res<FontHandle>) {
    let button = |key| {
        (
            Button,
            Node {
                width: percent(100),
                padding: UiRect::axes(px(30), px(15)),
                border: px(4).all(),
                border_radius: BorderRadius::all(px(20)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderColor::all(BORDER),
            BackgroundColor::from(BUTTON_BACKGROUND.with_alpha(OVERLAY_ALPHA)),
            children![(
                TextFont::from_font_size(40.0).with_font(font_handle.clone()),
                TextKey::new(key),
            ),],
        )
    };
    let root = commands
        .spawn((
            EscMenuRoot,
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
                width: percent(30),
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
        .spawn((ChildOf(menu), button("esc-menu-button-resume")))
        .observe(move |click: On<Pointer<Click>>, mut commands: Commands| {
            if click.button == PointerButton::Primary {
                commands.trigger(GameSpeedChangedEvent(GameSpeedAction::UiClose));
                commands.entity(root).despawn();
            }
        });

    commands
        .spawn((ChildOf(menu), button("esc-menu-button-save-game")))
        .observe(move |click: On<Pointer<Click>>, mut commands: Commands| {
            if click.button == PointerButton::Primary {
                commands.trigger(SaveDirective);
                commands.trigger(GameSpeedChangedEvent(GameSpeedAction::UiClose));
                commands.entity(root).despawn();
            }
        });

    commands
        .spawn((ChildOf(menu), button("esc-menu-button-to-main-menu")))
        .observe(
            move |click: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut next_state: ResMut<NextState<GameState>>| {
                if click.button == PointerButton::Primary {
                    commands.trigger(SaveDirective);
                    next_state.set(GameState::MainMenu);
                    commands.trigger(GameSpeedChangedEvent(GameSpeedAction::UiClose));
                    commands.entity(root).despawn();
                }
            },
        );

    commands
        .spawn((ChildOf(menu), button("esc-menu-button-quit")))
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
