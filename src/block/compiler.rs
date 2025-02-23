use bevy::{color::palettes::css::PERU, prelude::*};
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
    CopySP,      // スタックの指定の場所から８バイトをコピー
    OverWriteSP, // スタックの指定の場所で８バイトを書き換え 先に値をスタックに積んでおく v OverWriteSP p
    SaveR,
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
    Jump,
    SetFP,
    SetRET,
    ResetFP,
    PushRET,
    IfNotJump,
    ExportFP,
    PushFP,
    AddF,
    SubF,
    MulF,
    DivF,
    ModF,
    OutputF,
    End,
}

impl TryFrom<u8> for Opecodes {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Opecodes::CopySP),
            0x01 => Ok(Opecodes::OverWriteSP),
            0x02 => Ok(Opecodes::SaveR),
            0x03 => Ok(Opecodes::PushS32),
            0x04 => Ok(Opecodes::PushS64),
            0x05 => Ok(Opecodes::PopS32),
            0x06 => Ok(Opecodes::PopS64),
            0x07 => Ok(Opecodes::AddI),
            0x08 => Ok(Opecodes::SubI),
            0x09 => Ok(Opecodes::MulI),
            0x0A => Ok(Opecodes::DivI),
            0x0B => Ok(Opecodes::ModI),
            0x0C => Ok(Opecodes::OutputI),
            0x0D => Ok(Opecodes::Jump),
            0x0E => Ok(Opecodes::SetFP),
            0x0F => Ok(Opecodes::SetRET),
            0x10 => Ok(Opecodes::ResetFP),
            0x11 => Ok(Opecodes::PushRET),
            0x12 => Ok(Opecodes::IfNotJump),
            0x13 => Ok(Opecodes::ExportFP),
            0x14 => Ok(Opecodes::PushFP),
            0x15 => Ok(Opecodes::AddF),
            0x16 => Ok(Opecodes::SubF),
            0x17 => Ok(Opecodes::MulF),
            0x18 => Ok(Opecodes::DivF),
            0x19 => Ok(Opecodes::ModF),
            0x1A => Ok(Opecodes::OutputF),
            0x1B => Ok(Opecodes::End),
            _ => Err(()), // 無効な値はエラーを返す
        }
    }
}

#[derive(Resource, Default)]
pub struct Environment {
    pub stack: Vec<HashMap<String, (i64, String)>>,
}

impl Environment {
    pub fn find(&self, name: String) -> Result<(i64, String), String> {
        for i in (0..self.stack.len()).rev() {
            if let Some(res) = self.stack[i].get(&name) {
                return Ok(res.clone());
            }
        }
        Err(format!("variable '{}' is not defined.", name))
    }
    pub fn set_type(&mut self, name: String, type_name: String) -> Result<(), String> {
        for i in (0..self.stack.len()).rev() {
            if let Some(res) = self.stack[i].get_mut(&name) {
                res.1 = type_name.clone();
                return Ok(());
            }
        }
        Err(format!("variable '{}' is not defined.", name))
    }
}

fn check_type(t1: String, t2: String) -> bool {
    if t1 == "".to_string() {
        return true;
    }
    t1 == t2
}

impl AstNode {
    pub fn compile(&self, environment: &mut Environment) -> Result<(Vec<u8>, String), String> {
        static mut CURRENT_POS: u32 = 0;

        let mut res: Vec<u8> = vec![];
        let mut return_type: String = "".to_string();

        fn add_u8(vec: &mut Vec<u8>, n: u8) {
            vec.push(n);
            unsafe {
                CURRENT_POS += 1;
            }
        }
        fn add_u32(vec: &mut Vec<u8>, n: u32) {
            let nvec: [u8; 4] = n.to_le_bytes();
            for b in nvec {
                vec.push(b);
            }
            unsafe {
                CURRENT_POS += 4;
            }
        }
        fn add_u64(vec: &mut Vec<u8>, n: u64) {
            let nvec: [u8; 8] = n.to_le_bytes();
            for b in nvec {
                vec.push(b);
            }
            unsafe {
                CURRENT_POS += 8;
            }
        }
        fn add_i64(vec: &mut Vec<u8>, n: i64) {
            let nvec: [u8; 8] = n.to_le_bytes();
            for b in nvec {
                vec.push(b);
            }
            unsafe {
                CURRENT_POS += 8;
            }
        }
        fn add_f64(vec: &mut Vec<u8>, n: f64) {
            let nvec: [u8; 8] = n.to_le_bytes();
            for b in nvec {
                vec.push(b);
            }
            unsafe {
                CURRENT_POS += 8;
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
                if !check_type(a.1.clone(), expected_type.clone()) {
                    Err(format!(
                        "expected type {}, but found type {}.",
                        expected_type, a.1
                    ))
                } else {
                    let b = args[1].compile(environment)?;
                    if !check_type(b.1.clone(), expected_type.clone()) {
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
                            return_type = exp.1.clone();
                            res.extend(exp.0.clone());
                            if var.1 != "".to_string() && var.1 != exp.1 {
                                return Err(format!(
                                    "expected type '{}', but found type '{}'.",
                                    var.1, exp.1
                                ));
                            }

                            environment.set_type(idf.clone(), exp.1.clone())?;
                            add_u8(&mut res, Opecodes::OverWriteSP as u8);
                            add_i64(&mut res, var.0);
                        }
                        _ => return Err(format!("expected type was 'identifier'.")),
                    }
                }
                "lambda" => {
                    if options.len() == 0 {
                        return Err("statement 'lambda' needs argments list.".to_string());
                    }
                    if options.len() > 2 {
                        return Err(format!(
                            "statement 'lambda' takes 1 options but {} options was supplied.",
                            options.len()
                        ));
                    }

                    let mut compile_point = 0;
                    let mut is_stack_pushed = false;
                    match get_identifier_list(options[0].clone()) {
                        Ok(argments) => {
                            let mut hash: HashMap<String, (i64, String)> = HashMap::default();
                            for (i, var) in argments.iter().enumerate() {
                                hash.insert(var.to_string(), (-(i as i64 + 1) * 8, "".to_string()));
                            }
                            environment.stack.push(hash);
                            is_stack_pushed = true;
                            compile_point = 1;

                            if options.len() == 1 {
                                return Err("statement 'lambda' needs program.".to_string());
                            }
                        }
                        Err(_) => {}
                    }

                    unsafe {
                        add_u8(&mut res, Opecodes::PushS64 as u8);
                        let jump_pos = res.len();
                        let jump_pos_as_real = CURRENT_POS;
                        add_i64(&mut res, 0);

                        add_u8(&mut res, Opecodes::Jump as u8);

                        add_u8(&mut res, Opecodes::SetFP as u8); // FP設定

                        let (bytes, ret_type) = options[compile_point].compile(environment)?;
                        res.extend(bytes);
                        return_type = ret_type;

                        add_u8(&mut res, Opecodes::SetRET as u8); // リターンする値を設定
                        add_u8(&mut res, Opecodes::ExportFP as u8); // 引数変数削除のため
                        if is_stack_pushed {
                            if let Some(variables) = environment.stack.last() {
                                for _ in 0..variables.len() {
                                    add_u8(&mut res, Opecodes::PopS64 as u8);
                                }
                            }
                            environment.stack.pop();
                        }

                        add_u8(&mut res, Opecodes::Jump as u8); // もとの位置に戻る

                        let current_pos_bytes: [u8; 8] = (CURRENT_POS as i64).to_le_bytes();
                        for i in 0..8 {
                            res[jump_pos as usize + i] = current_pos_bytes[i];
                        }

                        add_u8(&mut res, Opecodes::PushS64 as u8);
                        add_i64(&mut res, (jump_pos_as_real + 9) as i64);
                    }
                }
                "if" => {
                    if options.len() != 3 {
                        return Err("statement 'if' needs three options.".to_string());
                    }

                    let exp = options[0].compile(environment)?;

                    res.extend(exp.0);
                    add_u8(&mut res, Opecodes::PushS64 as u8);
                    let jump_pos_to_else = res.len();
                    add_i64(&mut res, 0);
                    add_u8(&mut res, Opecodes::IfNotJump as u8);

                    let block1 = options[1].compile(environment)?;
                    res.extend(block1.0);

                    add_u8(&mut res, Opecodes::PushS64 as u8);
                    let jump_pos_outside = res.len();
                    add_i64(&mut res, 0);
                    add_u8(&mut res, Opecodes::Jump as u8);

                    unsafe {
                        let bytes = (CURRENT_POS as i64).to_le_bytes();
                        for i in 0..8 {
                            res[jump_pos_to_else + i] = bytes[i];
                        }

                        let block2 = options[2].compile(environment)?;
                        res.extend(block2.0);

                        let bytes = (CURRENT_POS as i64).to_le_bytes();
                        for i in 0..8 {
                            res[jump_pos_outside + i] = bytes[i];
                        }

                        if block1.1 == block2.1 {
                            return_type = block1.1.clone();
                        }
                    }
                }
                _ => return Err(format!("unknown statement '{}'.", statement)),
            },
            AstNode::ValueInteger(num) => {
                add_u8(&mut res, Opecodes::PushS64 as u8);
                add_i64(&mut res, *num);
                return_type = "integer".to_string();
            }
            AstNode::ValueFloat(num) => {
                add_u8(&mut res, Opecodes::PushS64 as u8);
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
                    if codes.len() == 0 {
                        return Ok((res, return_type));
                    }

                    let mut start_compile_point = 0;
                    let mut is_stack_pushed = false;
                    match get_identifier_list(codes[0].clone()) {
                        Ok(local_variables) => {
                            let mut hash: HashMap<String, (i64, String)> = HashMap::default();
                            for (i, var) in local_variables.iter().enumerate() {
                                hash.insert(var.to_string(), (i as i64 * 8, "".to_string()));
                                add_u8(&mut res, Opecodes::PushS64 as u8);
                                add_u64(&mut res, 0);
                            }
                            environment.stack.push(hash);
                            is_stack_pushed = true;
                            start_compile_point = 1;
                        }
                        Err(_) => {}
                    }

                    for code in codes[start_compile_point..].iter() {
                        let (bytes, ret_type) = code.compile(environment)?;
                        res.extend(bytes);
                        return_type = ret_type;
                    }

                    if is_stack_pushed {
                        if let Some(variables) = environment.stack.last() {
                            for _ in 0..variables.len() {
                                add_u8(&mut res, Opecodes::PopS64 as u8);
                            }
                        }
                        environment.stack.pop();
                    }
                }
                _ => return Err(format!("unknow list node '{}'.", name)),
            },
            AstNode::Identifier(str) => {
                let var = environment.find(str.clone())?;
                add_u8(&mut res, Opecodes::CopySP as u8);
                add_i64(&mut res, var.0);
                return_type = var.1.clone();
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
                    if !check_type(a.1.clone(), "integer".to_string()) {
                        return Err(format!("expected type integer, but found type {}.", a.1));
                    }

                    res.extend(a.0);
                    add_u8(&mut res, Opecodes::OutputI as u8);
                    return_type = "integer".to_string();
                }
                "addf" => {
                    let (a, b) = get_binop_args(args, "float".to_string(), environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::AddF as u8);
                    return_type = "float".to_string();
                }
                "subf" => {
                    let (a, b) = get_binop_args(args, "float".to_string(), environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::SubF as u8);
                    return_type = "float".to_string();
                }
                "mulf" => {
                    let (a, b) = get_binop_args(args, "float".to_string(), environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::MulF as u8);
                    return_type = "float".to_string();
                }
                "divf" => {
                    let (a, b) = get_binop_args(args, "float".to_string(), environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::DivF as u8);
                    return_type = "float".to_string();
                }
                "modf" => {
                    let (a, b) = get_binop_args(args, "float".to_string(), environment)?;
                    res.extend(a.0);
                    res.extend(b.0);
                    add_u8(&mut res, Opecodes::ModF as u8);
                    return_type = "float".to_string();
                }
                "printf" => {
                    if args.len() != 1 {
                        return Err(format!(
                            "this function takes 1 arguments but {} argument was supplied.",
                            args.len()
                        ));
                    }
                    let a = args[0].compile(environment)?;
                    if !check_type(a.1.clone(), "float".to_string()) {
                        return Err(format!("expected type float, but found type {}.", a.1));
                    }

                    res.extend(a.0);
                    add_u8(&mut res, Opecodes::OutputF as u8);
                    return_type = "float".to_string();
                }
                _ => unsafe {
                    add_u8(&mut res, Opecodes::PushFP as u8);

                    add_u8(&mut res, Opecodes::PushS64 as u8);
                    let jump_pos = res.len(); // 戻る場所を指定
                    add_u64(&mut res, 0);

                    for arg in args.iter().rev() {
                        let a = arg.compile(environment)?;
                        res.extend(a.0);
                    }

                    let func_pos = environment.find(func.clone())?;
                    add_u8(&mut res, Opecodes::CopySP as u8);
                    add_i64(&mut res, func_pos.0);
                    add_u8(&mut res, Opecodes::Jump as u8);

                    return_type = func_pos.1.clone();

                    println!("curpos:{}", CURRENT_POS);
                    let return_pos_bytes: [u8; 8] = (CURRENT_POS as i64).to_le_bytes();
                    for i in 0..8 {
                        res[jump_pos + i] = return_pos_bytes[i];
                    }

                    add_u8(&mut res, Opecodes::ResetFP as u8);
                    add_u8(&mut res, Opecodes::PushRET as u8); // 戻り値をスタックにプッシュ
                },
            },
        }

        Ok((res, return_type))
    }
}

fn bytes_to_i64(bytes: &[u8], start: usize) -> Result<i64, String> {
    // 指定位置から8バイト取り出せるかチェック
    if start + 8 <= bytes.len() {
        let slice = &bytes[start..start + 8]; // スライスを取得
        let array: [u8; 8] = slice.try_into().unwrap(); // [u8; 8] に変換
        Ok(i64::from_le_bytes(array)) // リトルエンディアンで変換
    } else {
        Err("could not find number.".to_string())
    }
}

struct Stack {
    pub sp: usize,
    pub stack: [u8; 100000],
}
impl Stack {
    pub fn print(&self) {
        for i in 0..self.sp {
            print!("{} ", self.stack[i]);
        }
        println!("");
    }
    pub fn push64(&mut self, code: [u8; 8]) {
        for i in 0..8 {
            self.stack[self.sp + i] = code[i];
        }
        self.sp += 8;
    }
    pub fn pop64(&mut self) -> [u8; 8] {
        let mut res: [u8; 8] = [0; 8];
        for i in 0..8 {
            res[i] = self.stack[self.sp - 8 + i];
        }
        self.sp -= 8;
        res
    }
    pub fn get64(&self, point: i64) -> [u8; 8] {
        let mut res: [u8; 8] = [0; 8];
        for i in 0..8 {
            res[i] = self.stack[point as usize + i];
        }
        res
    }
    pub fn set64(&mut self, value: [u8; 8], point: i64) {
        for i in 0..8 {
            self.stack[point as usize + i] = value[i];
        }
    }
}

pub fn execute_vm(code: Vec<u8>) -> Result<String, String> {
    let mut i: u32 = 0;
    let mut stack: Stack = Stack {
        sp: 0,
        stack: [0; 100000],
    };
    let mut fp: i64 = 0;
    let mut ret: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    let mut res = "".to_string();
    loop {
        print!("{}=>", i);
        if let Some(&byte) = code.get(i as usize) {
            if let Some(opcode) = (byte as u8).try_into().ok() {
                match opcode {
                    Opecodes::CopySP => {
                        let slice = &code[(i as usize + 1)..(i as usize + 9)]; // スライスを取得
                        let array: [u8; 8] = slice.try_into().unwrap(); // [u8; 8] に変換
                        stack.push64(stack.get64(fp + i64::from_le_bytes(array)));
                        i += 9;
                    }
                    Opecodes::OverWriteSP => {
                        let value = stack.pop64();
                        let slice = &code[(i as usize + 1)..(i as usize + 9)]; // スライスを取得
                        let array: [u8; 8] = slice.try_into().unwrap(); // [u8; 8] に変換
                        stack.set64(value, fp + i64::from_le_bytes(array));
                        stack.push64(value);
                        i += 9;
                    }
                    Opecodes::PushS32 => {}
                    Opecodes::PushS64 => {
                        let slice = &code[(i as usize + 1)..(i as usize + 9)]; // スライスを取得
                        let array: [u8; 8] = slice.try_into().unwrap(); // [u8; 8] に変換
                        stack.push64(array);

                        i += 9;
                    }
                    Opecodes::PopS32 => {}
                    Opecodes::PopS64 => {
                        stack.pop64();
                        i += 1;
                    }
                    Opecodes::SaveR => {}
                    Opecodes::AddI => {
                        let value1 = stack.pop64();
                        let value2 = stack.pop64();
                        stack.push64(
                            (i64::from_le_bytes(value2) + i64::from_le_bytes(value1)).to_le_bytes(),
                        );
                        i += 1;
                    }
                    Opecodes::SubI => {
                        let value1 = stack.pop64();
                        let value2 = stack.pop64();
                        stack.push64(
                            (i64::from_le_bytes(value2) - i64::from_le_bytes(value1)).to_le_bytes(),
                        );
                        i += 1;
                    }
                    Opecodes::MulI => {
                        let value1 = stack.pop64();
                        let value2 = stack.pop64();
                        stack.push64(
                            (i64::from_le_bytes(value2) * i64::from_le_bytes(value1)).to_le_bytes(),
                        );
                        i += 1;
                    }
                    Opecodes::DivI => {
                        let value1 = stack.pop64();
                        let value2 = stack.pop64();
                        stack.push64(
                            (i64::from_le_bytes(value2) / i64::from_le_bytes(value1)).to_le_bytes(),
                        );
                        i += 1;
                    }
                    Opecodes::ModI => {
                        let value1 = stack.pop64();
                        let value2 = stack.pop64();
                        stack.push64(
                            (i64::from_le_bytes(value2) % i64::from_le_bytes(value1)).to_le_bytes(),
                        );
                        i += 1;
                    }
                    Opecodes::OutputI => {
                        let value = stack.pop64();
                        println!("{}", i64::from_le_bytes(value));
                        res += &format!("{}\n", i64::from_le_bytes(value));
                        stack.push64(value);
                        i += 1;
                    }
                    Opecodes::Jump => {
                        let pos = stack.pop64();
                        let pos4 = [pos[0], pos[1], pos[2], pos[3]];
                        i = u32::from_le_bytes(pos4);
                        println!("{}", i);
                    }
                    Opecodes::SetFP => {
                        println!("fp:{}->", fp);
                        fp = stack.sp as i64;
                        println!("{}", fp);
                        i += 1;
                    }
                    Opecodes::SetRET => {
                        let r = stack.pop64();
                        for i in 0..8 {
                            ret[i] = r[i];
                        }
                        i += 1;
                    }
                    Opecodes::ResetFP => {
                        println!("fp:{}->", fp);
                        fp = i64::from_le_bytes(stack.pop64());
                        println!("{}", fp);
                        i += 1;
                    }
                    Opecodes::PushRET => {
                        stack.push64(ret);
                        i += 1;
                    }
                    Opecodes::IfNotJump => {
                        let pos = stack.pop64();
                        let r = stack.pop64();
                        let pos4 = [pos[0], pos[1], pos[2], pos[3]];
                        if i64::from_le_bytes(r) == 0 {
                            i = u32::from_le_bytes(pos4);
                        } else {
                            i += 1;
                        }
                    }
                    Opecodes::ExportFP => {
                        stack.sp = fp as usize;
                        i += 1;
                    }
                    Opecodes::PushFP => {
                        stack.push64(fp.to_le_bytes());
                        i += 1;
                    }
                    Opecodes::AddF => {
                        let value1 = stack.pop64();
                        let value2 = stack.pop64();
                        stack.push64(
                            (f64::from_le_bytes(value2) + f64::from_le_bytes(value1)).to_le_bytes(),
                        );
                        i += 1;
                    }
                    Opecodes::SubF => {
                        let value1 = stack.pop64();
                        let value2 = stack.pop64();
                        stack.push64(
                            (f64::from_le_bytes(value2) - f64::from_le_bytes(value1)).to_le_bytes(),
                        );
                        i += 1;
                    }
                    Opecodes::MulF => {
                        let value1 = stack.pop64();
                        let value2 = stack.pop64();
                        stack.push64(
                            (f64::from_le_bytes(value2) * f64::from_le_bytes(value1)).to_le_bytes(),
                        );
                        i += 1;
                    }
                    Opecodes::DivF => {
                        let value1 = stack.pop64();
                        let value2 = stack.pop64();
                        stack.push64(
                            (f64::from_le_bytes(value2) / f64::from_le_bytes(value1)).to_le_bytes(),
                        );
                        i += 1;
                    }
                    Opecodes::ModF => {
                        let value1 = stack.pop64();
                        let value2 = stack.pop64();
                        stack.push64(
                            (f64::from_le_bytes(value2) % f64::from_le_bytes(value1)).to_le_bytes(),
                        );
                        i += 1;
                    }
                    Opecodes::OutputF => {
                        let value = stack.pop64();
                        println!("{}", i64::from_le_bytes(value));
                        res += &format!("{}\n", f64::from_le_bytes(value));
                        stack.push64(value);
                        i += 1;
                    }
                    Opecodes::End => {
                        return Ok(res);
                    }
                }
            } else {
                return Err(format!("invalid opcode {:#X}", byte));
            }
        }
    }
}
