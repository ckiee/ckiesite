#[derive(PartialEq, Debug, Clone)]
pub enum AstNode {
    Directive(String, String),
    Heading { level: u16, title: BlockExprTree, children: AbstractSyntaxTree },
    BlockExprs(BlockExprTree)
}

#[derive(PartialEq, Debug, Clone)]
pub enum BlockExprNode {
    // Text(String),
    Char(char),
    Bold(BlockExprTree),
    Linespace
}

pub type BlockExprTree = Vec<BlockExprNode>;
pub type AbstractSyntaxTree = Vec<AstNode>;
