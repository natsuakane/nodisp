use bevy::prelude::*;
use std::collections::HashMap;

pub mod compiler;
use compiler::*;

#[derive(Copy, Clone)]
pub enum BlockType {
    Statement,
    Value,
    List,
    Identifier,
}

#[derive(Clone)]
pub struct Block {
    pub data: BlockData,
    pub position: Vec2,
    pub inputs: Vec<u32>,
    pub comment: String,
}

#[derive(Clone)]
pub struct BlockData {
    pub text: String,
    pub block_type: BlockType,
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
    pub id: u32,
}

// ドラッグ状態を管理するリソース
#[derive(Resource, Default)]
pub struct DragState {
    pub dragged_entity: Option<Entity>,
    pub drag_start: Option<Vec2>,
    pub is_dragging: bool,
}

#[derive(Resource, Default)]
pub struct StartBlock {
    pub start_block: u32,
}

#[derive(Component)]
pub struct Line {
    pub start: u32, // id
    pub end: u32,
    pub label: String,
}

impl Block {
    pub fn parse(&self, block_list: &BlockList) -> Result<AstNode, String> {
        match self.data.block_type {
            BlockType::Statement => {
                let mut options: Vec<AstNode> = vec![];
                for exp in self.inputs.clone() {
                    options.push(block_list.item[&exp].1.parse(block_list)?);
                }
                Ok(AstNode::Statement {
                    statement: self.data.text.clone(),
                    options,
                })
            }
            BlockType::Value => match self.data.text.parse::<i64>() {
                Ok(num) => Ok(AstNode::ValueInteger(num)),
                Err(_) => match self.data.text.parse::<f64>() {
                    Ok(num) => Ok(AstNode::ValueFloat(num)),
                    Err(_) => Ok(AstNode::ValueStr(self.data.text.clone())),
                },
            },
            BlockType::List => {
                let mut res: Vec<AstNode> = vec![];
                for exp in self.inputs.clone() {
                    res.push(block_list.item[&exp].1.parse(block_list)?);
                }
                Ok(AstNode::List {
                    name: self.data.text.clone(),
                    codes: res,
                })
            }
            BlockType::Identifier => {
                if self.inputs.len() != 0 {
                    let mut args: Vec<AstNode> = vec![];
                    for exp in self.inputs.clone() {
                        args.push(block_list.item[&exp].1.parse(block_list)?);
                    }
                    Ok(AstNode::Function {
                        func: self.data.text.clone(),
                        args,
                    })
                } else {
                    Ok(AstNode::Identifier(self.data.text.clone()))
                }
            }
        }
    }
}
