use bevy::{input_focus::InputFocus, prelude::*, ui::InteractionDisabled};

use crate::{
    constants::ui::*,
    ui::{MapUi, menu::Menu},
};

pub fn setup_observe_buttons(mut commands: Commands) {
    commands.add_observer(
        |over: On<Pointer<Over>>,
         mut buttons: Query<
            (
                &mut BackgroundColor,
                Option<&mut BorderColor>,
                Has<InteractionDisabled>,
            ),
            With<Button>,
        >,
         mut input_focus: ResMut<InputFocus>| {
            if let Ok((mut background, border, has_interaction_disabled)) =
                buttons.get_mut(over.entity)
                && !has_interaction_disabled
            {
                background.0 = BUTTON_HOVER_BACKGROUND.into();
                if let Some(mut border) = border {
                    border.set_all(BORDER_HIGHLIGHT);
                }
            }
            input_focus.set(over.entity);
        },
    );
    commands.add_observer(
        |out: On<Pointer<Out>>,
         mut buttons: Query<
            (
                &mut BackgroundColor,
                Option<&mut BorderColor>,
                Has<InteractionDisabled>,
            ),
            With<Button>,
        >,
         mut input_focus: ResMut<InputFocus>| {
            if let Ok((mut background, border, has_interaction_disabled)) =
                buttons.get_mut(out.entity)
                && !has_interaction_disabled
            {
                background.0 = BUTTON_BACKGROUND.into();
                if let Some(mut border) = border {
                    border.set_all(BORDER);
                }
            }
            input_focus.clear();
        },
    );
    commands.add_observer(
        |press: On<Pointer<Press>>, mut buttons: Query<(&mut BackgroundColor, Has<InteractionDisabled>), With<Button>>| {
            if press.button == PointerButton::Primary
                && let Ok((mut background, has_interaction_disabled)) = buttons.get_mut(press.entity)
                && !has_interaction_disabled
            {
                background.0 = BUTTON_PRESSED_BACKGROUND.into();
            }
        },
    );
    commands.add_observer(
        |click: On<Pointer<Click>>, mut buttons: Query<(&mut BackgroundColor, &mut Button, Has<InteractionDisabled>)>| {
            if click.button == PointerButton::Primary
                && let Ok((mut background, mut button, has_interaction_disabled)) = buttons.get_mut(click.entity)
                && !has_interaction_disabled
            {
                background.0 = BUTTON_HOVER_BACKGROUND.into();
                button.set_changed();
            }
        },
    );
    commands.add_observer(
        |mut drag: On<Pointer<Drag>>, buttons: Query<(), With<Button>>| {
            if buttons.contains(drag.entity) {
                drag.propagate(false);
            }
        },
    );

    commands.add_observer(
        |add: On<Add, InteractionDisabled>,
         mut buttons: Query<
            (&Children, &mut BackgroundColor, Option<&mut BorderColor>),
            With<Button>,
        >,
         mut text_colors: Query<&mut TextColor>| {
            if let Ok((children, mut backgroun_color, border_color)) = buttons.get_mut(add.entity) {
                backgroun_color.0 = BUTTON_BACKGROUND.into();
                if let Some(mut border_color) = border_color {
                    border_color.set_all(BORDER);
                }
                for child in children {
                    if let Ok(mut text_color) = text_colors.get_mut(*child) {
                        text_color.0 = TEXT_DISABLED.into();
                    }
                }
            }
        },
    );

    commands.add_observer(
        |remove: On<Remove, InteractionDisabled>,
         mut buttons: Query<
            (&Children, &mut BackgroundColor, Option<&mut BorderColor>),
            With<Button>,
        >,
         mut text_colors: Query<&mut TextColor>| {
            if let Ok((children, mut backgroun_color, border_color)) =
                buttons.get_mut(remove.entity)
            {
                backgroun_color.0 = BUTTON_BACKGROUND.into();
                if let Some(mut border_color) = border_color {
                    border_color.set_all(BORDER);
                }
                for child in children {
                    if let Ok(mut text_color) = text_colors.get_mut(*child) {
                        text_color.0 = TEXT.into();
                    }
                }
            }
        },
    );

    // click outside any menu should close all opened menus
    commands.add_observer(
        |click: On<Pointer<Click>>,
         mut commands: Commands,
         map_ui: Query<&MapUi>,
         menus: Query<Entity, With<Menu>>| {
            if click.button == PointerButton::Primary && map_ui.contains(click.entity) {
                for menu in &menus {
                    commands.entity(menu).try_despawn();
                }
            }
        },
    );
}
