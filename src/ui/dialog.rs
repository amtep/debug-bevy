use bevy::{
    input_focus::InputFocus,
    prelude::*,
    ui::{FocusPolicy, InteractionDisabled},
};
use bevy_ui_text_input::TextInputNode;

use crate::{
    constants::ui::{ZINDEX_DIALOG, colors::*, fonts::*},
    state::GameState,
    text::TextKey,
    time::ForcePause,
    ui::{FontHandle, tooltip::Tooltip},
};

#[derive(Component)]
struct DialogRoot(Entity);

#[derive(Component)]
struct DialogInner;

#[derive(Debug, Clone)]
enum DialogBody {
    Text(TextKey),
    Entity(Entity),
}

#[derive(Component, Default, Clone)]
pub struct Dialog {
    max_width: Option<Val>,
    max_height: Option<Val>,
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
pub struct DialogCancelled;

#[derive(Component)]
pub struct DialogConfirm(pub bool);

#[derive(Component)]
struct ConfirmButton(Entity);

#[derive(Component)]
struct DialogBackground(u32);

pub fn plugin(app: &mut App) {
    app.add_systems(OnExit(GameState::Load), setup_observe_dialogs)
        .add_systems(Update, listen_dialog_confirm);
}

fn setup_observe_dialogs(mut commands: Commands) {
    commands.add_observer(on_dialog_add);
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        Visibility::Hidden,
        GlobalZIndex(ZINDEX_DIALOG),
        FocusPolicy::Block,
        DialogBackground(0),
    ));
    commands.add_observer(on_dialog_root_add);
    commands.add_observer(on_dialog_root_remove);
}

impl Dialog {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Dialog {
    pub fn with_max_width(self, max_width: Val) -> Self {
        Self {
            max_width: Some(max_width),
            ..self
        }
    }

    #[expect(dead_code)]
    pub fn with_max_height(self, max_height: Val) -> Self {
        Self {
            max_width: Some(max_height),
            ..self
        }
    }

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

fn on_dialog_add(
    add: On<Add, Dialog>,
    mut commands: Commands,
    dialogs: Query<&Dialog>,
    dialog_roots: Query<&DialogRoot>,
    dialog_background: Single<Entity, With<DialogBackground>>,
    font_handle: Res<FontHandle>,
) {
    let dialog_entity = add.entity;
    let dialog = dialogs.get(dialog_entity).unwrap().clone();
    #[allow(clippy::cast_possible_truncation)]
    let index = dialog_roots.count() as i32;
    let font = font_handle.clone();

    let dialog_root = commands
        .spawn((
            ChildOf(*dialog_background),
            DialogRoot(dialog_entity),
            Node {
                left: percent(50 + index),
                top: percent(50 + index),
                min_width: percent(25),
                max_width: dialog.max_width.unwrap_or_else(|| percent(50)),
                min_height: percent(50),
                max_height: dialog.max_height.unwrap_or_else(|| percent(75)),
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            UiTransform {
                translation: Val2::percent(-50.0, -50.0),
                ..Default::default()
            },
            ZIndex(index),
            Pickable::IGNORE,
            DespawnOnExit(GameState::Main),
        ))
        .observe(
            |press: On<Pointer<Press>>, mut dialog_roots: Query<&mut ZIndex, With<DialogRoot>>| {
                let current_z_index = dialog_roots.get(press.entity).unwrap().0;
                #[allow(clippy::cast_possible_truncation)]
                let top_z_index = (dialog_roots.count() - 1) as i32;
                if current_z_index != top_z_index {
                    for mut z_index in &mut dialog_roots {
                        if z_index.0 > current_z_index {
                            z_index.0 -= 1;
                        }
                    }
                    dialog_roots.get_mut(press.entity).unwrap().0 = top_z_index;
                }
            },
        )
        .id();

    if dialog.pause {
        commands.entity(dialog_root).insert(ForcePause);
    }

    let mut entity_commands = commands.spawn((
        ChildOf(dialog_root),
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            width: percent(100),
            height: percent(100),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(10)),
            padding: UiRect::axes(px(20), px(5)),
            ..default()
        },
        BorderColor::all(BORDER_HIGHLIGHT),
        BackgroundColor::from(DIALOG_BACKGROUND),
        DialogInner,
        UiTransform::default(),
    ));

    entity_commands.observe(
        |drag: On<Pointer<Drag>>,
         mut ui_transforms: Query<&mut UiTransform, With<DialogInner>>,
         ui_scale: Res<UiScale>| {
            if drag.button == PointerButton::Primary
                && let Ok(mut transform) = ui_transforms.get_mut(drag.entity)
            {
                let Val::Px(x) = transform.translation.x else {
                    unreachable!()
                };
                let Val::Px(y) = transform.translation.y else {
                    unreachable!()
                };
                transform.translation.x = px(x + drag.delta.x / ui_scale.0);
                transform.translation.y = px(y + drag.delta.y / ui_scale.0);
            }
        },
    );

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
                let confirm_disabled = dialog.confirm_disabled.clone();
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
                                commands
                                    .entity(confirm_button.0)
                                    .insert(InteractionDisabled);
                                if let Some(confirm_disabled) = confirm_disabled.clone() {
                                    commands.entity(confirm_button.0).insert(
                                        Tooltip::new_text_color(confirm_disabled, TEXT_NEGATIVE),
                                    );
                                }
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
                                commands.entity(dialog_entity).insert(DialogCancelled);
                                commands.entity(dialog_entity).despawn();
                                commands.entity(dialog_root).despawn();
                            }
                        },
                    );
                }

                let confirm_label = dialog
                    .confirm_label
                    .unwrap_or_else(|| TextKey::new("dialog-confirm"));

                let mut confirm_button = parent.spawn(button(confirm_label));

                if let Some(confirm_disabled) = dialog.confirm_disabled {
                    confirm_button.insert((
                        InteractionDisabled,
                        Tooltip::new_text_color(confirm_disabled, TEXT_NEGATIVE),
                    ));
                }

                confirm_button.observe(
                    move |click: On<Pointer<Click>>,
                          mut commands: Commands,
                          has_disableds: Query<Has<InteractionDisabled>, With<Button>>| {
                        if click.button == PointerButton::Primary
                            && !has_disableds.get(click.entity).unwrap()
                        {
                            commands.entity(dialog_entity).insert(DialogConfirmed);
                            commands.entity(dialog_entity).despawn();
                            commands.entity(dialog_root).despawn();
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

fn on_dialog_root_add(
    _: On<Add, DialogRoot>,
    mut dialog_background: Single<(&mut Visibility, &mut DialogBackground)>,
) {
    dialog_background.1.0 += 1;
    if *dialog_background.0 == Visibility::Hidden {
        *dialog_background.0 = Visibility::Inherited;
    }
}

fn on_dialog_root_remove(
    _: On<Remove, DialogRoot>,
    mut dialog_background: Single<(&mut Visibility, &mut DialogBackground)>,
) {
    dialog_background.1.0 -= 1;
    if dialog_background.1.0 == 0 {
        *dialog_background.0 = Visibility::Hidden;
    }
}

fn listen_dialog_confirm(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    dialog_roots: Query<(Entity, &ZIndex, &DialogRoot, &ConfirmButton)>,
    has_disableds: Query<Has<InteractionDisabled>, With<Button>>,
    input_focus: Res<InputFocus>,
    text_inputs: Query<(), With<TextInputNode>>,
) {
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Backspace]) && !dialog_roots.is_empty() {
        if let Some(focus) = &input_focus.0
            && text_inputs.contains(*focus)
        {
            return;
        }

        #[allow(clippy::cast_possible_truncation)]
        let top = (dialog_roots.count() - 1) as i32;
        let (dialog_root, dialog_entity, confirm_button) = dialog_roots
            .iter()
            .find_map(|(entity, z_index, dialog_root, confirm_button)| {
                (z_index.0 == top).then_some((entity, dialog_root.0, confirm_button.0))
            })
            .unwrap();

        if keys.just_pressed(KeyCode::Enter) {
            if has_disableds.get(confirm_button).unwrap() {
                return;
            }
            commands.entity(dialog_entity).insert(DialogConfirmed);
        }

        commands.entity(dialog_root).despawn();
    }
}
