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
    app.add_systems(Update, toast_timer.run_if(in_state(GameState::Main)));
}

/// A marker struct for the UI box that holds the toasts.
#[derive(Component)]
pub struct ToastContainer;

const TOAST_CONTAINER_MAX_Y: f32 = 300.0;

/// A timer component that is used to update the progress bar and then despawn the toast that is its parent.
#[derive(Component)]
struct ToastTimer(Timer);

/// Overflow holder for toasts.
#[derive(Resource, Default)]
pub struct WaitingToasts(Vec<TextKey>);

/// This setup function is scheduled by the parent module because it has to be ordered with respect to other setups.
pub(super) fn setup(mut commands: Commands, mapui: Single<Entity, With<MapUi>>) {
    commands.init_resource::<WaitingToasts>();

    // The toasts are added as children to this box, until it gets too large,
    // then the overflow is added to the WaitingToasts resource.
    commands.spawn((
        ToastContainer,
        ChildOf(*mapui),
        Hovered::default(),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            max_width: px(200),
            left: px(10),
            bottom: px(0),
            margin: px(10).bottom(),
            row_gap: px(10),
            ..default()
        },
    ));
}

/// A one-shot system that adds a toast to the UI
pub fn add_toast(
    In((textkey, forced)): In<(TextKey, bool)>,
    mut commands: Commands,
    container: Single<(Entity, &ComputedNode), With<ToastContainer>>,
    mut waiting: ResMut<WaitingToasts>,
    font_handle: Res<FontHandle>,
) {
    let (container, cnode) = *container;
    if !forced && cnode.content_size.y > TOAST_CONTAINER_MAX_Y {
        waiting.0.push(textkey);
        return;
    }
    let mut entity_commands = commands.spawn((
        ChildOf(container),
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
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
}

fn toast_timer(
    mut commands: Commands,
    mut progress: Query<(&ChildOf, &mut Node, &mut ToastTimer)>,
    hovered: Single<&Hovered, With<ToastContainer>>,
    mut waiting: ResMut<WaitingToasts>,
    time: Res<Time<Virtual>>,
) {
    // Don't advance timers while the user is hovering over the toast container
    if hovered.0 {
        return;
    }

    for (ChildOf(parent), mut node, mut timer) in &mut progress {
        timer.0.tick(time.delta());
        node.width = percent(timer.0.fraction_remaining() * 100.0);
        if timer.0.is_finished() {
            commands.entity(*parent).despawn();
            if !waiting.0.is_empty() {
                let next = waiting.0.remove(0);
                // Have to force the toast because the computed node size hasn't caught up yet.
                commands.run_system_cached_with(add_toast, (next, true));
            }
        }
    }
}
