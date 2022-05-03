use std::fmt::{Display, Write, Pointer};

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum AstNode {
    SourceBlock {
        language: String,
        code: String,
    },
    Directive(Directive),
    /// Equivalent to html <hx>
    Heading {
        level: u16,
        title: BlockExprTree,
        children: AbstractSyntaxTree,
        routing: Option<HeaderRouting>,
    },
    Block(BlockType, BlockExprTree),
    /// Equivalent to html <hr>
    HorizRule,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum BlockExprNode {
    Char(char),
    Bold(BlockExprTree),
    Italic(BlockExprTree),
    Underline(BlockExprTree),
    Strikethrough(BlockExprTree),
    NonbreakingSpace(BlockExprTree),
    Code(String),
    Link(String, Option<BlockExprTree>),
    /// One or more newlines
    Linespace,
    HeaderRouting(HeaderRouting)
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum BlockType {
    Block,
    Inline,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Directive {
    Id(String),
    Title(String),
    /// Pre-pass datatype
    Raw(String, String),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct HeaderRouting {
    pub path: String, // TODO: add options
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

// impls

impl Display for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstNode::Block(_, bet) => bet.fmt(f)?,
            _ => {}
        }
        Ok(())
    }
}

impl Display for BlockExprNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &BlockExprNode::Char(c) => f.write_char(c)?,
            _ => {}
        }
        Ok(())
    }
}
