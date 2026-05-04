use bevy::prelude::*;

use crate::{
    bases::{Base, BasetypesAsset, BasetypesHandle, spawn_base},
    constants::ui::*,
    followers::Follower,
    funds::Funds,
    regions::{BasePlot, Location, Region},
    rng::RandomSource,
    suspicion::{MediaSuspicion, PoliceSuspicion},
    text::TextKey,
    ui::{
        BaseUi, FollowerList, UnicodeFontHandle,
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
                    border: UiRect::all(px(1)),
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
    commands.add_observer(on_changed_follower::<Insert>);
    commands.add_observer(on_changed_follower::<Replace>);
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
            text: format!("acquire-{}", name).into(),
            tooltip: if is_any_base_plot_vacant {
                format!("acquire-{}-tooltip", name).into()
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
                    for (name, settings) in base_types.iter() {
                        if name == clicked {
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
                                               random_source: ResMut<RandomSource>| {
                                            spawn_base(commands, funds, region_entity, regions, base_type.clone(), base_plots, base_types_handle, base_types_asset, random_source);
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
    base_plots: Query<(&ChildOf, &Views), With<BasePlot>>,
    regions: Query<&Views, With<Region>>,
    mut region_suspicion_uis: Query<&mut Node, With<RegionSuspicionUi>>,
    base_plot_uis: Query<&BasePlotUi>,
    font_handle: Res<FontHandle>,
) {
    let (base_plot, base_type) = bases.get(event.entity).unwrap();
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

    commands
        .spawn((
            ChildOf(base_plot_ui),
            ViewOf(event.entity),
            BaseUi,
            Node {
                flex_direction: FlexDirection::Column,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                padding: UiRect::horizontal(px(2)),
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(WHITE),
            BackgroundColor::from(BUTTON_BACKGROUND.with_alpha(0.75)),
        ))
        .observe(on_label_over)
        .observe(on_label_out)
        .with_children(|parent| {
            parent.spawn((
                TextKey::new(format!("basetype-{}", base_type.0)),
                TextFont::from_font_size(NORMAL).with_font(font_handle.0.clone()),
                TextLayout::new_with_justify(Justify::Center),
            ));
            parent.spawn((
                FollowerList,
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));
        });
}

fn on_changed_follower<E: EntityEvent>(
    event: On<E, Follower>,
    mut commands: Commands,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    followers: Query<&Follower>,
    base_views: Query<&Views, With<Base>>,
    base_uis: Query<&BaseUi>,
    follower_lists: Query<&FollowerList>,
    unicode_font_handle: Res<UnicodeFontHandle>,
) {
    let base = parents.get(event.event_target()).unwrap().0;
    let base_views = base_views.get(base).unwrap();
    let base_ui = base_views
        .iter()
        .find(|view| base_uis.contains(*view))
        .unwrap();
    let follower_list = children
        .get(base_ui)
        .unwrap()
        .iter()
        .find(|fl| follower_lists.contains(*fl))
        .unwrap();

    commands.entity(follower_list).despawn_children();

    let mut followers: Vec<Follower> = children
        .get(base)
        .unwrap()
        .iter()
        .map(|follower| *followers.get(follower).unwrap())
        .collect();

    followers.sort_unstable();

    let text_font = TextFont::from_font_size(SMALL).with_font(unicode_font_handle.0.clone());

    let bundles: Vec<_> = followers
        .iter()
        .map(|f| {
            let text = match f {
                Follower::Priest => Text::new("☉"),
                Follower::Goon => Text::new("♁"),
                Follower::Minion => Text::new("☿"),
            };
            (ChildOf(follower_list), text, text_font.clone())
        })
        .collect();

    commands.spawn_batch(bundles);
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
        for view in views.0.iter() {
            if let Ok(mut police_suspicion_meter) = police_suspicion_uis.get_mut(*view) {
                police_suspicion_meter.value = police.0;
            }
            if let Ok(mut media_suspicion_meter) = media_suspicion_uis.get_mut(*view) {
                media_suspicion_meter.value = media.0;
            }
        }
    }
}
