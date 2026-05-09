use bevy::{
    prelude::*,
    ui::{InteractionDisabled, UiSystems},
    window::PrimaryWindow,
};

use crate::{
    constants::ui::*,
    state::GameState,
    text::TextKey,
    ui::{FontHandle, tooltip::Tooltip},
};

pub fn plugin(app: &mut App) {
    app.add_systems(OnExit(GameState::Load), setup).add_systems(
        PostUpdate,
        override_menu_position
            .run_if(not(in_state(GameState::Load)))
            .after(UiSystems::Layout),
    );
}

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
    title: Option<TextKey>,
    entries: Vec<MenuEntry>,
}

impl Menu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: impl Into<TextKey>) -> Self {
        self.title = Some(title.into());
        self
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
struct MenuTitleUi;

#[derive(Component)]
struct MenuHeadingUi;

#[derive(Component)]
struct MenuItemUi;

#[derive(Component)]
pub struct MenuClicked(pub String);

pub fn setup(mut commands: Commands) {
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
    let mut entity_commands = commands.entity(menu_entity);

    entity_commands
        .insert((
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
        ))
        .observe(|mut click: On<Pointer<Click>>| {
            click.propagate(false);
        })
        .observe(|mut press: On<Pointer<Press>>| {
            press.propagate(false);
        })
        .observe(|mut over: On<Pointer<Over>>| {
            over.propagate(false);
        })
        .observe(|mut out: On<Pointer<Out>>| {
            out.propagate(false);
        });

    let hrule = (
        Node {
            min_width: px(20),
            height: px(1),
            align_self: AlignSelf::Center,
            margin: UiRect::right(px(2)),
            flex_grow: 1.0,
            ..default()
        },
        BackgroundColor::from(BORDER_HIGHLIGHT),
    );

    let short_hrule = (
        Node {
            width: px(5),
            height: px(1),
            align_self: AlignSelf::Center,
            margin: UiRect::horizontal(px(1)),
            ..default()
        },
        BackgroundColor::from(BORDER),
    );

    let font = TextFont::from_font_size(SMALL).with_font(font_handle.clone());

    entity_commands
        .with_children(move |parent| {
            if let Some(title) = menu.title {
                parent.spawn((
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::axes(px(5), px(2)),
                        ..default()
                    },
                    BackgroundColor::from(DARK_OVERLAY),
                    MenuTitleUi,
                ))
                .with_child(short_hrule.clone())
                .with_child((
                    title,
                    TextFont::from_font_size(NORMAL).with_font(font_handle.clone()),
                    TextColor::from(TEXT),
                    TextLayout::new_with_no_wrap(),
                ))
                .with_child(short_hrule);
            }

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
                            font.clone(),
                            TextLayout::new_with_no_wrap(),
                        ));
                    });

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    })
                    .with_children(|parent| {
                        for (index, item) in entry.items.into_iter().enumerate() {
                            let mut cmd = parent.spawn((
                                MenuItemUi,
                                Button,
                                Node {
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                BackgroundColor::from(MENU_BACKGROUND),
                                Tooltip::new_text_color(item.tooltip, if item.enabled { TEXT } else { TEXT_NEGATIVE })
                            ));

                            if !item.enabled {
                                cmd.insert(InteractionDisabled);
                            }

                            if index % 2 == 0 {
                                cmd.with_child((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        width: percent(100),
                                        height: percent(100),
                                        ..default()
                                    },
                                    BackgroundColor::from(WHITE.with_alpha(0.01))
                                ));
                            }


                            let text = item.text.0.clone();
                            cmd.with_child((
                                Node {
                                    margin: UiRect::axes(px(5), px(2)),
                                    ..default()
                                },
                                item.text,
                                TextColor::from(if item.enabled { TEXT } else { TEXT_DISABLED }),
                                font.clone(),
                                TextLayout::new_with_no_wrap(),
                                )).observe(move |click: On<Pointer<Click>>,
                                                     mut commands: Commands,
                                                     has_disableds: Query<Has<InteractionDisabled>, With<Button>>| {
                                            if click.button == PointerButton::Primary && !has_disableds.get(click.entity).unwrap() {
                                                commands.entity(menu_entity).insert(MenuClicked(text.clone()));
                                                commands.entity(menu_entity).despawn();
                                            }
                                });
                        }
                    });
            }
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
