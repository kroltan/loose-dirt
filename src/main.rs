mod tilemap;

use std::{cmp::Ordering, ops::Range};

use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseWheel, ElementState},
    math::Vec3Swizzles,
    prelude::*,
    render::camera::WindowOrigin,
};
use rand::Rng;
use tilemap::{
    DownNeighbour, LeftNeighbour, Material, RightNeighbour, TilePosition, Tilemap, TilemapPlugin,
    UpNeighbour,
};

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
const DOT_SIZE: usize = 8;
const BRUSH_SIZE: Range<usize> = 0..4;
const PALETTE: &'static [(Element, &'static str, KeyCode)] = &[
    (Element::Rock, "Rock", KeyCode::R),
    (Element::Water, "Water", KeyCode::W),
    (Element::Sand(0), "Sand", KeyCode::S),
];

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, StageLabel)]
enum GameStage {
    Interact,
    Run,
    Tally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Element {
    Air,
    Rock,
    Water,
    Sand(u8),
}

#[derive(Debug)]
struct Brush {
    size: usize,
    paint: Element,
}

#[derive(Debug)]
struct TutorialTimer {
    show: Timer,
    animate: Timer,
}

#[derive(Debug)]
struct PaletteItem {
    paint: Element,
    hotkey: KeyCode,
}

struct BrushSlider;

struct ViewCamera;

struct TutorialWindow;

fn main() {
    let (width, height) = {
        fn fit(value: f32) -> usize {
            value as usize / DOT_SIZE
        }

        (fit(WINDOW_WIDTH), fit(WINDOW_HEIGHT))
    };

    App::build()
        .insert_resource(WindowDescriptor {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            resizable: true,
            title: "Loose Dirt".to_owned(),
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Brush {
            size: 1,
            paint: PALETTE[0].0,
        })
        .insert_resource(TutorialTimer {
            show: Timer::from_seconds(5.0, false),
            animate: Timer::from_seconds(0.5, false),
        })
        .add_stage_after(
            CoreStage::Update,
            GameStage::Interact,
            SystemStage::parallel(),
        )
        .add_stage_after(GameStage::Interact, GameStage::Run, SystemStage::parallel())
        .add_stage_after(GameStage::Run, GameStage::Tally, SystemStage::parallel())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin::<Element>::new(
            width,
            height,
            DOT_SIZE as f32,
            Element::Air,
        ))
        .add_startup_system(init.system())
        .add_system_to_stage(GameStage::Interact, change_element.system())
        .add_system_to_stage(GameStage::Interact, brush.system())
        .add_system_to_stage(GameStage::Run, rules.system())
        .add_system_to_stage(GameStage::Run, update_visuals.system())
        .add_system_to_stage(GameStage::Run, tutorial.system())
        .run();
}

fn init(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.window_origin = WindowOrigin::Center;
    commands
        .spawn()
        .insert_bundle(camera_bundle)
        .insert(ViewCamera);

    asset_server.watch_for_changes().unwrap();

    let dark = materials.add(Color::rgba(0.0, 0.0, 0.0, 0.7).into());
    let transparent = materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0).into());

    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Auto,
                },
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: dark.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            for &(element, name, hotkey) in PALETTE {
                parent
                    .spawn_bundle(TextBundle {
                        style: Style {
                            size: Size {
                                width: Val::Auto,
                                height: Val::Px(20.0),
                            },
                            margin: Rect::all(Val::Px(10.0)),
                            ..Default::default()
                        },
                        text: Text::with_section(
                            format!("[{:?}] {}", hotkey, name),
                            TextStyle {
                                font: asset_server.load("menu.ttf"),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                            TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            },
                        ),
                        ..Default::default()
                    })
                    .insert(PaletteItem {
                        paint: element,
                        hotkey,
                    });
            }

            parent.spawn_bundle(NodeBundle {
                style: Style {
                    flex_grow: 1.0,
                    size: Size {
                        width: Val::Auto,
                        height: Val::Undefined,
                    },
                    ..Default::default()
                },
                material: transparent.clone(),
                ..Default::default()
            });

            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size {
                            width: Val::Px(250.0),
                            height: Val::Auto,
                        },
                        flex_direction: FlexDirection::ColumnReverse,
                        margin: Rect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    material: transparent.clone(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        style: Style {
                            size: Size {
                                width: Val::Undefined,
                                height: Val::Px(20.0),
                            },
                            ..Default::default()
                        },
                        text: Text::with_section(
                            "Brush Size",
                            TextStyle {
                                font: asset_server.load("menu.ttf"),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                            TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            },
                        ),
                        ..Default::default()
                    });
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(2.0),
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(BrushSlider);
                });
        });

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            material: transparent.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::ColumnReverse,
                        align_items: AlignItems::Center,
                        padding: Rect::all(Val::Px(10.0)),
                        size: Size {
                            width: Val::Auto,
                            height: Val::Auto,
                        },
                        ..Default::default()
                    },
                    material: dark.clone(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        style: Style {
                            size: Size {
                                width: Val::Auto,
                                height: Val::Px(20.0),
                            },
                            margin: Rect {
                                bottom: Val::Px(5.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: Text::with_section(
                            "How to Play",
                            TextStyle {
                                font: asset_server.load("menu-bold.ttf"),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                            TextAlignment {
                                horizontal: HorizontalAlign::Center,
                                vertical: VerticalAlign::Top,
                            },
                        ),
                        ..Default::default()
                    });
                    parent.spawn_bundle(TextBundle {
                        style: Style {
                            size: Size {
                                width: Val::Px(250.0),
                                height: Val::Auto,
                            },
                            flex_wrap: FlexWrap::Wrap,
                            ..Default::default()
                        },
                        text: Text::with_section(
                            include_str!("instructions.txt"),
                            TextStyle {
                                font: asset_server.load("menu.ttf"),
                                font_size: 18.0,
                                color: Color::WHITE,
                            },
                            TextAlignment {
                                horizontal: HorizontalAlign::Left,
                                vertical: VerticalAlign::Top,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(TutorialWindow);
        });
}

fn update_visuals(
    brush: Res<Brush>,
    mut tiles: Query<(&Element, &mut Material), Changed<Element>>,
    mut palette: Query<(&PaletteItem, &mut Text)>,
    mut slider: Query<&mut Style, With<BrushSlider>>,
) {
    for (element, mut material) in tiles.iter_mut() {
        material.0 = match element {
            Element::Air => 0,
            Element::Rock => 1,
            Element::Water => 2,
            Element::Sand(_) => 3,
        };
    }

    for (item, mut text) in palette.iter_mut() {
        let color = if item.paint == brush.paint {
            Color::WHITE
        } else {
            Color::GRAY
        };

        text.sections[0].style.color = color;
    }

    let precession = (brush.size - BRUSH_SIZE.start) as f32 / BRUSH_SIZE.len() as f32;

    for mut slider in slider.iter_mut() {
        slider.size.width = Val::Percent(5.0 + 95.0 * precession);
    }
}

fn brush(
    mut brush: ResMut<Brush>,
    mut keyboard: EventReader<KeyboardInput>,
    mut mouse: EventReader<MouseWheel>,
    palette: Query<&PaletteItem>,
) {
    for event in keyboard.iter() {
        if let &KeyboardInput {
            key_code: Some(key),
            state: ElementState::Pressed,
            ..
        } = event
        {
            for item in palette.iter() {
                if item.hotkey == key {
                    brush.paint = item.paint;
                }
            }
        }
    }

    for event in mouse.iter() {
        brush.size = match event.y.partial_cmp(&0.0) {
            Some(Ordering::Less) => brush.size.saturating_sub(1).max(BRUSH_SIZE.start),
            Some(Ordering::Greater) => (brush.size + 1).min(BRUSH_SIZE.end),
            _ => continue,
        };
    }
}

fn change_element(
    windows: Res<Windows>,
    brush: Res<Brush>,
    mouse: Res<Input<MouseButton>>,
    tilemap: Res<Tilemap>,
    mut tutorial: ResMut<TutorialTimer>,
    camera: Query<&Transform, With<ViewCamera>>,
    mut tiles: Query<&mut Element>,
) {
    let target = {
        let mut pressed_iter = mouse.get_pressed();

        let target = match pressed_iter.next() {
            Some(&MouseButton::Left) => brush.paint,
            Some(&MouseButton::Right) => Element::Air,
            _ => return,
        };

        if let Some(_) = pressed_iter.next() {
            return;
        }

        target
    };

    tutorial.show.reset();
    tutorial.show.pause();

    let window = windows.get_primary().unwrap();
    let window_size_delta =
        Vec2::new(window.width(), window.height()) - Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    let camera = camera.single().unwrap();

    let cursor = window.cursor_position().unwrap() - window_size_delta / 2.0;
    let cursor = camera.compute_matrix().transform_point3(cursor.extend(0.0));

    let (x, y) = tilemap.px_to_cell(cursor.xy());
    let offsets = -(brush.size as isize)..=brush.size as isize;

    for x_offset in offsets.clone() {
        for y_offset in offsets.clone() {
            let x = x + x_offset;
            let y = y + y_offset;

            let tile = match tilemap.get(x, y) {
                Some(tile) => tile,
                None => continue,
            };

            let element = tiles.get_component_mut::<Element>(tile).ok();

            if let Some(mut element) = element {
                *element = target;
            }
        }
    }
}

fn tutorial(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut timer: ResMut<TutorialTimer>,
    mut window: Query<&mut Style, With<TutorialWindow>>,
) {
    timer.show.tick(time.delta());

    let precession = if keyboard.pressed(KeyCode::Space) || keyboard.pressed(KeyCode::F1) {
        0.0
    } else if timer.show.finished() {
        timer.animate.tick(time.delta());
        timer.animate.percent_left()
    } else {
        timer.animate.reset();
        1.0
    };

    for mut window in window.iter_mut() {
        window.position.top = Val::Percent(precession * 100.0);
    }
}

fn rules(
    queries: QuerySet<(
        Query<&mut Element>,
        Query<
            (
                Entity,
                &TilePosition,
                Option<&UpNeighbour<Element>>,
                Option<&DownNeighbour<Element>>,
                Option<&LeftNeighbour<Element>>,
                Option<&RightNeighbour<Element>>,
            ),
            Or<(
                Changed<Element>,
                Changed<UpNeighbour<Element>>,
                Changed<DownNeighbour<Element>>,
                Changed<LeftNeighbour<Element>>,
                Changed<RightNeighbour<Element>>,
            )>,
        >,
    )>,
    tilemap: Res<Tilemap>,
) {
    for (entity, &TilePosition(x, y), up, down, left, right) in queries.q1().iter() {
        let mut element = unsafe { queries.q0().get_unchecked(entity) }.unwrap();

        if let Element::Air = *element {
            continue;
        }

        let up = up.map(|x| x.0);
        let down = down.map(|x| x.0);
        let left = left.map(|x| x.0);
        let right = right.map(|x| x.0);

        let (dest_x, dest_y, dest_element) = {
            match *element {
                Element::Air => continue,
                Element::Rock => match (up, down, left, right) {
                    (Some(Element::Rock) | None, _, _, _) => continue,
                    (_, Some(Element::Rock) | None, _, _) => continue,
                    (_, _, Some(Element::Rock) | None, _) => continue,
                    (_, _, _, Some(Element::Rock) | None) => continue,
                    _ => (x, y, Element::Sand(0)),
                },
                Element::Water => match down {
                    Some(Element::Air) => (x, y - 1, Element::Water),
                    _ => (x + destabilize_offset(left, right, 5.0), y, Element::Water),
                },
                Element::Sand(_) => match down {
                    Some(Element::Air | Element::Water) => (x, y - 1, Element::Sand(0)),
                    Some(Element::Rock) | None => (x, y, Element::Sand(0)),
                    Some(Element::Sand(distance)) => {
                        let strength =
                            distance + support_strength(left) + support_strength(right) + 1;

                        if strength < 3 {
                            (x, y, Element::Sand(strength))
                        } else {
                            (
                                x + destabilize_offset(left, right, 1.3),
                                y,
                                Element::Sand(0),
                            )
                        }
                    }
                },
            }
        };

        if dest_x == x && dest_y == y {
            if *element != dest_element {
                *element = dest_element;
            }
        } else if let Some(target) = tilemap.get(dest_x, dest_y) {
            let mut target = unsafe { queries.q0().get_unchecked(target) }.unwrap();
            std::mem::swap(&mut *target, &mut *element);
        }
    }
}

fn destabilize_offset(left: Option<Element>, right: Option<Element>, eagerness: f32) -> isize {
    let min = if let Some(Element::Air | Element::Water) = left {
        -eagerness
    } else {
        0.0
    };
    let max = if let Some(Element::Air | Element::Water) = right {
        eagerness
    } else {
        0.0
    };

    (rand::thread_rng().gen_range(min..=max) as isize).signum()
}

fn support_strength(element: Option<Element>) -> u8 {
    match element {
        Some(Element::Sand(_)) => 1,
        Some(Element::Rock) => 2,
        _ => 0,
    }
}
