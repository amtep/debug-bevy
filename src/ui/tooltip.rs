use std::time::Duration;

use bevy::prelude::*;

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
pub enum TooltipContent {
    Text(TextKey, TextColor),
    Custom(Entity),
}

impl Default for TooltipContent {
    fn default() -> Self {
        Self::Text(TextKey::new("debug-tooltip"), TEXT.into())
    }
}

#[derive(Component, Default, Clone)]
pub struct Tooltip {
    content: TooltipContent,
}

impl Tooltip {
    #[expect(dead_code)]
    pub fn new_text(text: TextKey) -> Self {
        Self {
            content: TooltipContent::Text(text, TEXT.into()),
        }
    }

    pub fn new_text_color(text: TextKey, color: impl Into<Color>) -> Self {
        Self {
            content: TooltipContent::Text(text, TextColor::from(color.into())),
        }
    }

    pub fn new_custom(entity: Entity) -> Self {
        Self {
            content: TooltipContent::Custom(entity),
        }
    }
}

/// Placeholder entity before the tooltip box is instantiated.
#[derive(Component, Clone, Copy)]
pub struct TooltipOpen(pub Entity);

#[derive(Component, Clone, Copy)]
pub struct TooltipInner(pub Entity);

#[derive(Component, Clone, Copy)]
pub struct TooltipBox;

#[derive(Component, Clone)]
pub struct TooltipTimer(Timer);

#[derive(Resource)]
struct TooltipPlaceholder(Entity);

pub fn setup_observe_tooltips(mut commands: Commands) {
    let placeholder = commands.spawn(Visibility::Hidden).id();
    commands.insert_resource(TooltipPlaceholder(placeholder));
    commands.init_resource::<TooltipSetting>();
    commands.add_observer(on_tooltip_remove);
    commands.add_observer(on_tooltip_over);
    commands.add_observer(on_tooltip_out);
}

pub fn listen_tooltip_timers(
    mut commands: Commands,
    mut tooltip_timers: Query<(Entity, &mut TooltipTimer)>,
    time: Res<Time<Real>>,
    tooltips: Query<&Tooltip>,
    font_handle: Res<FontHandle>,
) {
    for (tooltip_entity, mut timer) in tooltip_timers.iter_mut() {
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
                    max_width: px(250),
                    margin: UiRect::top(px(5)),
                    position_type: PositionType::Absolute,
                    border: UiRect::all(px(1)),
                    padding: UiRect::all(px(2)),
                    ..default()
                },
                GlobalZIndex(ZINDEX_TOOLTIP),
                BackgroundColor::from(TOOLTIP_BACKGROUND),
                BorderColor::all(BORDER),
            ));
            let box_entity = entity_commands.id();

            match &tooltip.content {
                TooltipContent::Text(text_key, text_color) => {
                    let text = entity_commands
                        .commands()
                        .spawn((
                            ChildOf(box_entity),
                            text_key.clone(),
                            *text_color,
                            TextFont::from_font_size(SMALL).with_font(font_handle.0.clone()),
                        ))
                        .id();
                    commands.entity(tooltip_entity).insert(TooltipInner(text));
                }
                TooltipContent::Custom(entity) => {
                    entity_commands.add_child(*entity);
                    commands
                        .entity(tooltip_entity)
                        .insert(TooltipInner(*entity));
                }
            }

            commands
                .entity(tooltip_entity)
                .insert(TooltipOpen(box_entity));
        }
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
        if let TooltipContent::Custom(entity) = tooltip.content {
            commands.entity(placeholder.0).add_child(entity);
        }
        commands.entity(tooltip_box).try_despawn();
    };

    commands
        .entity(out.entity)
        .try_remove::<(TooltipTimer, TooltipOpen, TooltipInner)>();
}

fn on_tooltip_remove(remove: On<Remove, Tooltip>, mut commands: Commands) {
    commands
        .entity(remove.entity)
        .try_remove::<(TooltipTimer, TooltipOpen, TooltipInner)>();
}
