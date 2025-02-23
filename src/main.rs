use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};
use bevy_simple_text_input::{TextInput, TextInputPlugin, TextInputSubmitEvent, TextInputSystem};
use block::StartBlock;
use rand::Rng;
use std::f64::consts::PI;
mod block;
use block::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // Bevyのデフォルトプラグインを追加
        .add_plugins(TextInputPlugin) // テキストプラグインを追加
        .add_systems(Startup, setup) // 起動時に実行するシステムを登録
        .add_systems(Startup, spawn_grid) // グリッドを追加
        .add_systems(Startup, spawn_trash_area) //ゴミ箱エリアを追加
        .add_systems(Startup, spawn_value_fields) // Identifier、数値召喚用テキストインプットを追加
        .add_systems(Startup, add_run_button) // 実行ボタン追加
        .add_systems(Update, run_button_click) // 実行ボタンイベント
        .add_systems(Update, move_camera) // マウス操作を登録
        .add_systems(Update, show_menu) // ブロック配置
        .add_systems(Update, menu_search.after(TextInputSystem)) // テキストインプットイベント
        .add_systems(Update, add_value.after(TextInputSystem)) // Identifier、数値召喚イベント
        .add_systems(Update, spawn_block_button) // ブロック配置
        .add_systems(Update, connect_blocks) // 接続
        .insert_resource(block::DragState::default()) // リソース追加
        .insert_resource(block::BlockDataList::default()) // ブロックのリストを追加
        .insert_resource(block::BlockList::default()) // 出されたブロックのリストを追加
        .insert_resource(block::StartBlock::default()) // スタート位置指定
        .insert_resource(block::compiler::Environment::default()) // 環境
        .add_systems(Update, drag_system) // ドラッグできるようにする
        .run();
}

fn setup(
    mut commands: Commands,
    mut block_data_list: ResMut<block::BlockDataList>,
    mut block_list: ResMut<block::BlockList>,
    asset_server: Res<AssetServer>,
    mut start_block: ResMut<block::StartBlock>,
) {
    // 2Dカメラを追加（四角形を描画するために必要）
    commands.spawn(Camera2d::default());

    const LISTPLACE: usize = 8;

    block_data_list.items = vec![
        block::BlockData {
            text: String::from("addi"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("subi"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("muli"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("divi"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("modi"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("printi"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("set"),
            block_type: block::BlockType::Statement,
        },
        block::BlockData {
            text: String::from("lambda"),
            block_type: block::BlockType::Statement,
        },
        block::BlockData {
            text: String::from("list"),
            block_type: block::BlockType::List,
        },
        block::BlockData {
            text: String::from("identifier_list"),
            block_type: block::BlockType::List,
        },
        block::BlockData {
            text: String::from("if"),
            block_type: block::BlockType::Statement,
        },
        block::BlockData {
            text: String::from("addf"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("subf"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("mulf"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("divf"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("modf"),
            block_type: block::BlockType::Identifier,
        },
        block::BlockData {
            text: String::from("printf"),
            block_type: block::BlockType::Identifier,
        },
    ];

    let id = spawn_block(
        &mut commands,
        block::Block {
            data: block_data_list.items[LISTPLACE].clone(),
            position: Vec2::new(0.0, 0.0),
            inputs: vec![],
            comment: "start block".to_string(),
        },
        asset_server.as_ref(),
        &mut block_list,
    );
    start_block.start_block = id;
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
            Transform::from_xyz(x_position, 0.0, -101.0),
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
            Transform::from_xyz(0.0, y_position, -101.0),
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

#[derive(Component)]
struct ValueInput;

fn spawn_value_fields(mut commands: Commands) {
    commands.spawn((
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
        ValueInput,
    ));
}

#[derive(Component)]
struct RunButton;

#[derive(Component)]
struct ResultText;

fn add_run_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Button,
        Node {
            width: Val::Px(80.0),
            height: Val::Px(80.0),
            top: Val::Px(0.0),
            right: Val::Percent(23.0),
            position_type: PositionType::Absolute,
            border: UiRect::all(Val::Px(5.0)),
            ..Default::default()
        },
        ImageNode {
            image: asset_server.load("images/ExecuteButton.png").clone(),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        RunButton,
    ));
}

fn run_button_click(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RunButton>),
    >,
    mut result_texts: Query<Entity, With<ResultText>>,
    block_list: Res<block::BlockList>,
    start_block: Res<block::StartBlock>,
    asset_server: Res<AssetServer>,
    mut environment: ResMut<block::compiler::Environment>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::srgba(0.8, 0.8, 0.8, 0.4).into();
                println!("Compiling...");
                environment.stack.clear();
                let start_point_block = block_list.item[&start_block.start_block].1.clone();
                let result = match start_point_block.parse(block_list.as_ref()) {
                    Ok(code) => match code.compile(environment.as_mut()) {
                        Ok((mut bytes, ret_type)) => {
                            bytes.push(block::compiler::Opecodes::End as u8);
                            for i in 0..bytes.len() {
                                print!("{}:{:#X} ", i, bytes[i]);
                            }
                            println!("=> {}", ret_type);

                            match block::compiler::execute_vm(bytes) {
                                Ok(res) => res,
                                Err(msg) => msg,
                            }
                        }
                        Err(msg) => {
                            format!("CompileError:{}", msg)
                        }
                    },
                    Err(msg) => {
                        format!("ParseError:{}", msg)
                    }
                };

                for text in result_texts.iter_mut() {
                    commands.entity(text).despawn_recursive();
                }
                commands.spawn((
                    Text::new(result.clone()),
                    TextFont {
                        font: asset_server.load("fonts/FiraCode-Medium.ttf"),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node {
                        top: Val::Px(100.0),
                        right: Val::Percent(23.0),
                        position_type: PositionType::Absolute,
                        border: UiRect::all(Val::Px(5.0)),
                        ..Default::default()
                    },
                    ResultText,
                ));
            }
            Interaction::Hovered => {
                *color = Color::srgba(0.8, 0.8, 0.8, 0.2).into();
            }
            Interaction::None => {
                *color = Color::srgba(0.0, 0.0, 0.0, 0.0).into();
            }
        };
    }
}

fn add_value(
    mut commands: Commands,
    mut events: EventReader<TextInputSubmitEvent>,
    mut block_data_list: ResMut<block::BlockDataList>,
    mut block_list: ResMut<block::BlockList>,
    asset_server: Res<AssetServer>,
    value_inputs: Query<Entity, With<ValueInput>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    for event in events.read() {
        if !value_inputs.contains(event.entity) {
            break;
        }

        let (camera, camera_transform) = camera_query.single();
        match event.value.parse::<f64>() {
            Ok(num) => {
                let position = Vec2::new(
                    window_query.single().width() / 2.0,
                    window_query.single().height() / 2.0,
                );
                if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, position)
                {
                    spawn_block(
                        &mut commands,
                        block::Block {
                            data: block::BlockData {
                                text: num.to_string(),
                                block_type: block::BlockType::Value,
                            },
                            position: world_position,
                            inputs: vec![],
                            comment: "".to_string(),
                        },
                        asset_server.as_ref(),
                        block_list.as_mut(),
                    );
                }
            }
            Err(_) => {
                let position = Vec2::new(
                    window_query.single().width() / 2.0,
                    window_query.single().height() / 2.0,
                );
                if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, position)
                {
                    spawn_block(
                        &mut commands,
                        block::Block {
                            data: block::BlockData {
                                text: event.value.clone(),
                                block_type: block::BlockType::Identifier,
                            },
                            position: world_position,
                            inputs: vec![],
                            comment: "".to_string(),
                        },
                        asset_server.as_ref(),
                        block_list.as_mut(),
                    );
                }
                block_data_list.items.push(block::BlockData {
                    text: event.value.clone(),
                    block_type: block::BlockType::Identifier,
                });
            }
        };
    }
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
struct BlockItem {
    pub data: block::BlockData,
}

#[derive(Component)]
struct SearchInput;

fn show_menu(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    menus: Query<Entity, With<Menu>>,
    block_list: Res<block::BlockDataList>,
    asset_server: Res<AssetServer>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        if let Some(screen_pos) = window_query.single().cursor_position() {
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
                            SearchInput,
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
                                        BlockItem {
                                            data: block::BlockData {
                                                text: block_list.items[i].text.clone(),
                                                block_type: block_list.items[i].block_type,
                                            },
                                        },
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
    block_list: Res<block::BlockDataList>,
    children: Query<(Entity, &Parent), With<BlockItem>>,
    asset_server: Res<AssetServer>,
    search_inputs: Query<Entity, With<SearchInput>>,
) {
    for event in events.read() {
        if !search_inputs.contains(event.entity) {
            break;
        }

        for (child_entity, parent) in children.iter() {
            if parent.get() == event.entity {
                commands.entity(child_entity).despawn_recursive();
            }
        }

        for i in 0..block_list.items.len() {
            if !(&block_list.items[i].text.clone()).contains(&(event.value)) {
                continue;
            }
            commands.entity(event.entity).with_children(|parent| {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(170.0),
                            height: Val::Px(20.0),
                            border: UiRect::all(Val::Px(5.0)),
                            ..Default::default()
                        },
                        BlockItem {
                            data: block::BlockData {
                                text: block_list.items[i].text.clone(),
                                block_type: block_list.items[i].block_type,
                            },
                        },
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
            });
        }

        println!("{:?} submitted: {}", event.entity, event.value);
    }
}

fn spawn_block_button(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &BlockItem,
        ),
        (Changed<Interaction>, With<BlockItem>),
    >,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    menus: Query<Entity, With<Menu>>,
    mut block_list: ResMut<block::BlockList>,
) {
    for (interaction, mut color, mut border_color, block_item) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::srgba(0.8, 0.8, 0.8, 0.8).into();
                border_color.0 = Color::srgba(0.8, 0.8, 0.8, 0.8).into();

                let window = window_query.single();
                let (camera, camera_transform) = camera_query.single();
                if let Some(screen_pos) = window.cursor_position() {
                    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos)
                    {
                        let newblock = block::Block {
                            data: block_item.data.clone(),
                            position: Vec2::new(world_pos.x, world_pos.y),
                            inputs: vec![],
                            comment: "".to_string(),
                        };
                        spawn_block(
                            &mut commands,
                            newblock,
                            asset_server.as_ref(),
                            &mut block_list,
                        );
                    }
                }

                for menu in menus.iter() {
                    commands.entity(menu).despawn_recursive();
                    return;
                }
            }
            Interaction::Hovered => {
                *color = Color::srgba(0.7, 0.7, 0.7, 0.8).into();
                border_color.0 = Color::srgba(0.7, 0.7, 0.7, 0.8).into();
            }
            Interaction::None => {
                *color = Color::srgba(0.2, 0.2, 0.2, 0.9).into();
                border_color.0 = Color::srgba(0.2, 0.2, 0.2, 0.9).into();
            }
        }
    }
}

pub fn spawn_block(
    commands: &mut Commands,
    block: Block,
    asset_server: &AssetServer,
    block_list: &mut BlockList,
) -> u32 {
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
    let typetext_entity = commands // it also adds comment
        .spawn((
            Text2d::new(
                String::from(match block.data.block_type {
                    BlockType::Statement => "statement",
                    BlockType::Value => "value",
                    BlockType::List => "list",
                    BlockType::Identifier => "identifier",
                }) + " : "
                    + &block.comment,
            ),
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
                    BlockType::List => Color::srgb(0.1, 0.1, 0.1),
                    BlockType::Identifier => Color::srgb(0.8, 0.8, 0.8),
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

    random_id
}

pub fn drag_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<DragState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut sprites: Query<(Entity, &mut Transform, &Draggable), With<Draggable>>,
    start_block: Res<StartBlock>,
    mut block_list: ResMut<BlockList>,
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
                for (entity, transform, _) in sprites.iter() {
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
                // スタートのブロックの場合は削除させない
                if let Some(entity) = drag_state.dragged_entity {
                    if let Ok((_, _, draggable)) = sprites.get_mut(entity) {
                        if draggable.id == start_block.start_block {
                            return;
                        }

                        if block_list.item.contains_key(&draggable.id)
                            && cursor_position.x >= window.size().x as f32 * 0.8
                        {
                            block_list.item.remove(&draggable.id);
                            for block in block_list.item.iter_mut() {
                                if block.1 .1.inputs.contains(&draggable.id) {
                                    if let Some(pos) =
                                        block.1 .1.inputs.iter().position(|x| *x == draggable.id)
                                    {
                                        block.1 .1.inputs.remove(pos);
                                    }
                                }
                            }
                        }
                    }
                }

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
                    if let Ok((_, mut transform, _)) = sprites.get_mut(entity) {
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
    block_list: &ResMut<BlockList>,
    block_query: Query<&Transform, With<Draggable>>,
) {
    println!("{},{}, {}", &line.start, &line.end, block_list.item.len(),);
    println!(
        "{}",
        block_list.item.keys().cloned().collect::<Vec<u32>>()[0]
    );
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
                Transform {
                    translation: Vec3::new(0.0, -13.0, -100.0),
                    rotation: Quat::from_rotation_z(PI as f32),
                    ..Default::default()
                },
            ))
            .id();

        // 矢印
        let arrow_entity1 = commands
            .spawn((
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(10.0, 2.0)),
                    ..Default::default()
                },
                Transform {
                    translation: Vec3::new(0.0, 3.0, -100.0),
                    rotation: Quat::from_rotation_z(-PI as f32 / 6.0),
                    ..Default::default()
                },
            ))
            .id();
        // 矢印
        let arrow_entity2 = commands
            .spawn((
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(10.0, 2.0)),
                    ..Default::default()
                },
                Transform {
                    translation: Vec3::new(0.0, -3.0, -100.0),
                    rotation: Quat::from_rotation_z(PI as f32 / 6.0),
                    ..Default::default()
                },
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
            .add_child(text_entity)
            .add_child(arrow_entity1)
            .add_child(arrow_entity2);
    } else {
        println!("failed to connect blocks.")
    };
}

pub fn connect_blocks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut block_list: ResMut<BlockList>,
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
                                    label: (block_list
                                        .as_mut()
                                        .item
                                        .get_mut(&END)
                                        .unwrap()
                                        .1
                                        .inputs
                                        .len()
                                        + 1)
                                    .to_string(),
                                };
                                spawn_line(
                                    &mut commands,
                                    line,
                                    &asset_server,
                                    &block_list,
                                    queries.p3(),
                                );

                                block_list
                                    .as_mut()
                                    .item
                                    .get_mut(&END)
                                    .unwrap()
                                    .1
                                    .inputs
                                    .push(START);

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
        if !block_list.item.contains_key(&line.start) || !block_list.item.contains_key(&line.end) {
            commands.entity(entity).despawn_recursive();
            return;
        }

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
