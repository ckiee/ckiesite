#[derive(PartialEq, Debug, Clone)]
pub enum AstNode {
    Directive(String, String),
    /// Equivalent to html <hx>
    Heading { level: u16, title: BlockExprTree, children: AbstractSyntaxTree },
    BlockExprs(BlockExprTree),
    /// Equivalent to html <p>
    Block(AbstractSyntaxTree),
}

#[derive(PartialEq, Debug, Clone)]
pub enum BlockExprNode {
    Char(char),
    /// Equivalent to html <b>
    Bold(BlockExprTree),
    /// One or more newlines
    Linespace
}

impl BlockExprNode {
    /// Returns `true` if the block expr node is [`Char`].
    ///
    /// [`Char`]: BlockExprNode::Char
    pub fn is_char(&self) -> bool {
        matches!(self, Self::Char(..))
    }
}

pub type BlockExprTree = Vec<BlockExprNode>;
pub type AbstractSyntaxTree = Vec<AstNode>;
