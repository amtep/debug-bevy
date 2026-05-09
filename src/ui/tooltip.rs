use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::constants::ui::*;
use crate::text::TextKey;
use crate::ui::FontHandle;

#[derive(Resource, Clone)]
pub struct TooltipSetting {
    delay: Duration,
}

impl Default for TooltipSetting {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(200),
        }
    }
}

#[derive(Clone)]
enum TooltipInner {
    StaticTexts(Vec<(Text, TextColor)>),
    TextKeys(Vec<(TextKey, TextColor)>),
    Custom(Entity),
}

impl Default for TooltipInner {
    fn default() -> Self {
        Self::TextKeys(vec![(TextKey::new("debug-tooltip"), TEXT.into())])
    }
}

#[derive(Component, Default, Clone)]
pub struct Tooltip {
    content: TooltipInner,
}

impl Tooltip {
    pub fn new_text(text: impl Into<TextKey>) -> Self {
        Self {
            content: TooltipInner::TextKeys(vec![(text.into(), TEXT.into())]),
        }
    }

    pub fn new_text_color(text: impl Into<TextKey>, color: impl Into<Color>) -> Self {
        Self {
            content: TooltipInner::TextKeys(vec![(text.into(), TextColor::from(color.into()))]),
        }
    }

    pub fn new_texts(texts: impl IntoIterator<Item = impl Into<TextKey>>) -> Self {
        Self {
            content: TooltipInner::TextKeys(
                texts
                    .into_iter()
                    .map(|text| (text.into(), TEXT.into()))
                    .collect(),
            ),
        }
    }

    #[expect(dead_code)]
    pub fn new_text_colors(
        text_colors: impl IntoIterator<Item = (impl Into<TextKey>, impl Into<Color>)>,
    ) -> Self {
        Self {
            content: TooltipInner::TextKeys(
                text_colors
                    .into_iter()
                    .map(|(text, color)| (text.into(), TextColor::from(color.into())))
                    .collect(),
            ),
        }
    }

    pub fn new_static_text(text: impl Into<Text>) -> Self {
        Self {
            content: TooltipInner::StaticTexts(vec![(text.into(), TEXT.into())]),
        }
    }

    pub fn new_static_text_color(text: impl Into<Text>, color: impl Into<Color>) -> Self {
        Self {
            content: TooltipInner::StaticTexts(vec![(text.into(), TextColor::from(color.into()))]),
        }
    }

    pub fn new_static_texts(texts: impl IntoIterator<Item = impl Into<Text>>) -> Self {
        Self {
            content: TooltipInner::StaticTexts(
                texts
                    .into_iter()
                    .map(|text| (text.into(), TEXT.into()))
                    .collect(),
            ),
        }
    }

    #[expect(dead_code)]
    pub fn new_static_text_colors(
        text_colors: impl IntoIterator<Item = (impl Into<Text>, impl Into<Color>)>,
    ) -> Self {
        Self {
            content: TooltipInner::StaticTexts(
                text_colors
                    .into_iter()
                    .map(|(text, color)| (text.into(), TextColor::from(color.into())))
                    .collect(),
            ),
        }
    }

    pub fn new_custom(entity: Entity) -> Self {
        Self {
            content: TooltipInner::Custom(entity),
        }
    }
}

/// Tooltip box and inner content entities.
#[derive(Component, Clone, Copy)]
pub struct TooltipOpen(pub Entity, pub Entity);

/// Tooltip box that is opened upon activation.
#[derive(Component, Clone, Copy)]
pub struct TooltipBox;

#[derive(Component, Clone)]
pub struct TooltipTimer(Timer);

#[derive(Resource)]
struct TooltipPlaceholder(Entity);

pub fn setup_observe_tooltips(mut commands: Commands) {
    // hide the tooltip custom entity when it is not open.
    let placeholder = commands.spawn(Visibility::Hidden).id();
    commands.insert_resource(TooltipPlaceholder(placeholder));
    commands.init_resource::<TooltipSetting>();
    commands.add_observer(on_tooltip_add);
    commands.add_observer(on_tooltip_remove);
    commands.add_observer(on_tooltip_over);
    commands.add_observer(on_tooltip_out);
}

const TOOLTIP_Y: f32 = 5.0;

pub fn listen_tooltip_timers(
    mut commands: Commands,
    mut tooltip_timers: Query<(Entity, &mut TooltipTimer)>,
    time: Res<Time<Real>>,
    tooltips: Query<&Tooltip>,
    font_handle: Res<FontHandle>,
) {
    for (tooltip_entity, mut timer) in &mut tooltip_timers {
        timer.0.tick(time.delta());

        if timer.0.is_finished() {
            commands.entity(tooltip_entity).remove::<TooltipTimer>();

            let tooltip = tooltips.get(tooltip_entity).unwrap();
            let mut entity_commands = commands.spawn((
                TooltipBox,
                ChildOf(tooltip_entity),
                Node {
                    left: px(10),
                    top: percent(100),
                    max_width: px(200),
                    margin: UiRect::top(px(TOOLTIP_Y)),
                    position_type: PositionType::Absolute,
                    border: UiRect::all(px(1)),
                    padding: UiRect::all(px(2)),
                    ..default()
                },
                Visibility::Hidden,
                GlobalZIndex(ZINDEX_TOOLTIP),
                BackgroundColor::from(TOOLTIP_BACKGROUND),
                BorderColor::all(BORDER),
            ));
            let box_entity = entity_commands.id();

            let inner_entity = match &tooltip.content {
                TooltipInner::TextKeys(text_colors) => entity_commands
                    .commands()
                    .spawn((
                        ChildOf(box_entity),
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        for (text, color) in text_colors {
                            parent.spawn((
                                text.clone(),
                                *color,
                                TextFont::from_font_size(SMALL).with_font(font_handle.clone()),
                            ));
                        }
                    })
                    .id(),
                TooltipInner::StaticTexts(text_colors) => entity_commands
                    .commands()
                    .spawn((
                        ChildOf(box_entity),
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        for (text, color) in text_colors {
                            parent.spawn((
                                text.clone(),
                                *color,
                                TextFont::from_font_size(SMALL).with_font(font_handle.clone()),
                            ));
                        }
                    })
                    .id(),
                TooltipInner::Custom(entity) => {
                    entity_commands.add_child(*entity);
                    *entity
                }
            };

            commands
                .entity(tooltip_entity)
                .insert(TooltipOpen(box_entity, inner_entity));
        }
    }
}

fn on_tooltip_add(
    add: On<Add, Tooltip>,
    mut commands: Commands,
    tooltips: Query<&Tooltip>,
    placeholder: Res<TooltipPlaceholder>,
) {
    if let TooltipInner::Custom(entity) = tooltips.get(add.entity).unwrap().content {
        commands.entity(placeholder.0).add_child(entity);
    }
}

fn on_tooltip_over(
    over: On<Pointer<Over>>,
    mut commands: Commands,
    tooltips: Query<(), With<Tooltip>>,
    tooltip_setting: Res<TooltipSetting>,
    tooltip_opens: Query<(), With<TooltipOpen>>,
) {
    if tooltips.contains(over.entity) && !tooltip_opens.contains(over.entity) {
        commands.entity(over.entity).insert(TooltipTimer(Timer::new(
            tooltip_setting.delay,
            TimerMode::Once,
        )));
    }
}

fn on_tooltip_out(
    out: On<Pointer<Out>>,
    mut commands: Commands,
    placeholder: Res<TooltipPlaceholder>,
    tooltips: Query<&Tooltip>,
    tooltip_opens: Query<&TooltipOpen>,
) {
    if let Ok(tooltip) = tooltips.get(out.entity)
        && let Ok(tooltip_box) = tooltip_opens.get(out.entity).map(|open| open.0)
    {
        if let TooltipInner::Custom(entity) = tooltip.content {
            commands.entity(placeholder.0).add_child(entity);
        }
        commands.entity(tooltip_box).try_despawn();
    }

    commands
        .entity(out.entity)
        .try_remove::<(TooltipTimer, TooltipOpen)>();
}

fn on_tooltip_remove(remove: On<Remove, Tooltip>, mut commands: Commands) {
    commands
        .entity(remove.entity)
        .try_remove::<(TooltipTimer, TooltipOpen)>();
}

pub fn override_tooltip_position(
    mut tooltip_boxes: Query<
        (
            &ChildOf,
            &UiGlobalTransform,
            &mut UiTransform,
            &mut Visibility,
            &ComputedNode,
        ),
        With<TooltipBox>,
    >,
    compute_nodes: Query<&ComputedNode>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let (window_width, window_height) = (window.width(), window.height());
    for (parent, global_transform, mut transform, mut visibility, computed_node) in
        &mut tooltip_boxes
    {
        if *visibility == Visibility::Hidden {
            let translation = global_transform.translation;
            let (x, y) = (translation.x, translation.y);
            let width = computed_node.size.x;
            let height = computed_node.size.y;

            #[expect(clippy::useless_let_if_seq, reason = "this is doing something else")]
            let mut is_visible = true;

            if x + width / 2.0 > window_width {
                transform.translation.x =
                    px((window_width - width / 2.0 - x) * computed_node.inverse_scale_factor);
                is_visible = false;
            }

            #[expect(clippy::suboptimal_flops, reason = "looks better this way")]
            if y + height / 2.0 > window_height {
                // place the tooltip box above the parent of the tooltip
                // but if the tooltip box grows, then it might cover the parent
                transform.translation.y = px(-(height
                    + compute_nodes.get(parent.0).unwrap().size.y
                    + (TOOLTIP_Y * 2.0))
                    * computed_node.inverse_scale_factor);
                is_visible = false;
            }

            if is_visible {
                *visibility = Visibility::Inherited;
            }
        }
    }
}
