use anyhow::Result;
use std::fmt::{Display, Pointer, Write};

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
        children: Vec<BackrefAstNode>,
        routing: Option<Route>,
    },
    Block(BetBlock),
    /// Equivalent to html <hr>
    HorizRule,
    ListItem(BetBlock)
}


#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct BackrefAstNode {
    /// an index into the PassedSyntaxTree this is in
    pub parent_idx: usize,
    pub inner: AstNode,
    pub render_group: Option<RenderGroup>
}

pub type PassedSyntaxTree = Vec<BackrefAstNode>;

// This is cursed. The AstNode::{Block,ListItem} implies
// the inline-ness instead of BetBlock and BlockType, and yet,
// this is still here. TODO Remove.
pub type BetBlock = (BlockType, BlockExprTree);

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum BlockType {
    Block,
    Inline,
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
    Link(LinkTarget, Option<BlockExprTree>),
    /// One or more newlines
    Linespace,
    HeaderRouting(Route),
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Directive {
    Id(String),
    Title(String),
    /// Pre-pass datatype
    Raw(String, String),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum LinkTarget {
    Heading { title: String },
    External(String),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Route {
    Page(String),             // index
    Section(String),          // #how
    RenderGroup(RenderGroup), // @rg
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum RenderGroup {
    Nav, // nav
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

impl Display for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let AstNode::Block((_, bet)) = self {
            bet.fmt(f)?
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

pub fn stringify_bet(bet: &Vec<BlockExprNode>) -> Result<String> {
    let mut buf = String::new();
    for ben in bet {
        write!(&mut buf, "{}", ben)?;
    }
    Ok(buf)
}

impl BackrefAstNode {
    pub fn new_unref(with: AstNode) -> Self {
        Self {
            parent_idx: 0,
            inner: with,
            render_group: None
        }
    }
}
