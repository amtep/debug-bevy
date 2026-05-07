use bevy::{prelude::*, ui::InteractionDisabled, window::PrimaryWindow};

use crate::{
    constants::ui::*,
    text::TextKey,
    ui::{FontHandle, tooltip::Tooltip},
};

const MENU_Y: f32 = 5.0;

#[derive(Clone)]
pub struct MenuItem {
    pub enabled: bool,
    pub text: TextKey,
    pub tooltip: TextKey,
}

#[derive(Clone)]
pub struct MenuEntry {
    heading: TextKey,
    items: Vec<MenuItem>,
}

impl MenuEntry {
    pub fn new(heading: impl Into<TextKey>) -> Self {
        Self {
            heading: heading.into(),
            items: Vec::new(),
        }
    }

    #[expect(dead_code)]
    pub fn with_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn with_items_iter<I: IntoIterator<Item = MenuItem>>(mut self, items: I) -> Self {
        self.items.extend(items);
        self
    }

    #[expect(dead_code)]
    pub fn with_items(mut self, items: &[MenuItem]) -> Self {
        self.items.extend_from_slice(items);
        self
    }
}

#[derive(Component, Default, Clone)]
pub struct Menu {
    entries: Vec<MenuEntry>,
}

impl Menu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_entry(mut self, entry: MenuEntry) -> Self {
        self.entries.push(entry);
        self
    }

    #[expect(dead_code)]
    pub fn with_entries_iter<I: IntoIterator<Item = MenuEntry>>(mut self, entries: I) -> Self {
        self.entries.extend(entries);
        self
    }

    #[expect(dead_code)]
    pub fn with_entries(mut self, entries: &[MenuEntry]) -> Self {
        self.entries.extend_from_slice(entries);
        self
    }
}

#[derive(Component)]
pub struct MenuRootUi;

#[derive(Component)]
struct MenuHeadingUi;

#[derive(Component)]
struct MenuItemUi;

#[derive(Component)]
pub struct MenuClicked(pub String);

pub fn setup_observe_menus(mut commands: Commands) {
    commands.add_observer(on_menu_add);
}

fn on_menu_add(
    add: On<Add, Menu>,
    mut commands: Commands,
    menus: Query<&Menu>,
    font_handle: Res<FontHandle>,
) {
    let menu_entity = add.entity;
    let menu = menus.get(menu_entity).unwrap().clone();
    let font = font_handle.clone();

    let mut entity_commands = commands.entity(menu_entity);

    entity_commands.insert((
        MenuRootUi,
        Node {
            left: px(10),
            top: percent(100),
            min_width: px(100),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            margin: UiRect::top(px(MENU_Y)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(2)),
            ..default()
        },
        BackgroundColor::from(MENU_BACKGROUND),
        BorderColor::all(BORDER_HIGHLIGHT),
        GlobalZIndex(ZINDEX_MENU),
        Visibility::Hidden,
    ));

    let hrule = (
        Node {
            width: auto(),
            height: px(1),
            align_self: AlignSelf::Center,
            margin: UiRect::right(px(2)),
            flex_grow: 1.0,
            ..default()
        },
        BackgroundColor::from(BORDER_HIGHLIGHT),
    );

    entity_commands
        .with_children(move |parent| {
            for entry in menu.entries {
                parent
                    .spawn((
                        Node {
                            padding: UiRect::axes(px(5), px(2)),
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        MenuHeadingUi,
                    ))
                    .with_children(|parent| {
                        parent.spawn(hrule.clone());
                        parent.spawn((
                            entry.heading,
                            TextColor::from(TEXT_HIGHLIGHT),
                            TextFont::from_font_size(SMALL).with_font(font.clone()),
                        ));
                    });

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    })
                    .with_children(|parent| {
                        for (index, item) in entry.items.into_iter().enumerate() {
                            let background_color = if index % 2 == 0 { WHITE.with_alpha(0.01) } else { Srgba::NONE };
                            let mut cmd = parent.spawn((
                                MenuItemUi,
                                Button,
                                Node {
                                    width: percent(100),
                                    padding: UiRect::axes(px(5), px(2)),
                                    ..default()
                                },
                                BackgroundColor::from(background_color),
                                Tooltip::new_text_color(item.tooltip, if item.enabled { TEXT } else { TEXT_NEGATIVE })
                            ));

                            if !item.enabled {
                                cmd.insert(InteractionDisabled);
                            }

                            let text = item.text.0.clone();

                            cmd.with_child((
                                item.text,
                                TextColor::from(if item.enabled { TEXT } else { TEXT_DISABLED }),
                                TextFont::from_font_size(SMALL).with_font(font.clone()),
                                TextLayout::new_with_no_wrap(),
                                )).observe(move |mut click: On<Pointer<Click>>,
                                                     mut commands: Commands,
                                                     has_disableds: Query<Has<InteractionDisabled>>| {
                                            if click.button == PointerButton::Primary && !has_disableds.get(click.entity).unwrap() {
                                                commands.entity(menu_entity).insert(MenuClicked(text.clone()));
                                                commands.entity(menu_entity).despawn();
                                            }
                                            // prevent the click to reopen menu
                                            click.propagate(false);
                                });
                        }
                    });
            }
        })
        .observe(|mut over: On<Pointer<Over>>| {
            over.propagate(false);
        })
        .observe(|mut out: On<Pointer<Out>>| {
            out.propagate(false);
        });
}

pub fn override_menu_position(
    mut menu_roots: Query<
        (
            &ChildOf,
            &UiGlobalTransform,
            &mut UiTransform,
            &mut Visibility,
            &ComputedNode,
        ),
        With<MenuRootUi>,
    >,
    compute_nodes: Query<&ComputedNode>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let (window_width, window_height) = (window.width(), window.height());
    for (parent, global_transform, mut transform, mut visibility, computed_node) in &mut menu_roots
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
                    + (MENU_Y * 2.0))
                    * computed_node.inverse_scale_factor);
                is_visible = false;
            }

            if is_visible {
                *visibility = Visibility::Inherited;
            }
        }
    }
}
