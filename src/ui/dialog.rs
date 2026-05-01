use bevy::{
    prelude::*,
    ui::{FocusPolicy, InteractionDisabled},
};
use pyri_tooltip::prelude::*;

use crate::{
    constants::ui::*,
    text::TextKey,
    time::{GameSpeedAction, GameSpeedChangedEvent},
    ui::FontHandle,
};

#[derive(Component)]
struct DialogRoot;

#[derive(Debug, Clone)]
enum DialogBody {
    Text(TextKey),
    Entity(Entity),
}

#[derive(Component, Default, Clone)]
pub struct Dialog {
    text_body_font: Option<Handle<Font>>,
    pause: bool,
    title: Option<TextKey>,
    body: Option<DialogBody>,
    /// default label: "Confirm"
    confirm_label: Option<TextKey>,
    /// tooltip for when the confirm is disabled
    confirm_disabled: Option<TextKey>,
    /// default label: None (no cancel button)
    cancel_label: Option<Option<TextKey>>,
}

#[derive(Component)]
pub struct DialogConfirmed;

#[derive(Component)]
pub struct DialogConfirm(pub bool);

#[derive(Component)]
struct ConfirmButton(Entity);

impl Dialog {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Dialog {
    pub fn with_pause(self) -> Self {
        Self {
            pause: true,
            ..self
        }
    }

    pub fn with_confirm_disabled(self, disabled_tooltip: impl Into<TextKey>) -> Self {
        Self {
            confirm_disabled: Some(disabled_tooltip.into()),
            ..self
        }
    }

    pub fn with_title(self, title: impl Into<TextKey>) -> Self {
        Self {
            title: Some(title.into()),
            ..self
        }
    }

    pub fn with_text_body(self, text: impl Into<TextKey>) -> Self {
        Self {
            body: Some(DialogBody::Text(text.into())),
            ..self
        }
    }

    #[expect(dead_code)]
    pub fn with_text_body_font(self, font: Handle<Font>) -> Self {
        Self {
            text_body_font: Some(font),
            ..self
        }
    }

    pub fn with_entity_body(self, entity: Entity) -> Self {
        Self {
            body: Some(DialogBody::Entity(entity)),
            ..self
        }
    }

    pub fn with_confirm_label(self, label: impl Into<TextKey>) -> Self {
        Self {
            confirm_label: Some(label.into()),
            ..self
        }
    }

    pub fn with_cancel(self) -> Self {
        Self {
            cancel_label: Some(None),
            ..self
        }
    }

    pub fn with_cancel_label(self, label: impl Into<TextKey>) -> Self {
        Self {
            cancel_label: Some(Some(label.into())),
            ..self
        }
    }
}

pub fn setup_observe_dialogs(mut commands: Commands) {
    commands.add_observer(on_dialog_add);
}

fn on_dialog_add(
    add: On<Add, Dialog>,
    mut commands: Commands,
    dialogs: Query<&Dialog>,
    font_handle: Res<FontHandle>,
) {
    let dialog_entity = add.entity;
    let dialog = dialogs.get(dialog_entity).unwrap().clone();
    let font = font_handle.0.clone();
    if dialog.pause {
        commands.trigger(GameSpeedChangedEvent(GameSpeedAction::DialogOpen));
    }

    let dialog_background = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            FocusPolicy::Block,
        ))
        .id();

    let mut entity_commands = commands.spawn((
        ChildOf(dialog_background),
        DialogRoot,
        Node {
            left: percent(50),
            top: percent(50),
            min_width: percent(25),
            max_width: percent(50),
            min_height: percent(50),
            max_height: percent(75),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(10)),
            padding: UiRect::axes(px(20), px(5)),
            ..Default::default()
        },
        UiTransform {
            translation: Val2::percent(-50.0, -50.0),
            ..Default::default()
        },
        BorderColor::all(BORDER_HIGHLIGHT),
        BackgroundColor::from(DIALOG_BACKGROUND),
        GlobalZIndex(ZINDEX_DIALOG),
    ));

    let dialog_root = entity_commands.id();

    let hrule = (
        Node {
            width: percent(90),
            height: px(1),
            margin: UiRect::vertical(px(5)),
            ..default()
        },
        BackgroundColor::from(BORDER),
    );

    if let Some(title) = dialog.title {
        entity_commands
            .with_child((
                title,
                TextColor::from(TEXT),
                TextFont::from_font_size(HEADING).with_font(font.clone()),
            ))
            .with_child(hrule.clone());
    } else {
        entity_commands.with_child(Node {
            height: px(10),
            ..default()
        });
    }

    let tooltip_entity = dialog.confirm_disabled.as_ref().map(|text_key| {
        entity_commands
            .commands()
            .spawn((
                text_key.clone(),
                TextColor::from(TEXT_NEGATIVE),
                TextFont::from_font_size(SMALL).with_font(font.clone()),
                Visibility::Hidden,
                GlobalZIndex(ZINDEX_DIALOG + 1),
            ))
            .id()
    });

    if let Some(body) = dialog.body {
        match body {
            DialogBody::Text(text_key) => {
                let font = dialog.text_body_font.unwrap_or_else(|| font.clone());
                entity_commands.with_child((
                    text_key,
                    TextColor::from(TEXT),
                    TextLayout::new_with_justify(Justify::Justified),
                    TextFont::from_font_size(SMALL).with_font(font),
                ))
            }
            DialogBody::Entity(entity) => {
                entity_commands.commands().entity(entity).observe(
                    move |confirm: On<Insert, DialogConfirm>,
                          mut commands: Commands,
                          dialog_confirms: Query<&DialogConfirm>,
                          confirm_buttons: Query<&ConfirmButton>| {
                        if let Ok(confirm_button) = confirm_buttons.get(dialog_root) {
                            let dialog_confirm = dialog_confirms.get(confirm.entity).unwrap();
                            if dialog_confirm.0 {
                                commands
                                    .entity(confirm_button.0)
                                    .try_remove::<(InteractionDisabled, Tooltip)>();
                            } else {
                                commands.entity(confirm_button.0).insert((
                                    InteractionDisabled,
                                    Tooltip::cursor(tooltip_entity.unwrap())
                                        .with_activation(TooltipActivation::SHORT_DELAY),
                                ));
                            }
                        }
                    },
                );
                entity_commands.add_child(entity)
            }
        }
        .with_child(Node {
            flex_grow: 1.0,
            ..default()
        })
        .with_child(hrule);
    }

    entity_commands.with_children(|parent| {
        parent
            .spawn(Node {
                width: percent(90),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                margin: UiRect::vertical(px(5)),
                column_gap: percent(5),
                ..Default::default()
            })
            .with_children(|parent| {
                let button = |text_key| {
                    (
                        Node {
                            width: percent(50),
                            height: px(30),
                            border: UiRect::all(px(1)),
                            border_radius: BorderRadius::all(px(5)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        BorderColor::all(BORDER),
                        BackgroundColor::from(BUTTON_BACKGROUND),
                        Button,
                        children![(
                            text_key,
                            TextColor::from(TEXT),
                            TextFont::from_font_size(LARGE).with_font(font.clone()),
                        )],
                    )
                };

                if let Some(cancel_label) = dialog.cancel_label {
                    let cancel_label =
                        cancel_label.unwrap_or_else(|| TextKey::new("dialog-cancel"));

                    parent.spawn(button(cancel_label)).observe(
                        move |click: On<Pointer<Click>>, mut commands: Commands| {
                            if click.button == PointerButton::Primary {
                                commands.entity(dialog_entity).remove::<Dialog>();
                                commands.entity(dialog_background).despawn();
                                if dialog.pause {
                                    commands.trigger(GameSpeedChangedEvent(
                                        GameSpeedAction::DialogClose,
                                    ));
                                }
                            }
                        },
                    );
                }

                let confirm_label = dialog
                    .confirm_label
                    .unwrap_or_else(|| TextKey::new("dialog-confirm"));

                let mut confirm_button = parent.spawn(button(confirm_label));

                if dialog.confirm_disabled.is_some() {
                    confirm_button.insert((
                        InteractionDisabled,
                        Tooltip::cursor(tooltip_entity.unwrap())
                            .with_activation(TooltipActivation::IMMEDIATE),
                    ));
                }

                confirm_button.observe(
                    move |click: On<Pointer<Click>>,
                          mut commands: Commands,
                          has_disableds: Query<Has<InteractionDisabled>>| {
                        if click.button == PointerButton::Primary
                            && !has_disableds.get(click.entity).unwrap()
                        {
                            commands.entity(dialog_entity).insert(DialogConfirmed);
                            commands.entity(dialog_entity).despawn();
                            commands.entity(dialog_background).despawn();
                            if let Some(tooltip_entity) = tooltip_entity {
                                commands.entity(tooltip_entity).despawn();
                            }
                            if dialog.pause {
                                commands
                                    .trigger(GameSpeedChangedEvent(GameSpeedAction::DialogClose));
                            }
                        }
                    },
                );

                let confirm_button = confirm_button.id();
                parent
                    .commands()
                    .entity(dialog_root)
                    .insert(ConfirmButton(confirm_button));
            });
    });
}
