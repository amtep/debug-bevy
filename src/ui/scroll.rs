use bevy::{input::mouse::MouseScrollUnit, prelude::*};

pub fn listen_scroll(mut ev: On<Pointer<Scroll>>, mut q: Query<(&mut ScrollPosition, &Node)>) {
    if let Ok((mut scroll_position, node)) = q.get_mut(ev.entity) {
        let distance = if ev.event.unit == MouseScrollUnit::Line {
            ev.event.y * 20.0
        } else {
            ev.event.y
        };
        if node.overflow.y == OverflowAxis::Scroll {
            // Vertical scrollable
            scroll_position.0.y -= distance;
            ev.propagate(false);
        } else if node.overflow.x == OverflowAxis::Scroll {
            // Horizontal scrollable
            scroll_position.0.x -= distance;
            ev.propagate(false);
        }
    }
}
