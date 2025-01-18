use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;
use std::collections::HashMap;

#[derive(Copy, Clone)]
pub enum BlockType {
    Statement,
    Value,
    Function,
    Variable,
}

pub struct Block {
    pub data: BlockData,
    pub position: Vec2,
    pub inputs: Vec<Block>,
}

#[derive(Clone)]
pub struct BlockData {
    pub text: String,
    pub block_type: BlockType,
    pub input_value_types: Vec<String>,
    pub output_value_type: String,
}

#[derive(Resource, Default)]
pub struct BlockDataList {
    pub items: Vec<BlockData>,
}

#[derive(Resource, Default)]
pub struct BlockList {
    pub item: HashMap<u32, (Entity, Block)>,
}

// ドラッグ可能なことを示すマーカーコンポーネント
#[derive(Component)]
pub struct Draggable {
    id: u32,
}

// ドラッグ状態を管理するリソース
#[derive(Resource, Default)]
pub struct DragState {
    dragged_entity: Option<Entity>,
    drag_start: Option<Vec2>,
    pub is_dragging: bool,
}

#[derive(Component)]
pub struct Line {
    start: u32, // id
    end: u32,
    label: String,
}

pub fn spawn_block(
    commands: &mut Commands,
    block: Block,
    asset_server: &AssetServer,
    block_list: &mut BlockList,
) {
    let text_entity = commands
        .spawn((
            Text2d::new(String::from(block.data.text.clone())),
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            TextFont {
                font: asset_server.load("fonts/FiraCode-Medium.ttf"),
                font_size: 20.0,
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ))
        .id();
    let typetext_entity = commands
        .spawn((
            Text2d::new(String::from(match block.data.block_type {
                BlockType::Statement => "statement",
                BlockType::Value => "value",
                BlockType::Function => "function",
                BlockType::Variable => "variable",
            })),
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            TextFont {
                font: asset_server.load("fonts/FiraCode-Medium.ttf"),
                font_size: 7.0,
                ..Default::default()
            },
            Transform::from_xyz(
                block.data.text.clone().len() as f32 * -15.0 / 2.0 + 20.0,
                15.0,
                1.0,
            ),
        ))
        .id();
    let shadow_entity = commands
        .spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.9),
                custom_size: Some(Vec2::new(
                    block.data.text.clone().len() as f32 * 15.0 + 5.0,
                    25.0,
                )),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, -10.0),
        ))
        .id();
    let mut rng = rand::thread_rng();
    let random_id: u32 = rng.gen_range(1..=u32::MAX); // 0を除く
    let block_entity = commands
        .spawn((
            Sprite {
                color: match block.data.block_type {
                    BlockType::Statement => Color::srgb(1.0, 0.3, 0.3),
                    BlockType::Value => Color::srgb(0.1, 0.8, 0.1),
                    BlockType::Function => Color::srgb(0.3, 0.3, 1.0),
                    BlockType::Variable => Color::srgb(0.3, 0.3, 0.3),
                },
                custom_size: Some(Vec2::new(block.data.text.clone().len() as f32 * 15.0, 20.0)),
                ..Default::default()
            },
            Transform::from_xyz(block.position.x, block.position.y, 0.0),
            Draggable { id: random_id },
        ))
        .add_child(text_entity)
        .add_child(typetext_entity)
        .add_child(shadow_entity)
        .id();
    block_list.item.insert(random_id, (block_entity, block));
}

pub fn drag_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<DragState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut sprites: Query<(Entity, &mut Transform), With<Draggable>>,
) {
    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_position) = window.cursor_position() {
        // カーソル位置をワールド座標に変換
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            // マウスの左ボタンが押されたとき
            if mouse_button.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ShiftLeft)
            {
                // ドラッグ開始：カーソルの位置にあるエンティティを探す
                for (entity, transform) in sprites.iter() {
                    let sprite_pos = transform.translation.truncate();
                    if world_position.distance(sprite_pos) < 25.0 {
                        // 判定範囲
                        drag_state.dragged_entity = Some(entity);
                        drag_state.drag_start = Some(world_position);
                        drag_state.is_dragging = true;
                        break;
                    }
                }
            }
            // マウスの左ボタンが離されたとき
            else if mouse_button.just_released(MouseButton::Left)
                && keyboard.pressed(KeyCode::ShiftLeft)
                && drag_state.dragged_entity != None
            {
                // エンティティを削除
                if cursor_position.x >= window.size().x as f32 * 0.8 {
                    commands
                        .entity(drag_state.dragged_entity.unwrap())
                        .despawn_recursive();
                }

                // ドラッグ終了
                drag_state.dragged_entity = None;
                drag_state.drag_start = None;
                drag_state.is_dragging = false;
            }
            // ドラッグ中
            else if mouse_button.pressed(MouseButton::Left)
                && keyboard.pressed(KeyCode::ShiftLeft)
            {
                if let Some(entity) = drag_state.dragged_entity {
                    if let Ok((_, mut transform)) = sprites.get_mut(entity) {
                        // エンティティの位置を更新
                        transform.translation.x = world_position.x;
                        transform.translation.y = world_position.y;
                    }
                }
            }
        }
    }
}

pub fn spawn_line(
    commands: &mut Commands,
    line: Line,
    asset_server: &AssetServer,
    block_list: &Res<BlockList>,
    block_query: Query<&Transform, With<Draggable>>,
) {
    if let (Ok(start), Ok(end)) = (
        block_query.get(block_list.item[&line.start].0),
        block_query.get(block_list.item[&line.end].0),
    ) {
        let start = start.translation;
        let end = end.translation;
        let text_entity = commands
            .spawn((
                Text2d::new(String::from(line.label.clone())),
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                TextFont {
                    font: asset_server.load("fonts/FiraCode-Medium.ttf"),
                    font_size: 20.0,
                    ..Default::default()
                },
                Transform::from_xyz(0.0, 0.0, -100.0),
            ))
            .id();

        // 線の長さと角度を計算
        let difference = end - start;
        let length = difference.length();
        let rotation = difference.y.atan2(difference.x);

        // 線をSpriteとして生成
        commands
            .spawn((
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(length, 2.0)), // 長さと太さ
                    ..default()
                },
                Transform {
                    translation: Vec3::new(
                        start.x + difference.x / 2.0, // 中点のx座標
                        start.y + difference.y / 2.0, // 中点のy座標
                        -100.0,
                    ),
                    rotation: Quat::from_rotation_z(rotation),
                    ..default()
                },
                Line {
                    start: line.start,
                    end: line.end,
                    label: line.label.clone(),
                },
            ))
            .add_child(text_entity);
    }
}

pub fn connect_blocks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    block_list: Res<BlockList>,
    mut queries: ParamSet<(
        Query<(Entity, &mut Transform, &Draggable), With<Draggable>>,
        Query<&mut Transform, With<Draggable>>,
        Query<(Entity, &mut Transform), With<Draggable>>,
        Query<&Transform, With<Draggable>>,
    )>,
    mut line_query: Query<
        (Entity, &mut Transform, &mut Sprite, &Line),
        (With<Line>, Without<Draggable>),
    >,
) {
    static mut START: u32 = 0;
    static mut END: u32 = 0;

    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_position) = window.cursor_position() {
        // カーソル位置をワールド座標に変換
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            if mouse_button.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::Tab) {
                for (_, transform, draggable) in queries.p0().iter() {
                    let sprite_pos = transform.translation.truncate();
                    if world_position.distance(sprite_pos) < 25.0 {
                        // 判定範囲
                        unsafe {
                            if START == 0 {
                                START = draggable.id;
                            } else if START != END {
                                END = draggable.id;
                                let line = Line {
                                    start: START,
                                    end: END,
                                    label: "".to_string(),
                                };
                                spawn_line(
                                    &mut commands,
                                    line,
                                    &asset_server,
                                    &block_list,
                                    queries.p3(),
                                );

                                START = 0;
                                END = 0;
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    for (entity, mut transform, mut sprite, line) in line_query.iter_mut() {
        let start_entity = block_list.as_ref().item[&line.start].0;
        let end_entity = block_list.as_ref().item[&line.end].0;
        let query = queries.p1();
        if let (Ok(start), Ok(end)) = (query.get(start_entity), query.get(end_entity)) {
            let start = start.translation;
            let end = end.translation;

            // 線の長さと角度を計算
            let difference = end - start;
            let length = difference.length();
            let rotation = difference.y.atan2(difference.x);

            let newpos = Vec3::new(
                start.x + difference.x / 2.0, // 中点のx座標
                start.y + difference.y / 2.0, // 中点のy座標
                -100.0,
            );
            transform.translation.x = newpos.x;
            transform.translation.y = newpos.y;
            transform.rotation = Quat::from_rotation_z(rotation);
            sprite.custom_size = Some(Vec2::new(length, 2.0));
        } else {
            commands.entity(entity).despawn_recursive();
        }
    }
}
