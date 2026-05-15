use bevy::{
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar},
};

use crate::{
    constants::ui::{colors::*, fonts::*},
    discoveries::{
        DiscoveriesAsset, DiscoveriesHandle, DiscoveriesResearched, DiscoverySelected,
        DiscoveryVisibility, ResearchPoints, learn_new_discovery,
    },
    funds::Funds,
    text::TextKey,
    ui::{
        FontHandle,
        dialog::{Dialog, DialogCancelled, DialogConfirm, DialogConfirmed},
        scroll::on_scroll,
        tooltip::Tooltip,
    },
};

#[derive(Component)]
struct DiscoveryUi(String);

#[derive(Component)]
struct DiscoveryTextUi;

#[derive(Event)]
struct DiscoveryChanged(String);

pub fn open_discoveries_menu(
    mut commands: Commands,
    discoveries_handle: Res<DiscoveriesHandle>,
    discoveries_assets: Res<Assets<DiscoveriesAsset>>,
    discovered: Res<DiscoveriesResearched>,
    font_handle: Res<FontHandle>,
    secrets: Res<ResearchPoints>,
    funds: Res<Funds>,
) {
    let discoveries = &discoveries_assets.get(discoveries_handle.0.id()).unwrap().0;

    let discoveries_root = commands
        .spawn(Node {
            width: percent(100),
            min_height: px(430),
            max_height: px(430),
            column_gap: px(10),
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
                    margin: UiRect::bottom(px(5)),
                    ..default()
                },
                TextKey::new(textkey),
                TextColor::from(TEXT),
                TextFont::from_font_size(SUB_HEADING).with_font(font_handle.clone()),
            ))
            .id();
        let container = commands
            .spawn((
                ChildOf(root),
                Node {
                    width: percent(100),
                    height: percent(92),
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

    'outer: for (name, discovery) in discoveries {
        let available = !discovered.contains_key(name);
        if available {
            // Check that all required discoveries have already been discovered
            for req in &discovery.requires {
                if !discovered.contains_key(req) {
                    continue 'outer;
                }
            }
        } else {
            // SAFETY: we already know !available, so discovered.contains_key.
            if *discovered.get(name).unwrap() == DiscoveryVisibility::Hidden {
                continue;
            }
        }
        let parent = if available {
            available_node
        } else {
            discovered_node
        };

        let discovery_observer = commands
            .add_observer(
                |event: On<DiscoveryChanged>,
                 mut discoveries: Query<(&mut Node, &DiscoveryUi, &Children)>,
                 mut texts: Query<&mut TextColor, With<DiscoveryTextUi>>| {
                    for (mut node, discovery, children) in &mut discoveries {
                        let mut text = children
                            .iter()
                            .find(|c| texts.contains(*c))
                            .and_then(|c| texts.get_mut(c).ok())
                            .unwrap();
                        if discovery.0 == event.0 {
                            node.border = px(4).all();
                            node.padding = px(4).all();
                            text.set_if_neq(TEXT_HIGHLIGHT.into());
                        } else {
                            node.border = px(2).all();
                            node.padding = px(6).all();
                            text.set_if_neq(TEXT.into());
                        }
                    }
                },
            )
            .id();

        let mut entity_commands = commands.spawn((
            ChildOf(parent),
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                border: px(2).all(),
                border_radius: BorderRadius::all(px(10)),
                padding: px(6).all(),
                ..default()
            },
            BorderColor::all(BORDER),
        ));

        entity_commands.add_child(discovery_observer);

        entity_commands.with_children(|parent| {
            parent.spawn((
                DiscoveryTextUi,
                TextKey::new(format!("discovery-{name}")),
                TextColor::from(TEXT),
                TextFont::from_font_size(SUB_HEADING).with_font(font_handle.clone()),
            ));
            parent.spawn((
                TextKey::new(format!("discovery-{name}.desc")),
                TextColor::from(TEXT),
                TextFont::from_font_size(NORMAL).with_font(font_handle.clone()),
            ));
            parent.spawn(Node {
                height: px(5),
                ..default()
            });
            if available && discovery.funds_cost > 0 {
                parent.spawn((
                    TextKey::new("discoveries-funds-cost").add_arg("funds", discovery.funds_cost),
                    TextColor::from(TEXT),
                    TextFont::from_font_size(SMALL).with_font(font_handle.clone()),
                ));
            }
            if available && discovery.research_cost > 0 {
                parent.spawn((
                    TextKey::new("discoveries-research-cost")
                        .add_arg("points", discovery.research_cost as f64),
                    TextColor::from(TEXT),
                    TextFont::from_font_size(SMALL).with_font(font_handle.clone()),
                ));
            }
        });

        if available {
            entity_commands.insert(DiscoveryUi(name.clone()));
            let funds_cost = discovery.funds_cost;
            let research_cost = discovery.research_cost;
            entity_commands.observe(
                move |click: On<Pointer<Click>>,
                 mut commands: Commands,
                 buttons: Query<(&DiscoveryUi, Has<InteractionDisabled>), With<Button>>| {
                    let (discovery_ui, has_interaction_disabled) = buttons.get(click.entity).unwrap();
                    if click.button == PointerButton::Primary && !has_interaction_disabled {
                        commands.trigger(DiscoveryChanged(discovery_ui.0.clone()));
                        commands.insert_resource(DiscoverySelected(discovery_ui.0.clone(), funds_cost, research_cost));
                        commands.entity(discoveries_root).insert(DialogConfirm(true));
                    }
                },
            );
        }

        let not_enough_points = discovery.research_cost > secrets.0;
        let not_enough_funds = discovery.funds_cost != 0 && discovery.funds_cost > funds.0;

        if available && (not_enough_points || not_enough_funds) {
            entity_commands.insert(InteractionDisabled);

            let mut text_keys: Vec<TextKey> = Vec::new();
            if not_enough_points {
                text_keys.push("discoveries-not-enough-secrets-tooltip".into());
            }
            if not_enough_funds {
                text_keys.push("discoveries-not-enough-funds-tooltip".into());
            }

            entity_commands.insert(Tooltip::new_text_colors(
                text_keys.into_iter().map(|t| (t, TEXT_NEGATIVE)),
            ));
        }
    }

    commands
        .spawn(
            Dialog::new()
                .with_confirm_disabled("discoveries-menu.confirm-tooltip")
                .with_title("discoveries-menu.title")
                .with_entity_body(discoveries_root)
                .with_confirm_label("discoveries-menu.confirm")
                .with_cancel_label("discoveries-menu.cancel")
                .with_pause(),
        )
        .observe(move |_: On<Add, DialogConfirmed>, mut commands: Commands| {
            commands.run_system_cached(learn_new_discovery);
            commands.remove_resource::<DiscoverySelected>();
        })
        .observe(move |_: On<Add, DialogCancelled>, mut commands: Commands| {
            commands.remove_resource::<DiscoverySelected>();
        });
}
