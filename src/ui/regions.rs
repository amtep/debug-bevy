use bevy::prelude::*;

use crate::{
    bases::{BasetypesAsset, BasetypesHandle, spawn_base},
    constants::ui::{colors::*, fonts::*},
    discoveries::DiscoveriesResearched,
    funds::Funds,
    regions::{BasePlot, Location, Region},
    suspicion::{MediaSuspicion, PoliceSuspicion},
    text::TextKey,
    ui::{
        BasePlotUi, EmojiFontHandle, MonoFontHandle, RegionSuspicionUi,
        bases::{on_follower_count_insert, on_spawn_base},
        dialog::{Dialog, DialogConfirmed},
        menu::{Menu, MenuClicked, MenuEntry, MenuItem},
        tooltip::Tooltip,
    },
};

use super::{DisplayFontHandle, FontHandle, MapUi, MeterDisplay, ViewOf, Views};

pub fn plugin(app: &mut App) {
    app.add_observer(on_location_reloaded)
        .add_observer(on_spawn_base)
        .add_observer(on_follower_count_insert);
}

#[derive(Component)]
pub struct PoliceSuspicionUi;

#[derive(Component)]
pub struct MediaSuspicionUi;

#[derive(Component)]
struct RegionUi;

pub fn setup(
    mut commands: Commands,
    map_ui: Single<Entity, With<MapUi>>,
    regions: Query<(Entity, &Region, &Location, &Children)>,
    base_plots: Query<&Location, With<BasePlot>>,
    display_font_handle: Res<DisplayFontHandle>,
    mono_font_handle: Res<MonoFontHandle>,
    emoji_font_handle: Res<EmojiFontHandle>,
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
                Button,
                RegionUi,
                BorderColor::all(BORDER),
                BackgroundColor::from(BUTTON_BACKGROUND.with_alpha(OVERLAY_ALPHA)),
            ))
            .observe(on_region_click)
            .with_children(|parent| {
                parent.spawn((
                    region.get_text_key(),
                    TextFont::from_font_size(SUB_HEADING).with_font(display_font_handle.clone()),
                ));
                parent
                    .spawn((
                        ViewOf(entity),
                        RegionSuspicionUi,
                        Node {
                            top: percent(100),
                            position_type: PositionType::Absolute,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            display: Display::None,
                            margin: UiRect::top(px(4)),
                            column_gap: px(5),
                            border: UiRect::bottom(px(1)),
                            ..default()
                        },
                        BorderColor::all(BORDER),
                        BackgroundColor::from(DARK_OVERLAY),
                        children![
                            (
                                Node {
                                    min_width: px(38),
                                    justify_content: JustifyContent::SpaceBetween,
                                    ..default()
                                },
                                Tooltip::new_text("police-suspicion-tooltip"),
                                children![
                                    (
                                        Text::new('🚨'),
                                        TextColor::from(TEXT),
                                        TextFont::from_font_size(TINY)
                                            .with_font(emoji_font_handle.clone()),
                                    ),
                                    (
                                        TextFont::from_font_size(SMALL)
                                            .with_font(mono_font_handle.clone()),
                                        MeterDisplay::<u32> {
                                            value: 0,
                                            low_threshold: 334,
                                            high_threshold: 667,
                                        },
                                        PoliceSuspicionUi,
                                        ViewOf(entity),
                                    )
                                ]
                            ),
                            (
                                Node {
                                    min_width: px(38),
                                    justify_content: JustifyContent::SpaceBetween,
                                    ..default()
                                },
                                Tooltip::new_text("media-suspicion-tooltip"),
                                children![
                                    (
                                        Text::new('📺'),
                                        TextColor::from(TEXT),
                                        TextFont::from_font_size(TINY)
                                            .with_font(emoji_font_handle.clone()),
                                    ),
                                    (
                                        TextFont::from_font_size(SMALL)
                                            .with_font(mono_font_handle.clone()),
                                        MeterDisplay::<u32> {
                                            value: 0,
                                            low_threshold: 334,
                                            high_threshold: 667,
                                        },
                                        MediaSuspicionUi,
                                        ViewOf(entity)
                                    ),
                                ]
                            )
                        ],
                    ))
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
}

// INFO: Assume only the location has changed, while none is added or removed.
fn on_location_reloaded(
    event: On<Insert, Location>,
    parts: Query<(&Location, &Views)>,
    mut nodes: Query<&mut Node>,
) {
    let Ok((location, views)) = parts.get(event.entity) else {
        // UI is not ready yet
        return;
    };

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
    discoveries_researched: Res<DiscoveriesResearched>,
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
            (settings.regions.is_empty() || settings.regions.contains(&region.name))
                && settings
                    .requires_discovery
                    .as_ref()
                    .is_none_or(|discovery| discoveries_researched.contains(discovery))
        })
        .map(|(name, _)| MenuItem {
            enabled: is_any_base_plot_vacant,
            text: format!("acquire-{name}").into(),
            tooltip: if is_any_base_plot_vacant {
                format!("acquire-{name}-tooltip").into()
            } else {
                "acquire-no-vacant-base-plot-tooltip".into()
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

                if let Some(clicked) = menu_clicked.1.strip_prefix("acquire-") {
                    for (name, settings) in base_types {
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
                                                .with_font(font_handle.clone()),
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
                                        .add_arg("funds", settings.initial_cost),
                                );
                            }

                            commands.spawn(dialog).observe(
                                move |_: On<Add, DialogConfirmed>, mut commands: Commands| {
                                    commands.run_system_cached_with(
                                        spawn_base,
                                        (region_entity, base_type.clone()),
                                    );
                                },
                            );
                        }
                    }
                }
            },
        );
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
            } else if let Ok(mut media_suspicion_meter) = media_suspicion_uis.get_mut(*view) {
                media_suspicion_meter.value = media.0;
            }
        }
    }
}
