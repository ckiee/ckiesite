use std::{iter::Peekable, slice::Iter};

use tracing::debug;

use super::{data::AstNode, AbstractSyntaxTree, BlockExprNode, BlockExprTree};

pub enum StopAt {
    NextHeadingWithLevel(u16),
    Eof,
}

pub fn flat_nodes_to_tree(
    nodes: &mut Peekable<Iter<AstNode>>,
    stop_at: StopAt,
) -> AbstractSyntaxTree {
    let mut out: AbstractSyntaxTree = vec![];
    while let Some(node) = {
        match stop_at {
            StopAt::NextHeadingWithLevel(target_level) => match nodes.peek() {
                Some(AstNode::Heading { level, .. }) if *level <= target_level => None,
                Some(_) | None => nodes.next(),
            },
            StopAt::Eof => nodes.next(),
        }
    } {
        match node {
            AstNode::Heading {
                children: _,
                level,
                title,
            } => out.push(AstNode::Heading {
                level: *level,
                title: bet_pass(&mut title.iter().peekable(), &mut Default::default()),
                children: flat_nodes_to_tree(nodes, StopAt::NextHeadingWithLevel(*level)),
            }),

            // Optimization: Linespace is not very useful in the final AST
            AstNode::Block(_, bet) if bet == &vec![BlockExprNode::Linespace] => {}

            AstNode::Block(ty, bet) => out.push(AstNode::Block(
                ty.clone(),
                bet_pass(&mut bet.iter().peekable(), &mut Default::default()),
            )),

            other => out.push(other.clone()),
        }
    }

    out
}

#[derive(Default)]
struct BetPassState {
    inside_nbsp: bool,
}

impl BetPassState {
    fn inside_nbsp(&mut self) -> &mut Self {
        self.inside_nbsp = true;
        self
    }
}

fn bet_pass(nodes: &mut Peekable<Iter<BlockExprNode>>, state: &mut BetPassState) -> BlockExprTree {
    let mut out: BlockExprTree = vec![];
    while let Some(node) = nodes.next() {
        match node {
            BlockExprNode::NonbreakingSpace(bet) => out.append(&mut bet_pass(
                &mut bet.iter().peekable(),
                state.inside_nbsp(),
            )),
            BlockExprNode::Char(' ') if state.inside_nbsp => {
                out.push(BlockExprNode::Char('\u{20}'))
            }
            // hey hey, wouldn't it be neat if links like "Org Mode" wouldn't wrap around?
            // we don't really have to add a BEN::NbSp(), which is a bit weird ~~but skips a 2nd pass!~~
            // This doesn't work because I forgot that we have to recurse `bet`. Oops, TODO.
            // BlockExprNode::Link(link, bet)
            //     if bet.all(|s| {match s.chars().next() {
            //         None => false,
            //         Some(ch) => ch.is_uppercase(),
            //     }}) => {
            //         debug!("BET in BEN::Link {:#?}", bet);
            //         out.push(BlockExprNode::Link(link.to_string(), bet_pass(&mut bet.iter().peekable(), state.inside_nbsp())))
            //     },

            BlockExprNode::Bold(bet) => out.push(BlockExprNode::Bold(bet_pass(
                &mut bet.iter().peekable(),
                state,
            ))),
            BlockExprNode::Italic(bet) => out.push(BlockExprNode::Italic(bet_pass(
                &mut bet.iter().peekable(),
                state,
            ))),
            BlockExprNode::Underline(bet) => out.push(BlockExprNode::Underline(bet_pass(
                &mut bet.iter().peekable(),
                state,
            ))),
            BlockExprNode::Strikethrough(bet) => out.push(BlockExprNode::Strikethrough(bet_pass(
                &mut bet.iter().peekable(),
                state,
            ))),
            BlockExprNode::Link(url, bet) => out.push(BlockExprNode::Link(
                url.to_string(),
                bet_pass(&mut bet.iter().peekable(), state),
            )),
            other => out.push(other.clone()),
        }
    }

    out
}
