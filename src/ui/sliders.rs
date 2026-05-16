use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{
        CoreSliderDragState, Slider as BevySlider, SliderPrecision, SliderRange, SliderThumb,
        SliderValue, TrackClick, observe, slider_self_update,
    },
};

use crate::{
    constants::ui::colors::{DARK_GREY, WHITE, YELLOW},
    text::TextKey,
};

pub fn plugin(app: &mut App) {
    app.add_observer(on_slider_add)
        .add_systems(Update, (update_slider_visuals, update_slider_texts));
}

#[derive(Component)]
#[require(SliderValue(0.0), SliderRange::new(0.0, 100.0))]
pub struct Slider {
    major_axis_size: Val,
    is_vertical: bool,
    precision: i32,
}

/// Can be either static Text or [`TextKey`] with value variable.
#[derive(Component)]
pub enum SliderText {
    Static(Entity),
    TextKey(Entity, &'static str),
}

impl Slider {
    pub fn new(is_vertical: bool) -> Self {
        Self {
            major_axis_size: Val::Auto,
            is_vertical,
            precision: 0,
        }
    }

    pub fn with_major_axis_size(self, size: Val) -> Self {
        Self {
            major_axis_size: size,
            ..self
        }
    }

    pub fn with_precision(self, precision: i32) -> Self {
        Self { precision, ..self }
    }
}

const SLIDER_CONSTANT: u32 = 4;

fn on_slider_add(add: On<Add, Slider>, mut commands: Commands, sliders: Query<&Slider>) {
    let slider = sliders.get(add.entity).unwrap();

    let node = if slider.is_vertical {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            width: px(SLIDER_CONSTANT * 2),
            height: slider.major_axis_size,
            ..default()
        }
    } else {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            height: px(SLIDER_CONSTANT * 2),
            width: slider.major_axis_size,
            ..default()
        }
    };

    let bar_node = if slider.is_vertical {
        Node {
            width: px(SLIDER_CONSTANT),
            border_radius: BorderRadius::all(px(SLIDER_CONSTANT as f32 / 2.0)),
            ..default()
        }
    } else {
        Node {
            height: px(SLIDER_CONSTANT),
            border_radius: BorderRadius::all(px(SLIDER_CONSTANT as f32 / 2.0)),
            ..default()
        }
    };

    let thumb_node = if slider.is_vertical {
        Node {
            display: Display::Flex,
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(SLIDER_CONSTANT * 2),
            ..default()
        }
    } else {
        Node {
            display: Display::Flex,
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(SLIDER_CONSTANT * 2),
            top: px(0),
            bottom: px(0),
            ..default()
        }
    };

    let thumb_inner_node = if slider.is_vertical {
        Node {
            display: Display::Flex,
            width: px(SLIDER_CONSTANT * 2),
            height: px(SLIDER_CONSTANT * 2),
            position_type: PositionType::Absolute,
            bottom: percent(0),
            border_radius: BorderRadius::MAX,
            ..default()
        }
    } else {
        Node {
            display: Display::Flex,
            width: px(SLIDER_CONSTANT * 2),
            height: px(SLIDER_CONSTANT * 2),
            position_type: PositionType::Absolute,
            left: percent(0),
            border_radius: BorderRadius::MAX,
            ..default()
        }
    };

    commands
        .entity(add.entity)
        .insert((
            node,
            Hovered::default(),
            BevySlider {
                track_click: TrackClick::Snap,
            },
            SliderPrecision(slider.precision),
            children![
                (bar_node, BackgroundColor::from(DARK_GREY)),
                (
                    thumb_node,
                    children![(SliderThumb, BackgroundColor::from(WHITE), thumb_inner_node)]
                )
            ],
            observe(slider_self_update),
        ))
        .observe(|mut drag: On<Pointer<Drag>>| {
            drag.propagate(false);
        });
}

fn update_slider_visuals(
    sliders: Query<
        (
            Entity,
            &SliderValue,
            &SliderRange,
            &Hovered,
            &CoreSliderDragState,
            &Slider,
        ),
        (
            Or<(
                Changed<SliderValue>,
                Changed<Hovered>,
                Changed<CoreSliderDragState>,
            )>,
        ),
    >,
    children: Query<&Children>,
    mut thumbs: Query<(&mut Node, &mut BackgroundColor), With<SliderThumb>>,
) {
    for (slider_entity, value, range, hovered, drag_state, slider) in sliders.iter() {
        for child in children.iter_descendants(slider_entity) {
            if let Ok((mut thumb_node, mut thumb_bg)) = thumbs.get_mut(child) {
                let position = range.thumb_position(value.0) * 100.0;
                if slider.is_vertical {
                    thumb_node.bottom = percent(position);
                } else {
                    thumb_node.left = percent(position);
                }
                thumb_bg.0 = if hovered.0 | drag_state.dragging {
                    YELLOW
                } else {
                    WHITE
                }
                .into();
            }
        }
    }
}

#[allow(clippy::cast_sign_loss)]
fn update_slider_texts(
    sliders: Query<(&SliderValue, &Slider, &SliderText), Changed<SliderValue>>,
    mut text_keys: Query<&mut TextKey>,
    mut texts: Query<&mut Text>,
) {
    for (value, slider, slider_text) in sliders.iter() {
        let value = value.0 + 0.0; // force +0.0
        if let SliderText::Static(entity) = slider_text
            && let Ok(mut text) = texts.get_mut(*entity)
        {
            **text = format!("{value:.0$}", slider.precision.max(0) as usize);
        } else if let SliderText::TextKey(entity, variable) = slider_text
            && let Ok(mut text_key) = text_keys.get_mut(*entity)
        {
            text_key.replace_arg(variable, value as f64);
        }
    }
}
