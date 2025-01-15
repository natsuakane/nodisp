use bevy::prelude::*;
use bevy::text::*;
use bevy::window::PrimaryWindow;

#[derive(Copy, Clone)]
pub enum BlockType {
    Statement,
    Value,
    Function,
    Variable,
}

pub fn newBlockType(bt: BlockType) -> BlockType {
    match bt {
        BlockType::Statement => BlockType::Statement,
        BlockType::Value => BlockType::Value,
        BlockType::Function => BlockType::Function,
        BlockType::Variable => BlockType::Variable,
    }
}

pub struct Block {
    pub data: BlockData,
    pub position: Vec2,
}

#[derive(Clone)]
pub struct BlockData {
    pub text: String,
    pub block_type: BlockType,
}

#[derive(Resource, Default)]
pub struct BlockList {
    pub items: Vec<BlockData>,
}

// ドラッグ可能なことを示すマーカーコンポーネント
#[derive(Component)]
pub struct Draggable;

// ドラッグ状態を管理するリソース
#[derive(Resource, Default)]
pub struct DragState {
    dragged_entity: Option<Entity>,
    drag_start: Option<Vec2>,
    pub is_dragging: bool,
}

pub fn spawn_block(commands: &mut Commands, block: Block, asset_server: &AssetServer) {
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
    commands
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
            Draggable,
        ))
        .add_child(text_entity)
        .add_child(typetext_entity)
        .add_child(shadow_entity);
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
