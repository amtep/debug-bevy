//! A UI module for "toasts", which are little messages that temporarily show up in a corner of the screen.
//! They let the player know about events that may be interesting but are not very important.
//! Each new toast pushes the older ones up, and the oldest ones time out on their own.
//!
//! Invoke `commands.run_system_cached_with(add_toast, TextKey::new("something"))` to pop up a toast.
// TODO: maybe animate the toasts moving up and down?
use bevy::{picking::hover::Hovered, prelude::*};

use crate::{
    constants::ui::{
        TOAST_TIMER_DAYS,
        colors::{BORDER, TEXT, THEME_DARK_PURPLE},
        fonts::SMALL,
    },
    state::GameState,
    text::TextKey,
    ui::{FontHandle, MapUi},
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (toast_timer, animate_toasts)
            .chain()
            .run_if(in_state(GameState::Main)),
    )
    .add_systems(OnEnter(GameState::Main), setup);
}

/// The toasts currently on the screen, ordered from top to bottom.
/// They will all be children of [`MapUi`].
#[derive(Resource, Default)]
struct ActiveToasts(Vec<Entity>);

/// This is not a hard limit, because we don't know the size of new toasts in advance.
const TOAST_HEIGHT_LIMIT: f32 = 300.0;

/// A timer component that is used to update the progress bar and then despawn the toast that is its parent.
#[derive(Component)]
struct ToastTimer(Timer);

/// Holder for new toasts.
/// They will pop up on the screen in order, when there is room.
/// Add a new toast by pushing to this vector.
#[derive(Resource, Default)]
pub struct WaitingToasts(Vec<TextKey>);

impl WaitingToasts {
    pub fn push(&mut self, text_key: impl Into<TextKey>) {
        self.0.push(text_key.into());
    }
}

fn setup(mut commands: Commands) {
    commands.init_resource::<ActiveToasts>();
    commands.init_resource::<WaitingToasts>();
}

const ROW_GAP: f32 = 10.0;

fn animate_toasts(
    mut commands: Commands,
    mut active: ResMut<ActiveToasts>,
    mut waiting: ResMut<WaitingToasts>,
    mut toasts: Query<(&mut Node, &ComputedNode)>,
    mapui: Single<(Entity, &ComputedNode), With<MapUi>>,
    time: Res<Time<Real>>,
    font_handle: Res<FontHandle>,
) {
    let (mapui_e, mapui_cnode) = *mapui;
    let y_bottom = mapui_cnode.size.y * mapui_cnode.inverse_scale_factor;
    let mut y = y_bottom - ROW_GAP;

    let speed = 180.0 * time.delta_secs();

    for &e in active.0.iter().rev() {
        if let Ok((mut node, cnode)) = toasts.get_mut(e) {
            y -= cnode.content_size.y * cnode.inverse_scale_factor;
            // y is where the top of the node is supposed to be,
            // and we move the node slightly toward that position.
            if let Val::Px(node_y) = node.top {
                if node_y < y {
                    node.top = px(node_y + speed.min(y - node_y));
                } else if node_y > y {
                    node.top = px(node_y - speed.min(node_y - y));
                }
            }
            y -= ROW_GAP;
        }
    }

    if waiting.0.is_empty() || y_bottom - y >= TOAST_HEIGHT_LIMIT {
        return;
    }

    let textkey = waiting.0.remove(0);

    let mut entity_commands = commands.spawn((
        ChildOf(mapui_e),
        Node {
            position_type: PositionType::Absolute,
            top: px(y_bottom),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Hovered::default(),
    ));
    entity_commands.with_child((
        textkey,
        TextColor::from(TEXT),
        BackgroundColor::from(THEME_DARK_PURPLE),
        TextFont::from_font_size(SMALL).with_font(font_handle.0.clone()),
    ));
    entity_commands.with_child((
        Node {
            // This width will get updated by the toast timer system
            width: percent(100),
            height: px(4),
            ..default()
        },
        BackgroundColor::from(BORDER),
        ToastTimer(Timer::from_seconds(TOAST_TIMER_DAYS, TimerMode::Once)),
    ));
    active.0.push(entity_commands.id());
}

fn toast_timer(
    mut commands: Commands,
    mut progress: Query<(&ChildOf, &mut Node, &mut ToastTimer)>,
    hover: Query<&Hovered>,
    mut active: ResMut<ActiveToasts>,
    time: Res<Time<Virtual>>,
) {
    // Stop all toast timers if any of them are hovered.
    for &e in &active.0 {
        if let Ok(hovered) = hover.get(e)
            && hovered.0
        {
            return;
        }
    }

    for (ChildOf(parent), mut node, mut timer) in &mut progress {
        timer.0.tick(time.delta());
        node.width = percent(timer.0.fraction_remaining() * 100.0);
        if timer.0.is_finished() {
            active.0.retain(|e| e != parent);
            commands.entity(*parent).despawn();
        }
    }
}
