use bevy::prelude::*;

use crate::{
    bases::{Base, BasetypesAsset, BasetypesHandle},
    constants::ui::*,
    followers::{Follower, FollowerCount},
    regions::{BasePlot, Region},
    text::TextKey,
    ui::{BasePlotUi, RegionSuspicionUi, UnicodeFontHandle, menu::Menu, tooltip::Tooltip},
};

use super::{ViewOf, Views, on_label_out, on_label_over};

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
            BorderColor::all(BORDER),
            BackgroundColor::from(BUTTON_BACKGROUND.with_alpha(0.75)),
        ))
        .observe(on_label_over)
        .observe(on_label_out)
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
    bases: Query<&Base>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let base = base_uis.get(click.entity).unwrap().0;
    let base = &bases.get(base).unwrap().0;

    commands
        .entity(click.entity)
        .with_child(Menu::new().with_title(format!("basetype-{base}")));
}

pub fn on_follower_count_insert(
    insert: On<Insert, FollowerCount>,
    mut commands: Commands,
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
        let iter = std::iter::repeat_n(f.to_symbol(), **c);

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
                let f: &str = f.into();
                #[allow(clippy::cast_precision_loss)]
                TextKey::new("follower-list-tooltip")
                    .add_arg("count", **c as f64)
                    .add_arg("follower-type", f)
            }),
        ));
}
