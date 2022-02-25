#[derive(PartialEq, Debug, Clone)]
pub enum AstNode {
    Directive(String, String),
    Heading { level: u16, title: Vec<BlockExprNode>, children: Vec<AstNode> },
    BlockExprs(Vec<BlockExprNode>)
}

#[derive(PartialEq, Debug, Clone)]
pub enum BlockExprNode {
    // Text(String),
    Char(char),
    Bold(Vec<BlockExprNode>),
    Linespace
}
