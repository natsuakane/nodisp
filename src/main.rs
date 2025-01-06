use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    scene::ron::value::Float,
    window::PrimaryWindow,
};

mod block;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // Bevyのデフォルトプラグインを追加
        .add_systems(Startup, setup) // 起動時に実行するシステムを登録
        .add_systems(Startup, spawn_grid) // グリッドを追加
        .add_systems(Update, move_camera) // マウス操作を登録
        .add_systems(Update, show_menu) // ブロック配置
        .run();
}

fn setup(mut commands: Commands) {
    // 2Dカメラを追加（四角形を描画するために必要）
    commands.spawn(Camera2d::default());

    // 四角形を追加
    for i in 1..10 {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 1.0),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..Default::default()
            },
            Transform::from_xyz(200.0 * (i % 1000) as f32, 200.0 * (i / 5) as f32, 0.0), // 位置を指定
        ));
    }
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

fn show_menu(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    buttons: Res<ButtonInput<MouseButton>>,
    asset_server: Res<AssetServer>,
) {
    if buttons.pressed(MouseButton::Right) {
        if let Some(screen_pos) = window_query.single().cursor_position() {
            let (camera, camera_transform) = camera_query.single();

            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
                let newblock = block::Block {
                    text: String::from("let aaa = bbb"),
                    position: Vec2::new(world_pos.x, world_pos.y),
                    block_type: block::BlockType::Variable,
                };
                block::spawn_block(&mut commands, newblock, asset_server);
            }
        }
    }
}

fn move_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
) {
    let (mut transform, mut projection) = match query.get_single_mut() {
        Ok(camera) => camera,
        Err(_) => return,
    };

    // カメラの現在のズームレベルに基づいて移動速度を調整
    let movement_speed = projection.scale;

    // マウスドラッグでカメラを移動
    if buttons.pressed(MouseButton::Left) {
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
