use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};
use bevy_simple_text_input::{TextInput, TextInputPlugin, TextInputSubmitEvent, TextInputSystem};

mod block;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // Bevyのデフォルトプラグインを追加
        .add_plugins(TextInputPlugin) // テキストプラグインを追加
        .add_systems(Startup, setup) // 起動時に実行するシステムを登録
        .add_systems(Startup, spawn_grid) // グリッドを追加
        .add_systems(Startup, spawn_trash_area) //ゴミ箱エリアを追加
        .add_systems(Update, move_camera) // マウス操作を登録
        .add_systems(Update, show_menu) // ブロック配置
        .add_systems(Update, menu_search.after(TextInputSystem)) // テキストインプットイベント
        .insert_resource(block::DragState::default()) // リソース追加
        .insert_resource(block::BlockList::default()) // ブロックのリストを追加
        .add_systems(Update, block::drag_system) // ドラッグできるようにする
        .run();
}

fn setup(mut commands: Commands, mut block_list: ResMut<block::BlockList>) {
    // 2Dカメラを追加（四角形を描画するために必要）
    commands.spawn(Camera2d::default());

    block_list.items = vec![
        block::BlockData {
            text: String::from("+"),
            block_type: block::BlockType::Expression,
        },
        block::BlockData {
            text: String::from("-"),
            block_type: block::BlockType::Expression,
        },
        block::BlockData {
            text: String::from("*"),
            block_type: block::BlockType::Expression,
        },
        block::BlockData {
            text: String::from("/"),
            block_type: block::BlockType::Expression,
        },
        block::BlockData {
            text: String::from("%"),
            block_type: block::BlockType::Expression,
        },
        block::BlockData {
            text: String::from("print"),
            block_type: block::BlockType::Function,
        },
    ];
}

fn spawn_grid(mut commands: Commands) {
    let grid_size = 1000000.0;
    let cell_count = 10000;
    let cell_size = grid_size / cell_count as f32;
    let grid_color = Color::srgb_u8(128, 128, 128);
    let line_thickness = 1.0;

    // 縦線
    for i in 0..=cell_count {
        let x_position = (i as f32 * cell_size) - (grid_size / 2.0);
        commands.spawn((
            Sprite {
                color: grid_color,
                custom_size: Some(Vec2::new(line_thickness, grid_size)),
                ..Default::default()
            },
            Transform::from_xyz(x_position, 0.0, -1.0),
        ));
    }

    // 横線
    for i in 0..=cell_count {
        let y_position = (i as f32 * cell_size) - (grid_size / 2.0);
        commands.spawn((
            Sprite {
                color: grid_color,
                custom_size: Some(Vec2::new(grid_size, line_thickness)),
                ..Default::default()
            },
            Transform::from_xyz(0.0, y_position, -1.0),
        ));
    }
}

fn spawn_trash_area(mut commands: Commands, asset_server: Res<AssetServer>) {
    let trash_image = commands
        .spawn((
            ImageNode {
                image: asset_server.load("images/gomibako.png").clone(),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
            Node {
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Node {
                width: Val::Percent(20.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.3)),
        ))
        .add_child(trash_image);
}

fn move_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
) {
    let (mut transform, mut projection) = match query.get_single_mut() {
        Ok(camera) => camera,
        Err(_) => return,
    };

    // カメラの現在のズームレベルに基づいて移動速度を調整
    let movement_speed = projection.scale;

    // マウスドラッグでカメラを移動
    if buttons.pressed(MouseButton::Left) && !keyboard.pressed(KeyCode::ShiftLeft) {
        for event in mouse_motion.read() {
            transform.translation.x -= event.delta.x * movement_speed;
            transform.translation.y += event.delta.y * movement_speed;
        }
    }

    // マウスホイールでズームイン・アウト
    for event in mouse_wheel.read() {
        const ZOOM_SPEED: f32 = 0.05;
        const MIN_ZOOM: f32 = 0.1;
        const MAX_ZOOM: f32 = 5.0;

        let zoom_delta = -event.y * ZOOM_SPEED;
        let new_scale = projection.scale * (1.0 + zoom_delta);
        projection.scale = new_scale.clamp(MIN_ZOOM, MAX_ZOOM);
    }
}

#[derive(Component)]
struct Menu;

#[derive(Component)]
struct BlockItem;

fn show_menu(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    menus: Query<(Entity), With<Menu>>,
    block_list: Res<block::BlockList>,
    asset_server: Res<AssetServer>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        if let Some(screen_pos) = window_query.single().cursor_position() {
            /*
            let (camera, camera_transform) = camera_query.single();
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
                let newblock = block::Block {
                    text: (0..7).map(|_| rng.sample(Alphanumeric) as char).collect(),
                    position: Vec2::new(world_pos.x, world_pos.y),
                    block_type: block::BlockType::Expression,
                };
                block::spawn_block(&mut commands, newblock, asset_server);
            }
            */
            for menu in menus.iter() {
                commands.entity(menu).despawn_recursive();
                return;
            }

            commands
                .spawn((
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(400.0),
                        left: Val::Px(screen_pos.x),
                        top: Val::Px(screen_pos.y),
                        overflow: Overflow::clip(),
                        ..Default::default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                    Menu,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(170.0),
                                border: UiRect::all(Val::Px(5.0)),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(15.0)),
                                row_gap: Val::Px(3.0),
                                column_gap: Val::Px(3.0),
                                ..Default::default()
                            },
                            BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                            TextInput,
                        ))
                        .with_children(|parent| {
                            for i in 0..block_list.items.len() {
                                parent
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(170.0),
                                            height: Val::Px(20.0),
                                            border: UiRect::all(Val::Px(5.0)),
                                            ..Default::default()
                                        },
                                        BlockItem,
                                        BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                                    ))
                                    .with_child((
                                        Text::new(block_list.items[i].text.clone()),
                                        TextFont {
                                            font: asset_server.load("fonts/FiraCode-Medium.ttf"),
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                    ));
                            }
                        });
                });
        }
    }
}

fn menu_search(
    mut commands: Commands,
    mut events: EventReader<TextInputSubmitEvent>,
    menus: Query<(Entity), With<Menu>>,
    block_list: Res<block::BlockList>,
    children: Query<(Entity, &Parent), With<BlockItem>>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        for (child_entity, parent) in children.iter() {
            if parent.get() == event.entity {
                println!("AAA");
                commands.entity(child_entity).despawn_recursive();
            }
        }

        for i in 0..block_list.items.len() {
            println!(
                "{},{},{}",
                &(event.value),
                &block_list.items[i].text.clone(),
                (&block_list.items[i].text.clone()).contains(&(event.value))
            );
            if !(&block_list.items[i].text.clone()).contains(&(event.value)) {
                continue;
            }
            commands
                .entity(event.entity)
                .with_children(|parent| {
                    parent.spawn((
                        Button,
                        Node {
                            width: Val::Px(170.0),
                            height: Val::Px(20.0),
                            border: UiRect::all(Val::Px(5.0)),
                            ..Default::default()
                        },
                        BlockItem,
                        BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                    ));
                })
                .with_child((
                    Text::new(block_list.items[i].text.clone()),
                    TextFont {
                        font: asset_server.load("fonts/FiraCode-Medium.ttf"),
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
        }

        println!("{:?} submitted: {}", event.entity, event.value);
    }
}
