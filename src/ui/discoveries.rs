use bevy::{
    prelude::*,
    ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar},
};

use crate::{
    constants::ui::{BORDER, HEADING, NORMAL, SUB_HEADING, TEXT},
    discoveries::{DiscoveriesAsset, DiscoveriesHandle, DiscoveriesResearched},
    text::TextKey,
    ui::{FontHandle, dialog::Dialog, scroll::on_scroll},
};

pub fn open_discoveries_menu(
    mut commands: Commands,
    discoveries_handle: Res<DiscoveriesHandle>,
    discoveries_assets: Res<Assets<DiscoveriesAsset>>,
    discovered: Res<DiscoveriesResearched>,
    font_handle: Res<FontHandle>,
) {
    let discoveries = &discoveries_assets.get(discoveries_handle.0.id()).unwrap().0;

    let discoveries_root = commands
        .spawn(Node {
            width: percent(100),
            ..default()
        })
        .id();
    let mut make_tab = |textkey| {
        let root = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    width: percent(50),
                    ..default()
                },
                ChildOf(discoveries_root),
            ))
            .with_child((
                Node {
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                TextKey::new(textkey),
                TextColor::from(TEXT),
                TextFont::from_font_size(HEADING).with_font(font_handle.clone()),
            ))
            .with_child(Node {
                height: px(1),
                width: percent(100),
                margin: UiRect {
                    top: px(5),
                    bottom: px(10),
                    ..default()
                },
                ..default()
            })
            .id();
        let container = commands
            .spawn((
                ChildOf(root),
                Node {
                    width: percent(100),
                    height: percent(50),
                    ..default()
                },
            ))
            .id();
        let body = commands
            .spawn((
                ChildOf(container),
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    row_gap: px(4),
                    ..default()
                },
            ))
            .observe(on_scroll)
            .id();
        commands
            .spawn((
                ChildOf(container),
                Scrollbar {
                    target: body,
                    orientation: ControlOrientation::Vertical,
                    min_thumb_length: 20.0,
                },
                Node {
                    width: px(5),
                    height: percent(100),
                    border: px(1).all(),
                    margin: px(5).left(),
                    ..default()
                },
                BorderColor::all(BORDER),
            ))
            .with_child((
                CoreScrollbarThumb,
                Node {
                    position_type: PositionType::Absolute,
                    border_radius: BorderRadius::all(px(2)),
                    ..default()
                },
                BackgroundColor::from(BORDER),
            ));
        body
    };
    let available_node = make_tab("discoveries-menu.available");
    let discovered_node = make_tab("discoveries-menu.discovered");

    for (name, discovery) in discoveries {
        // TODO: check required discoveries in available_node, hide if not valid
        // TODO: check funds and research cost in available_node, interaction disabled if not valid
        // TODO: allow selecting available discoveries
        // TODO: learn selected discovery on dialog confirm (if validity checks still pass)
        let parent = if discovered.contains(name) {
            discovered_node
        } else {
            available_node
        };
        commands.spawn((
            ChildOf(parent),
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                border: px(2).all(),
                border_radius: BorderRadius::all(px(10)),
                padding: px(4).all(),
                ..default()
            },
            BorderColor::all(BORDER),
            children![
                (
                    TextKey::new(format!("discovery-{name}")),
                    TextColor::from(TEXT),
                    TextFont::from_font_size(SUB_HEADING).with_font(font_handle.clone()),
                ),
                (
                    TextKey::new(format!("discovery-{name}.desc")),
                    TextColor::from(TEXT),
                    TextFont::from_font_size(NORMAL).with_font(font_handle.clone()),
                ),
            ],
        ));
    }

    // TODO: figure out why confirm and cancel buttons don't show up
    commands.spawn(
        Dialog::new()
            .with_confirm_disabled("discoveries-menu.confirm-tooltip")
            .with_title("discoveries-menu.title")
            .with_entity_body(discoveries_root)
            .with_confirm_label("discoveries-menu.confirm")
            .with_cancel_label("discoveries-menu.cancel"),
    );
}
