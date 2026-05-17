use bevy::{
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{SliderRange, SliderValue},
};

use crate::{
    bases::{Base, BasetypesAsset, BasetypesHandle, transfer_follower_costs, transfer_followers},
    constants::{
        files::TEXTURE_EARTH_BACKGROUND,
        ui::{
            colors::*,
            fonts::{NORMAL, SMALL},
        },
    },
    followers::{Follower, FollowerCount, FollowersAsset, FollowersHandle},
    funds::Funds,
    regions::{BasePlot, Location, Region},
    state::GameState,
    suspicion::{IntelligenceSuspicion, SuspicionType},
    tasks::{Task, TasksAsset, TasksHandle},
    text::TextKey,
    ui::{
        BasePlotUi, EmojiFontHandle, FontHandle, MonoFontHandle, RegionSuspicionUi, Selected,
        UnicodeFontHandle,
        dialog::{Dialog, DialogConfirm, DialogConfirmed},
        menu::{Menu, MenuClicked, MenuEntry, MenuItem},
        sliders::{Slider, SliderText},
        suspicion_type_color, suspicion_type_icon,
        tooltip::Tooltip,
    },
};

use super::{ViewOf, Views};

#[derive(Component)]
pub struct BaseUi {
    follower_list: Entity,
}

#[derive(Component)]
pub struct FollowerListUi;

#[derive(Component)]
pub struct FollowerListBoxUi;

#[derive(Component)]
struct FollowerSliderUi;

#[derive(Component)]
struct FollowerTransferBaseSelectorUi(Entity);

#[derive(Component)]
struct FollowerTransferCostFunds;

#[derive(Component)]
struct FollowerTransferCostSuspicion;

pub fn on_spawn_base(
    event: On<Insert, Base>,
    mut commands: Commands,
    state: Res<State<GameState>>,
    bases: Query<(&ChildOf, &Base)>,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    base_plots: Query<(&ChildOf, &Views), With<BasePlot>>,
    regions: Query<&Views, With<Region>>,
    mut region_suspicion_uis: Query<&mut Node, With<RegionSuspicionUi>>,
    base_plot_uis: Query<&BasePlotUi>,
    asset_server: Res<AssetServer>,
    unicode_font_handle: Res<UnicodeFontHandle>,
) {
    if *state != GameState::Main {
        return;
    }

    let (base_plot, base) = bases.get(event.entity).unwrap();
    let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;
    let base_type = base_types.get(&base.0).unwrap();
    let (region, base_plot_views) = base_plots.get(base_plot.0).unwrap();
    let region_views = regions.get(region.0).unwrap();
    let mut region_suspicion_ui_node = region_views
        .iter()
        .find(|view| region_suspicion_uis.contains(*view))
        .map(|view| region_suspicion_uis.get_mut(view).unwrap())
        .unwrap();

    if region_suspicion_ui_node.display == Display::None {
        region_suspicion_ui_node.display = Display::Flex;
    }

    let base_plot_ui = base_plot_views
        .iter()
        .find(|view| base_plot_uis.contains(*view))
        .unwrap();

    let follower_list = commands
        .spawn((
            TextFont::from_font_size(SMALL).with_font(unicode_font_handle.clone()),
            Text::default(),
            FollowerListUi,
        ))
        .id();

    commands
        .spawn((
            ChildOf(base_plot_ui),
            ViewOf(event.entity),
            BaseUi { follower_list },
            Node {
                flex_direction: FlexDirection::Column,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                ..default()
            },
            Button,
            BorderColor::all(BORDER),
            BackgroundColor::from(BUTTON_BACKGROUND.with_alpha(OVERLAY_ALPHA)),
        ))
        .observe(on_base_click)
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: px(32),
                    height: px(32),
                    ..default()
                },
                ImageNode {
                    image: asset_server.load(format!("textures/{}.png", base.0)),
                    color: color(&base_type.color).into(),
                    ..default()
                },
            ));

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: percent(100),
                        margin: UiRect::top(px(2)),
                        border: UiRect::bottom(px(1)),
                        padding: UiRect::horizontal(px(1)),
                        ..default()
                    },
                    Visibility::Hidden,
                    FollowerListBoxUi,
                    BorderColor::all(BORDER),
                    BackgroundColor::from(DARK_OVERLAY),
                ))
                .add_child(follower_list)
                .observe(|mut click: On<Pointer<Press>>| {
                    click.propagate(false);
                })
                .observe(|mut click: On<Pointer<Click>>| {
                    click.propagate(false);
                })
                .observe(|mut over: On<Pointer<Over>>| {
                    over.propagate(false);
                })
                .observe(|mut out: On<Pointer<Out>>| {
                    out.propagate(false);
                });
        });
}

fn on_base_click(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    base_uis: Query<&ViewOf, With<BaseUi>>,
    bases: Query<(&Children, &Base)>,
    followers: Query<(&Follower, &FollowerCount, &Children)>,
    tasks: Query<&Task>,
    task_handle: Res<TasksHandle>,
    task_assets: Res<Assets<TasksAsset>>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let base_entity = base_uis.get(click.entity).unwrap().0;
    let (children, base) = bases.get(base_entity).unwrap();
    let task_settings = &task_assets.get(task_handle.0.id()).unwrap().0;

    let follower_iter = children
        .iter()
        .filter_map(|child| followers.get(child).ok())
        .filter(|(_, c, _)| c.0 != 0)
        .map(|(f, c, children)| {
            let current_task = children.iter().find_map(|c| tasks.get(c).ok()).unwrap();
            let task_iter = task_settings
                .iter()
                .filter(|(_, v)| v.follower_types.contains(f))
                .map(|(k, _)| {
                    let enabled = &current_task.0 != k;
                    let tooltip = if enabled {
                        TextKey::new(format!("switch-task-{k}-tooltip"))
                    } else {
                        TextKey::new("switch-task-current-task-tooltip")
                    };
                    MenuItem {
                        enabled,
                        text: TextKey::new(format!("task-{k}")),
                        tooltip,
                    }
                });

            let entry = MenuEntry::new(
                TextKey::new(format!("follower-type-{}", f.0)).add_arg("count", c.0 as f64),
            );
            // another base exists for transfer
            if bases.count() == 1 {
                entry.with_items_iter(task_iter)
            } else {
                entry.with_items_iter(task_iter.chain(std::iter::once(MenuItem {
                    enabled: true,
                    text: TextKey::new("follower-transfer"),
                    tooltip: TextKey::new("follower-transfer-tooltip"),
                })))
            }
        });

    commands
        .spawn((
            ChildOf(click.entity),
            Menu::new()
                .with_title(format!("basetype-{}", base.0))
                .with_entries_iter(follower_iter),
        ))
        .observe(
            move |menu_clicked: On<Add, MenuClicked>,
                  menu_clickeds: Query<&MenuClicked>,
                  mut commands: Commands,
                  bases: Query<&Children, With<Base>>,
                  followers: Query<(&Follower, &FollowerCount, &Children)>,
                  tasks: Query<&Task>| {
                let MenuClicked(heading, item) = menu_clickeds.get(menu_clicked.entity).unwrap();

                if let Some(follower) = heading.strip_prefix("follower-type-") {
                    let children = bases.get(base_entity).unwrap();
                    if let Some(task) = item.strip_prefix("task-") {
                        let follower_children = children
                            .iter()
                            .find_map(|child| {
                                followers
                                    .get(child)
                                    .ok()
                                    .and_then(|f| (**f.0 == follower).then_some(f.2))
                            })
                            .unwrap();
                        let task_entity = follower_children
                            .iter()
                            .find(|c| tasks.contains(*c))
                            .unwrap();
                        commands.entity(task_entity).insert(Task(task.to_owned()));
                    } else if item == "follower-transfer" {
                        let (follower_entity, follower_count) = children
                            .iter()
                            .find_map(|child| {
                                followers
                                    .get(child)
                                    .ok()
                                    .and_then(|f| (**f.0 == follower).then_some((child, f.1)))
                            })
                            .unwrap();
                        commands.run_system_cached_with(
                            transfer_followers_dialog,
                            (
                                base_entity,
                                follower_entity,
                                follower.to_owned(),
                                *follower_count,
                            ),
                        );
                    }
                }
            },
        );
}

fn transfer_followers_dialog(
    In((base_entity, follower_entity, follower, follower_count)): In<(
        Entity,
        Entity,
        String,
        FollowerCount,
    )>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bases: Query<(Entity, &Base, &ChildOf, &Children)>,
    follower_counts: Query<&FollowerCount>,
    base_plots: Query<&Location, With<BasePlot>>,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    font_handle: Res<FontHandle>,
    mono_font_handle: Res<MonoFontHandle>,
    emoji_font_handle: Res<EmojiFontHandle>,
) {
    let entity = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            margin: px(10).vertical(),
            ..default()
        })
        .id();

    commands
        .spawn((
            ChildOf(entity),
            Node {
                width: px(720),
                height: px(400),
                ..default()
            },
            ImageNode {
                image: asset_server.load(TEXTURE_EARTH_BACKGROUND),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
        ))
        .with_children(|parent| {
            let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;

            for (base_e, base, base_plot_entity, children) in &bases {
                let location = base_plots.get(base_plot_entity.0).unwrap();
                let max_follower_count = base_types.get(&base.0).unwrap().max_follower_count;
                let current_follower_count = children
                    .iter()
                    .filter_map(|c| follower_counts.get(c).ok().map(|c| c.0))
                    .sum::<usize>();

                let tooltip = if base_e == base_entity {
                    Tooltip::new_text_colors([
                        (
                            TextKey::new("follower-transfer-source-base"),
                            TEXT_HIGHLIGHT,
                        ),
                        (
                            TextKey::new("follower-transfer-current-follower-count")
                                .add_arg("count", current_follower_count as f64),
                            TEXT,
                        ),
                        (
                            TextKey::new("follower-transfer-maximum-follower-count")
                                .add_arg("count", max_follower_count as f64),
                            TEXT,
                        ),
                    ])
                } else if current_follower_count == max_follower_count {
                    Tooltip::new_text_colors([
                        (TextKey::new("follower-transfer-full-base"), TEXT_NEGATIVE),
                        (
                            TextKey::new("follower-transfer-current-follower-count")
                                .add_arg("count", current_follower_count as f64),
                            TEXT,
                        ),
                        (
                            TextKey::new("follower-transfer-maximum-follower-count")
                                .add_arg("count", max_follower_count as f64),
                            TEXT,
                        ),
                    ])
                } else {
                    Tooltip::new_texts([
                        TextKey::new("follower-transfer-current-follower-count")
                            .add_arg("count", current_follower_count as f64),
                        TextKey::new("follower-transfer-maximum-follower-count")
                            .add_arg("count", max_follower_count as f64),
                    ])
                };

                let remaining_capacity = max_follower_count - current_follower_count;

                let mut entity_commands = parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(location.x),
                        top: percent(location.y),
                        width: px(25),
                        height: px(25),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(4)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    UiTransform {
                        translation: Val2::percent(-50.0, -50.0),
                        ..default()
                    },
                    Button,
                    BorderColor::all(BORDER),
                    BackgroundColor::from(BUTTON_BACKGROUND),
                    FollowerTransferBaseSelectorUi(base_e),
                    tooltip,
                ));

                if base_e == base_entity || remaining_capacity == 0 {
                    entity_commands.insert(InteractionDisabled);
                }

                entity_commands
                    .with_child((
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        ImageNode {
                            image: asset_server.load(format!("textures/{}.png", base.0)),
                            image_mode: NodeImageMode::Stretch,
                            color: (if base_e == base_entity {
                                RED
                            } else if remaining_capacity == 0 {
                                GREY
                            } else {
                                WHITE
                            })
                            .into(),
                            ..default()
                        },
                    ))
                    .observe(
                        move |click: On<Pointer<Click>>,
                              mut commands: Commands,
                              follower_transfer_base_selector_uis: Query<
                            (Entity, &Children, Has<InteractionDisabled>),
                            With<FollowerTransferBaseSelectorUi>,
                        >,
                              mut image_nodes: Query<&mut ImageNode>,
                              follower_slider_ui: Single<
                            (Entity, &SliderRange),
                            With<FollowerSliderUi>,
                        >| {
                            if click.button == PointerButton::Primary
                                && !follower_transfer_base_selector_uis
                                    .get(click.entity)
                                    .unwrap()
                                    .2
                            {
                                for (e, children, has_interaction_disabled) in
                                    &follower_transfer_base_selector_uis
                                {
                                    if !has_interaction_disabled {
                                        commands.entity(e).remove::<Selected>();
                                        image_nodes
                                            .get_mut(*children.first().unwrap())
                                            .unwrap()
                                            .color = WHITE.into();
                                    }
                                }
                                let child = follower_transfer_base_selector_uis
                                    .get(click.entity)
                                    .unwrap()
                                    .1
                                    .first()
                                    .unwrap();
                                image_nodes.get_mut(*child).unwrap().color = GREEN.into();
                                commands.entity(click.entity).insert(Selected);
                                let max = (follower_count.0).min(remaining_capacity) as f32;
                                commands.entity(entity).insert(DialogConfirm::Enable);
                                commands
                                    .entity(follower_slider_ui.0)
                                    .insert(SliderRange::from_range(0.0..=max));
                                commands
                                    .entity(follower_slider_ui.0)
                                    .insert(SliderValue(max));
                            }
                        },
                    );
            }
        });

    commands
        .spawn((
            ChildOf(entity),
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                margin: px(5).top(),
                column_gap: px(20),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextKey::new("follower-transfer-number")
                    .add_arg("follower-type", follower.as_str())
                    .add_arg("count", follower_count.0 as f64),
                TextColor::from(WHITE),
                TextFont::from_font_size(NORMAL).with_font(font_handle.clone()),
            ));

            let follower = follower.clone();

            let slider = parent
                .spawn((
                    FollowerSliderUi,
                    Slider::new(false).with_major_axis_size(px(150)),
                    SliderValue(follower_count.0 as f32),
                    SliderRange::new(0.0, follower_count.0 as f32),
                ))
                .observe(
                    move |insert: On<Insert, SliderValue>,
                          slider_values: Query<&SliderValue>,
                          mut commands: Commands,
                          funds: Res<Funds>,
                          parents: Query<&ChildOf>,
                          follower_transfer_base_selector_ui: Option<
                        Single<&FollowerTransferBaseSelectorUi, With<Selected>>,
                    >,
                          mut funds_text_key: Single<
                        &mut TextKey,
                        (
                            With<FollowerTransferCostFunds>,
                            Without<FollowerTransferCostSuspicion>,
                        ),
                    >,
                          mut suspicion_text_key: Single<
                        &mut TextKey,
                        (
                            With<FollowerTransferCostSuspicion>,
                            Without<FollowerTransferCostFunds>,
                        ),
                    >| {
                        #[allow(clippy::cast_sign_loss)]
                        #[allow(clippy::cast_possible_truncation)]
                        if let Some(selector_ui) = follower_transfer_base_selector_ui {
                            let value = slider_values.get(insert.entity).unwrap();
                            let (funds_change, intel) = transfer_follower_costs(
                                base_entity,
                                selector_ui.0,
                                follower.clone(),
                                value.0 as usize,
                                parents,
                            );
                            if funds.0 + funds_change < 0 {
                                let text_key =
                                    TextKey::new("follower-transfer-confirm-funds-tooltip")
                                        .add_arg("funds", -funds_change as f64);
                                commands
                                    .entity(entity)
                                    .insert(DialogConfirm::Disable(Some(text_key)));
                            }
                            funds_text_key.replace_arg("funds", funds_change as f64);
                            suspicion_text_key.replace_arg("amount", intel as f64);
                        }
                    },
                )
                .id();

            let slider_text = parent
                .spawn((
                    Node {
                        min_width: px(50),
                        margin: px(2).top(),
                        ..default()
                    },
                    TextKey::new("follower-count").add_arg("count", 0.0),
                    TextColor::from(TEXT),
                    TextFont::from_font_size(NORMAL).with_font(mono_font_handle.clone()),
                ))
                .id();

            parent.spawn((
                Node {
                    min_width: px(60),
                    margin: px(2).top(),
                    ..default()
                },
                FollowerTransferCostFunds,
                TextKey::new("funds-change-display").add_arg("funds", 0.0),
                TextColor::from(TEXT_FUNDS),
                TextFont::from_font_size(NORMAL).with_font(mono_font_handle.clone()),
            ));

            parent
                .spawn(Node {
                    margin: px(2).top(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(suspicion_type_icon(SuspicionType::Intelligence)),
                        TextColor::from(suspicion_type_color(SuspicionType::Intelligence)),
                        TextFont::from_font_size(SMALL).with_font(emoji_font_handle.clone()),
                    ));
                    parent.spawn((
                        Node {
                            min_width: px(50),
                            ..default()
                        },
                        FollowerTransferCostSuspicion,
                        TextKey::new("suspicion-change").add_arg("amount", 0.0),
                        TextColor::from(TEXT),
                        TextFont::from_font_size(NORMAL).with_font(mono_font_handle.clone()),
                    ));
                });

            parent
                .commands()
                .entity(slider)
                .insert(SliderText::TextKey(slider_text, "count"));
        });

    commands
        .spawn(
            Dialog::new()
                .with_title(
                    TextKey::new("follower-transfer-title")
                        .add_arg("follower-type", follower.as_str())
                        .add_arg("count", follower_count.0 as f64),
                )
                .with_entity_body(entity)
                .with_cancel()
                .with_max_height(percent(80))
                .with_max_width(percent(80))
                .with_confirm_disabled("follower-transfer-confirm-tooltip")
                .with_confirm_label("follower-transfer-confirm")
                .with_pause(),
        )
        .observe(
            #[allow(clippy::cast_sign_loss)]
            #[allow(clippy::cast_possible_truncation)]
            move |_: On<Add, DialogConfirmed>,
                  mut commands: Commands,
                  parents: Query<&ChildOf>,
                  mut funds: ResMut<Funds>,
                  mut intel_suspicion: ResMut<IntelligenceSuspicion>,
                  slider_value: Single<&SliderValue, With<FollowerSliderUi>>,
                  selected: Single<&FollowerTransferBaseSelectorUi, With<Selected>>| {
                if slider_value.0 == 0.0 {
                    return;
                }

                let (funds_change, intel) = transfer_follower_costs(
                    base_entity,
                    selected.0,
                    follower.clone(),
                    slider_value.0 as usize,
                    parents,
                );

                funds.0 += funds_change;
                intel_suspicion.0 += intel;

                commands.run_system_cached_with(
                    transfer_followers,
                    (
                        selected.0,
                        follower_entity,
                        follower.clone(),
                        slider_value.0 as usize,
                    ),
                );
            },
        );
}

pub fn on_follower_count_insert(
    insert: On<Insert, FollowerCount>,
    mut commands: Commands,
    state: Res<State<GameState>>,
    bases: Query<&Children, With<Base>>,
    followers: Query<(&Follower, &FollowerCount)>,
    follower_counts: Query<&ChildOf, With<FollowerCount>>,
    base_views: Query<&Views, With<Base>>,
    base_uis: Query<&BaseUi>,
    mut follower_list_uis: Query<(&mut Text, &ChildOf), With<FollowerListUi>>,
    mut follower_list_box_uis: Query<&mut Visibility, With<FollowerListBoxUi>>,
    followers_handle: Res<FollowersHandle>,
    followers_assets: Res<Assets<FollowersAsset>>,
) {
    if *state != GameState::Main {
        return;
    }

    let base = follower_counts.get(insert.entity).unwrap();
    let base_views = base_views.get(base.0).unwrap();
    let follower_list = base_views
        .iter()
        .find_map(|view| base_uis.get(view).ok())
        .unwrap()
        .follower_list;
    let (mut follower_list_text, follower_list_box) =
        follower_list_uis.get_mut(follower_list).unwrap();

    let followers: Vec<(&Follower, FollowerCount)> = bases
        .get(base.0)
        .unwrap()
        .iter()
        .filter_map(|f| followers.get(f).ok().map(|(f, c)| (f, *c)))
        .collect();

    let followers_settings = &followers_assets.get(followers_handle.0.id()).unwrap().0;

    let new_text = followers.iter().fold(String::new(), |mut text, (f, c)| {
        let iter = std::iter::repeat_n(followers_settings.get(&f.0).unwrap().symbol, **c);
        text.extend(iter);
        text
    });

    let mut follower_list_box_visibility =
        follower_list_box_uis.get_mut(follower_list_box.0).unwrap();
    if new_text.is_empty() {
        *follower_list_box_visibility = Visibility::Hidden;
    } else {
        *follower_list_box_visibility = Visibility::Inherited;
    }

    follower_list_text.0 = new_text;

    commands
        .entity(follower_list_box.0)
        .insert(Tooltip::new_texts(
            followers.iter().filter(|(_, c)| **c != 0).map(|(f, c)| {
                TextKey::new("follower-list-tooltip")
                    .add_arg("count", **c as f64)
                    .add_arg("follower-type", f.to_string())
            }),
        ));
}
