use bevy::prelude::*;

use crate::{
    bases::{Base, BasetypesAsset, BasetypesHandle, spawn_base},
    constants::ui::*,
    followers::{Follower, FollowerCount, FollowersAsset, FollowersHandle},
    funds::Funds,
    regions::{BasePlot, Location, Region},
    rng::RandomSource,
    suspicion::{MediaSuspicion, PoliceSuspicion},
    text::TextKey,
    ui::{
        UnicodeFontHandle,
        dialog::{Dialog, DialogConfirmed},
        menu::{Menu, MenuClicked, MenuEntry, MenuItem},
    },
};

use super::{
    DisplayFontHandle, FontHandle, MapUi, MeterDisplay, ViewOf, Views, on_label_out, on_label_over,
};

#[derive(Component)]
pub struct RegionSuspicionUi;

#[derive(Component)]
pub struct PoliceSuspicionUi;

#[derive(Component)]
pub struct MediaSuspicionUi;

#[derive(Component)]
struct RegionUi;

#[derive(Component)]
pub struct BasePlotUi;

#[derive(Component)]
pub struct BaseUi {
    follower_list: Entity,
}

#[derive(Component)]
pub struct FollowerListUi;

#[derive(Component)]
pub struct FollowerListBoxUi;

pub fn setup(
    mut commands: Commands,
    map_ui: Single<Entity, With<MapUi>>,
    regions: Query<(Entity, &Region, &Location, &Children)>,
    base_plots: Query<&Location, With<BasePlot>>,
    display_font_handle: Res<DisplayFontHandle>,
    font_handle: Res<FontHandle>,
) {
    for (entity, region, location, children) in regions.iter() {
        commands
            .spawn((
                ChildOf(*map_ui),
                ViewOf(entity),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(location.x),
                    top: percent(location.y),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(px(1.5)),
                    border_radius: BorderRadius::all(px(10)),
                    padding: UiRect::all(px(5)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                UiTransform {
                    translation: Val2::percent(-50.0, -50.0),
                    ..default()
                },
                RegionUi,
                BorderColor::all(BORDER),
                BackgroundColor::from(MENU_BACKGROUND.with_alpha(0.75)),
            ))
            .observe(on_label_over)
            .observe(on_label_out)
            .observe(on_region_click)
            .with_children(|parent| {
                parent.spawn((
                    region.get_text_key(),
                    TextFont::from_font_size(SUB_HEADING).with_font(display_font_handle.0.clone()),
                ));
                parent.spawn((
                    ViewOf(entity),
                    RegionSuspicionUi,
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        column_gap: px(10),
                        display: Display::None,
                        ..default()
                    },
                    children![
                        (
                            TextFont::from_font_size(SMALL).with_font(font_handle.0.clone()),
                            MeterDisplay::<u32> {
                                value: 0,
                                low_threshold: 34,
                                high_threshold: 67,
                            },
                            PoliceSuspicionUi,
                            ViewOf(entity),
                        ),
                        (
                            TextFont::from_font_size(SMALL).with_font(font_handle.0.clone()),
                            MeterDisplay::<u32> {
                                value: 0,
                                low_threshold: 34,
                                high_threshold: 67,
                            },
                            MediaSuspicionUi,
                            ViewOf(entity),
                        )
                    ],
                ));
            });
        for child in children {
            let location = base_plots.get(*child).unwrap();
            commands.spawn((
                ChildOf(*map_ui),
                ViewOf(*child),
                BasePlotUi,
                Node {
                    left: percent(location.x),
                    top: percent(location.y),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                UiTransform {
                    translation: Val2::percent(-50.0, -50.0),
                    ..default()
                },
            ));
        }
    }

    commands.add_observer(on_location_reloaded);
    commands.add_observer(on_spawn_base);
    commands.add_observer(on_follower_count_insert);
}

// INFO: Assume only the location has changed, while none is added or removed.
fn on_location_reloaded(
    event: On<Insert, Location>,
    parts: Query<(&Location, &Views)>,
    mut nodes: Query<&mut Node>,
) {
    let (location, views) = parts.get(event.entity).unwrap();

    for view in &views.0 {
        if let Ok(mut node) = nodes.get_mut(*view) {
            node.left = percent(location.x);
            node.top = percent(location.y);
        }
    }
}

fn on_region_click(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    region_uis: Query<&ViewOf, With<RegionUi>>,
    regions: Query<(&Region, &Children)>,
    base_plots: Query<Has<Children>, With<BasePlot>>,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
) {
    if click.button != PointerButton::Primary {
        return;
    }
    let region_entity = region_uis.get(click.entity).unwrap().0;
    let (region, children) = regions.get(region_entity).unwrap();
    let is_any_base_plot_vacant = children
        .iter()
        .any(|base_plot| base_plots.get(base_plot) == Ok(false));

    let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;
    let iter = base_types
        .iter()
        .filter(|(_, settings)| {
            // TODO: use settings.hidden along with trigger to only show on certain conditions.
            settings.regions.is_empty() || settings.regions.contains(&region.name)
        })
        .map(|(name, _)| MenuItem {
            enabled: is_any_base_plot_vacant,
            text: format!("acquire-{name}").into(),
            tooltip: if is_any_base_plot_vacant {
                format!("acquire-{name}-tooltip").into()
            } else {
                "acquire-tooltip-no-vacant-base-plot".into()
            },
        });
    let entry = MenuEntry::new("menu-region-bases").with_items_iter(iter);

    commands
        .spawn((ChildOf(click.entity), Menu::new().with_entry(entry)))
        .observe(
            move |menu_clicked: On<Add, MenuClicked>,
                  mut commands: Commands,
                  funds: Res<Funds>,
                  menu_clickeds: Query<&MenuClicked>,
                  base_types_handle: Res<BasetypesHandle>,
                  base_types_asset: Res<Assets<BasetypesAsset>>,
                  font_handle: Res<FontHandle>| {
                let menu_clicked = menu_clickeds.get(menu_clicked.entity).unwrap();
                let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;

                if let Some(clicked) = menu_clicked.0.strip_prefix("acquire-") {
                    for (name, settings) in base_types {
                        if name == clicked {
                            #[expect(clippy::cast_precision_loss, reason = "can't be helped")]
                            let entity = commands
                                .spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::top(px(20)),
                                    row_gap: px(20),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    let line = |key, arg, value| {
                                        (
                                            TextKey::new(key).add_arg(arg, value),
                                            TextColor::from(TEXT),
                                            TextFont::from_font_size(LARGE)
                                                .with_font(font_handle.0.clone()),
                                        )
                                    };

                                    parent.spawn(line(
                                        "acquire-basetype-dialog-max-pop",
                                        "count",
                                        settings.max_pop as f64,
                                    ));
                                    parent.spawn(line(
                                        "acquire-basetype-dialog-initial-cost",
                                        "funds",
                                        settings.initial_cost as f64,
                                    ));
                                    parent.spawn(line(
                                        "acquire-basetype-dialog-cost-per-day",
                                        "funds",
                                        settings.cost_per_day as f64,
                                    ));
                                    parent.spawn(line(
                                        "acquire-basetype-dialog-police-suspicion",
                                        "suspicion",
                                        settings.police_suspicion as f64,
                                    ));
                                    parent.spawn(line(
                                        "acquire-basetype-dialog-media-suspicion",
                                        "suspicion",
                                        settings.media_suspicion as f64,
                                    ));
                                })
                                .id();

                            let base_type = name.clone();

                            let mut dialog = Dialog::new()
                                .with_pause()
                                .with_cancel()
                                .with_title(menu_clicked.0.as_str())
                                .with_entity_body(entity);

                            if settings.initial_cost > funds.0 {
                                dialog = dialog.with_confirm_disabled(
                                    TextKey::new("acquire-basetype-dialog-confirm-tooltip")
                                        .add_arg("funds", settings.initial_cost)
                                );
                            }

                            commands
                                .spawn(dialog)
                                .observe(move |_: On<Add, DialogConfirmed>,
                                               commands: Commands,
                                               funds: ResMut<Funds>,
                                               regions: Query<&Children, With<Region>>,
                                               base_plots: Query<Has<Children>, With<BasePlot>>,
                                               base_types_handle: Res<BasetypesHandle>,
                                               base_types_asset: Res<Assets<BasetypesAsset>>,
                                               followers_handle: Res<FollowersHandle>,
                                               followers_asset: Res<Assets<FollowersAsset>>,
                                               random_source: ResMut<RandomSource>| {
                                            spawn_base(commands, funds, region_entity, regions, base_type.clone(), base_plots, base_types_handle, base_types_asset, followers_handle, followers_asset, random_source);
                                });
                        }
                    }
                }
            },
        );
}

fn on_spawn_base(
    event: On<Insert, Base>,
    mut commands: Commands,
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
            TextFont::from_font_size(SMALL).with_font(unicode_font_handle.0.clone()),
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
            BorderColor::all(BORDER),
            BackgroundColor::from(BUTTON_BACKGROUND.with_alpha(0.75)),
        ))
        .observe(on_label_over)
        .observe(on_label_out)
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
                        padding: UiRect::all(px(1)),
                        ..default()
                    },
                    Visibility::Hidden,
                    FollowerListBoxUi,
                    BorderColor::all(BORDER),
                    BackgroundColor::from(BLACK),
                ))
                .add_child(follower_list)
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

pub fn on_follower_count_insert(
    insert: On<Insert, FollowerCount>,
    bases: Query<&Children, With<Base>>,
    followers: Query<(&Follower, &FollowerCount)>,
    follower_counts: Query<&ChildOf, With<FollowerCount>>,
    base_views: Query<&Views, With<Base>>,
    base_uis: Query<&BaseUi>,
    mut follower_list_uis: Query<(&mut Text, &ChildOf), With<FollowerListUi>>,
    mut follower_list_box_uis: Query<&mut Visibility, With<FollowerListBoxUi>>,
) {
    let base = follower_counts.get(insert.entity).unwrap();
    let base_views = base_views.get(base.0).unwrap();
    let follower_list = base_views
        .iter()
        .find_map(|view| base_uis.get(view).ok())
        .unwrap()
        .follower_list;
    let (mut follower_list_text, follower_list_box) =
        follower_list_uis.get_mut(follower_list).unwrap();

    let mut followers: Vec<(Follower, FollowerCount)> = bases
        .get(base.0)
        .unwrap()
        .iter()
        .filter_map(|f| followers.get(f).ok().map(|(f, c)| (*f, *c)))
        .collect();

    followers.sort_unstable_by_key(|(f, _)| *f);

    let new_text = followers.iter().fold(String::new(), |mut text, (f, c)| {
        let iter = std::iter::repeat_n(
            match f {
                Follower::Priest => '☉',
                Follower::Goon => '♁',
                Follower::Minion => '☿',
            },
            **c,
        );

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
}

pub fn update_regional_suspicion(
    regions: Query<
        (&Views, &PoliceSuspicion, &MediaSuspicion),
        (
            With<Region>,
            Or<(Changed<PoliceSuspicion>, Changed<MediaSuspicion>)>,
        ),
    >,
    mut police_suspicion_uis: Query<
        &mut MeterDisplay<u32>,
        (With<PoliceSuspicionUi>, Without<MediaSuspicionUi>),
    >,
    mut media_suspicion_uis: Query<
        &mut MeterDisplay<u32>,
        (With<MediaSuspicionUi>, Without<PoliceSuspicionUi>),
    >,
) {
    for (views, police, media) in regions.iter() {
        for view in &views.0 {
            if let Ok(mut police_suspicion_meter) = police_suspicion_uis.get_mut(*view) {
                police_suspicion_meter.value = police.0;
            }
            if let Ok(mut media_suspicion_meter) = media_suspicion_uis.get_mut(*view) {
                media_suspicion_meter.value = media.0;
            }
        }
    }
}
