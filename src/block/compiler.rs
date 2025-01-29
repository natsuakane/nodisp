use bevy::prelude::*;
use std::collections::HashMap;

pub enum AstNode {
    Statement(Vec<AstNode>),
    ValueInteger(i64),
    ValueFloat(f64),
    ValueStr(String),
    Function { func: String, args: Vec<AstNode> },
    List(Vec<AstNode>),
    Identifier(String),
}

pub enum Opecodes {
    PushR,
    PopR,
    PushRP,
    PopRP,
    ClearR,
    CopyR,
    AddI,
    SubI,
    MulI,
    DivI,
    ModI,
    OutputI,
    End,
}

#[derive(Resource, Default)]
pub struct Environment {
    pub stack: Vec<HashMap<String, (u64, String)>>,
}

impl AstNode {
    pub fn compile(&self, environment: &mut Environment) -> Result<(Vec<u8>, String), String> {
        let mut res: Vec<u8> = vec![];
        let mut return_type: String = "".to_string();

        fn add_u8(vec: &mut Vec<u8>, n: u8) {
            vec.push(n);
        }
        fn add_u32(vec: &mut Vec<u8>, n: u32) {
            let nvec: [u8; 4] = n.to_le_bytes();
            for b in nvec {
                vec.push(b);
            }
        }
        fn add_u64(vec: &mut Vec<u8>, n: u64) {
            let nvec: [u8; 8] = n.to_le_bytes();
            for b in nvec {
                vec.push(b);
            }
        }
        fn add_i64(vec: &mut Vec<u8>, n: i64) {
            let nvec: [u8; 8] = n.to_le_bytes();
            for b in nvec {
                vec.push(b);
            }
        }
        fn add_f64(vec: &mut Vec<u8>, n: f64) {
            let nvec: [u8; 8] = n.to_le_bytes();
            for b in nvec {
                vec.push(b);
            }
        }
        fn get_binop_args(
            args: &Vec<AstNode>,
            environment: &mut Environment,
        ) -> Result<((Vec<u8>, String), (Vec<u8>, String)), String> {
            if args.len() != 2 {
                Err(format!(
                    "this function takes 2 arguments but {} argument was supplied",
                    args.len()
                ))
            } else {
                Ok((args[0].compile(environment)?, args[1].compile(environment)?))
            }
        }

        match self {
            AstNode::Statement(vec) => {}
            AstNode::ValueInteger(num) => {
                add_u8(&mut res, Opecodes::PushR as u8);
                add_i64(&mut res, *num);
                return_type = "integer".to_string();
            }
            AstNode::ValueFloat(num) => {
                add_u8(&mut res, Opecodes::PushR as u8);
                add_f64(&mut res, *num);
                return_type = "float".to_string();
            }
            AstNode::ValueStr(str) => {
                // 未完成
                let bytes = str.clone().into_bytes();
                for b in bytes {
                    res.push(b);
                }
                return_type = "string".to_string();
            }
            AstNode::List(vec) => {
                for (i, code) in vec.iter().enumerate() {
                    let (bytes, ret_type) = code.compile(environment)?;
                    res.extend(bytes);
                    return_type = ret_type;
                    if i != vec.len() - 1 {
                        add_u8(&mut res, Opecodes::ClearR as u8);
                    }
                }
            }
            AstNode::Identifier(str) => {}
            AstNode::Function { func, args } => match func.as_str() {
                "addi" => {
                    let (a, b) = get_binop_args(args, environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::AddI as u8);
                    return_type = "integer".to_string();
                }
                "subi" => {
                    let (a, b) = get_binop_args(args, environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::SubI as u8);
                    return_type = "integer".to_string();
                }
                "muli" => {
                    let (a, b) = get_binop_args(args, environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::MulI as u8);
                    return_type = "integer".to_string();
                }
                "divi" => {
                    let (a, b) = get_binop_args(args, environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::DivI as u8);
                    return_type = "integer".to_string();
                }
                "modi" => {
                    let (a, b) = get_binop_args(args, environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::ModI as u8);
                    return_type = "integer".to_string();
                }
                _ => {}
            },
        }

        Ok((res, return_type))
    }
}
