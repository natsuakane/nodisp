use bevy::prelude::*;
use bevy::text::*;

pub enum BlockType {
    Statement,
    Expression,
    Function,
    Variable,
}

pub struct Block {
    pub text: String,
    pub position: Vec2,
    pub block_type: BlockType,
}

pub fn spawn_block(commands: &mut Commands, block: Block, asset_server: Res<AssetServer>) {
    let text_entity = commands
        .spawn((
            Text2d::new(String::from(block.text.clone())),
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
            Text2d::new(String::from(match block.block_type {
                BlockType::Statement => "statement",
                BlockType::Expression => "expression",
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
                block.text.clone().len() as f32 * -15.0 / 2.0 + 20.0,
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
                    block.text.clone().len() as f32 * 15.0 + 5.0,
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
                color: match block.block_type {
                    BlockType::Statement => Color::srgb(1.0, 0.3, 0.3),
                    BlockType::Expression => Color::srgb(0.1, 0.8, 0.1),
                    BlockType::Function => Color::srgb(0.3, 0.3, 1.0),
                    BlockType::Variable => Color::srgb(0.3, 0.3, 0.3),
                },
                custom_size: Some(Vec2::new(block.text.clone().len() as f32 * 15.0, 20.0)),
                ..Default::default()
            },
            Transform::from_xyz(block.position.x, block.position.y, 0.0),
        ))
        .add_child(text_entity)
        .add_child(typetext_entity)
        .add_child(shadow_entity);
}
