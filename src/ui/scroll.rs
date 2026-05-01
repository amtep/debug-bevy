use bevy::{input::mouse::MouseScrollUnit, prelude::*};

pub fn listen_scroll(
    mut ev: On<Pointer<Scroll>>,
    mut q: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    if let Ok((mut scroll_position, node, computed_node)) = q.get_mut(ev.entity) {
        let distance = if ev.unit == MouseScrollUnit::Line {
            ev.y * 20.0
        } else {
            ev.y
        };
        if node.overflow.y == OverflowAxis::Scroll {
            let max = (computed_node.content_size.y - computed_node.size.y)
                * computed_node.inverse_scale_factor;
            // Vertical scrollable
            scroll_position.0.y = (scroll_position.0.y - distance).max(0.0).min(max);
        } else if node.overflow.x == OverflowAxis::Scroll {
            let max = (computed_node.content_size.x - computed_node.size.x)
                * computed_node.inverse_scale_factor;
            // Horizontal scrollable
            scroll_position.0.x = (scroll_position.0.y - distance).max(0.0).min(max);
        }
        ev.propagate(false);
    }
}
