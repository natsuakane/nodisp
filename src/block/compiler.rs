use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
pub enum AstNode {
    Statement {
        statement: String,
        options: Vec<AstNode>,
    },
    ValueInteger(i64),
    ValueFloat(f64),
    ValueStr(String),
    Function {
        func: String,
        args: Vec<AstNode>,
    },
    List {
        name: String,
        codes: Vec<AstNode>,
    },
    Identifier(String),
}

pub enum Opecodes {
    PushR,
    PopR,
    PushRP,
    PopRP,
    ClearR,
    CopyR,
    PushS32,
    PushS64,
    PopS32,
    PopS64,
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

impl Environment {
    pub fn find(&self, name: String) -> Result<(u64, String), String> {
        for i in (0..self.stack.len()).rev() {
            if let Some(res) = self.stack[i].get(&name) {
                return Ok(res.clone());
            }
        }
        Err(format!("variable '{}' is not defined.", name))
    }
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
            expected_type: String,
            environment: &mut Environment,
        ) -> Result<((Vec<u8>, String), (Vec<u8>, String)), String> {
            if args.len() != 2 {
                Err(format!(
                    "this function takes 2 arguments but {} argument was supplied.",
                    args.len()
                ))
            } else {
                let a = args[0].compile(environment)?;
                if a.1 != expected_type {
                    Err(format!(
                        "expected type {}, but found type {}.",
                        expected_type, a.1
                    ))
                } else {
                    let b = args[1].compile(environment)?;
                    if b.1 != expected_type {
                        Err(format!(
                            "expected type {}, but found type {}.",
                            expected_type, b.1
                        ))
                    } else {
                        Ok((a, b))
                    }
                }
            }
        }
        fn get_identifier_list(identifier_list: AstNode) -> Result<Vec<String>, String> {
            match identifier_list {
                AstNode::List { name, codes } => {
                    if name != "identifier_list".to_string() {
                        return Err(format!("expected type was 'identifier'."));
                    }
                    let mut res: Vec<String> = vec![];
                    for identifier in codes {
                        if let AstNode::Identifier(id) = identifier {
                            res.push(id);
                        } else {
                            return Err(format!("expected type was 'identifier'."));
                        }
                    }
                    Ok(res)
                }
                _ => {
                    return Err(format!("expected block was 'identifier_list'"));
                }
            }
        }

        match self {
            AstNode::Statement { statement, options } => match statement.as_str() {
                "set" => {
                    if options.len() != 2 {
                        return Err(format!(
                            "this statement takes 1 options but {} options was supplied.",
                            options.len()
                        ));
                    }
                    match &options[0] {
                        AstNode::Identifier(idf) => {
                            let var = environment.find(idf.clone())?;
                            let exp = &options[1].compile(environment)?;
                            res.extend(exp.0.clone());

                            if var.1 != exp.1 {
                                return Err(format!(
                                    "expected type '{}', but found type '{}'.",
                                    var.1, exp.1
                                ));
                            }

                            add_u8(&mut res, Opecodes::PopRP as u8);
                            add_u64(&mut res, var.0);
                        }
                        _ => return Err(format!("expected type was 'identifier'.")),
                    }
                }
                _ => return Err(format!("Unknown statement '{}'.", statement)),
            },
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
            AstNode::List { name, codes } => match name.as_str() {
                "list" => {
                    let local_variables = get_identifier_list(codes[0].clone())?;
                    let mut hash: HashMap<String, (u64, String)> = HashMap::default();
                    for (i, var) in local_variables.iter().enumerate() {
                        hash.insert(var.to_string(), (i as u64 * 8, "integer".to_string()));
                        add_u8(&mut res, Opecodes::PushS64 as u8);
                        add_u64(&mut res, 0);
                    }
                    environment.stack.push(hash);

                    for (i, code) in codes[1..].iter().enumerate() {
                        let (bytes, ret_type) = code.compile(environment)?;
                        res.extend(bytes);
                        return_type = ret_type;
                        if i != codes.len() - 1 {
                            add_u8(&mut res, Opecodes::ClearR as u8);
                        }
                    }
                }
                _ => return Err(format!("unknow list node '{}'.", name)),
            },
            AstNode::Identifier(str) => {
                let var = environment.find(str.clone())?;
                add_u8(&mut res, Opecodes::PushRP as u8);
                add_u64(&mut res, var.0);
                return_type = var.1;
            }
            AstNode::Function { func, args } => match func.as_str() {
                "addi" => {
                    let (a, b) = get_binop_args(args, "integer".to_string(), environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::AddI as u8);
                    return_type = "integer".to_string();
                }
                "subi" => {
                    let (a, b) = get_binop_args(args, "integer".to_string(), environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::SubI as u8);
                    return_type = "integer".to_string();
                }
                "muli" => {
                    let (a, b) = get_binop_args(args, "integer".to_string(), environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::MulI as u8);
                    return_type = "integer".to_string();
                }
                "divi" => {
                    let (a, b) = get_binop_args(args, "integer".to_string(), environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::DivI as u8);
                    return_type = "integer".to_string();
                }
                "modi" => {
                    let (a, b) = get_binop_args(args, "integer".to_string(), environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::ModI as u8);
                    return_type = "integer".to_string();
                }
                "printi" => {
                    if args.len() != 1 {
                        return Err(format!(
                            "this function takes 1 arguments but {} argument was supplied.",
                            args.len()
                        ));
                    }
                    let a = args[0].compile(environment)?;
                    if a.1 != "integer".to_string() {
                        return Err(format!("expected type integer, but found type {}.", a.1));
                    }
                    add_u8(&mut res, Opecodes::OutputI as u8);
                    return_type = "integer".to_string();
                }
                _ => {}
            },
        }

        Ok((res, return_type))
    }
}
