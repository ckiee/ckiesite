#[derive(PartialEq, Debug, Clone)]
pub enum AstNode {
    SourceBlock {
        language: String,
        code: String,
    },
    Directive(String, String),
    /// Equivalent to html <hx>
    Heading {
        level: u16,
        title: BlockExprTree,
        children: AbstractSyntaxTree,
    },
    Block(BlockType, BlockExprTree),
    /// Equivalent to html <hr>
    HorizRule,
}

#[derive(PartialEq, Debug, Clone)]
pub enum BlockExprNode {
    Char(char),
    Bold(BlockExprTree),
    Italic(BlockExprTree),
    Underline(BlockExprTree),
    Strikethrough(BlockExprTree),
    Code(String),
    /// One or more newlines
    Linespace,
}

#[derive(PartialEq, Debug, Clone)]
pub enum BlockType {
    Block,
    Inline,
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
