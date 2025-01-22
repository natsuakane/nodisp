pub enum AstNode {
    Statement(Vec<AstNode>),
    ValueNum(f64),
    ValueStr(String),
    Function(Vec<AstNode>),
    List(Vec<AstNode>),
    Identifier(String),
}
