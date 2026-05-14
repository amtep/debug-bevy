use bevy::prelude::*;

use crate::{
    bases::{Base, BasetypesAsset, BasetypesHandle},
    constants::ui::{colors::*, fonts::SMALL},
    followers::{Follower, FollowerCount, FollowersAsset, FollowersHandle},
    regions::{BasePlot, Region},
    state::GameState,
    tasks::{Task, TasksAsset, TasksHandle},
    text::TextKey,
    ui::{
        BasePlotUi, RegionSuspicionUi, UnicodeFontHandle,
        menu::{Menu, MenuClicked, MenuEntry, MenuItem},
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

            MenuEntry::new(
                TextKey::new(format!("follower-type-{}", f.0)).add_arg("count", c.0 as f64),
            )
            .with_items_iter(task_iter)
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
                  followers: Query<(&Follower, &Children)>,
                  tasks: Query<&Task>| {
                let MenuClicked(heading, item) = menu_clickeds.get(menu_clicked.entity).unwrap();

                if let Some(follower) = heading.strip_prefix("follower-type-")
                    && let Some(task) = item.strip_prefix("task-")
                {
                    let children = bases.get(base_entity).unwrap();
                    let follower_children = children
                        .iter()
                        .find_map(|child| {
                            followers
                                .get(child)
                                .ok()
                                .and_then(|f| (**f.0 == follower).then_some(f.1))
                        })
                        .unwrap();
                    let task_entity = follower_children
                        .iter()
                        .find(|c| tasks.contains(*c))
                        .unwrap();
                    commands.entity(task_entity).insert(Task(task.to_owned()));
                }
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
