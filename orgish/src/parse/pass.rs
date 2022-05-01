use std::{iter::Peekable, slice::Iter};

use anyhow::{bail, Result};

use super::{data::AstNode, AbstractSyntaxTree, BlockExprNode, BlockExprTree, Directive};

pub enum StopAt {
    NextHeadingWithLevel(u16),
    Eof,
}

pub fn flat_nodes_to_tree(
    nodes: &mut Peekable<Iter<AstNode>>,
    stop_at: StopAt,
) -> Result<AbstractSyntaxTree> {
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
                routing
            } => out.push(AstNode::Heading {
                level: *level,
                title: bet_pass(
                    &mut title.iter().peekable(),
                    &mut BetPassState::new_with_ast_node(node.clone()),
                )?,
                children: flat_nodes_to_tree(nodes, StopAt::NextHeadingWithLevel(*level))?,
                routing: routing.clone()
            }),

            // Optimization: Linespace is not very useful in the final AST
            AstNode::Block(_, bet) if bet == &vec![BlockExprNode::Linespace] => {}

            AstNode::Block(ty, bet) => out.push(AstNode::Block(
                ty.clone(),
                bet_pass(
                    &mut bet.iter().peekable(),
                    &mut BetPassState::new_with_ast_node(node.clone()),
                )?,
            )),

            AstNode::Directive(dir) => match dir {
                Directive::Raw(k, v) => {
                    match match k.to_lowercase().as_str() {
                        "id" => Some(Directive::Id(v.to_string())),
                        "title" => Some(Directive::Title(v.to_string())),
                        _ => None,
                    } {
                        None => {}
                        Some(dir) => out.push(AstNode::Directive(dir)),
                    };
                }
                _ => unreachable!(),
            },

            other => out.push(other.clone()),
        }
    }

    Ok(out)
}

struct BetPassState {
    inside_nbsp: bool,
    top_level_ast_node: AstNode,
}

impl BetPassState {
    fn new_with_ast_node(top_level_ast_node: AstNode) -> Self {
        Self {
            inside_nbsp: false,
            top_level_ast_node,
        }
    }
    fn inside_nbsp(&mut self) -> &mut Self {
        self.inside_nbsp = true;
        self
    }
}

fn bet_pass(
    nodes: &mut Peekable<Iter<BlockExprNode>>,
    state: &mut BetPassState,
) -> Result<BlockExprTree> {
    let mut out: BlockExprTree = vec![];
    for node in nodes {
        // debug!("bet_pass: {:?}", &node);
        match node {
            BlockExprNode::NonbreakingSpace(bet) => out.append(&mut bet_pass(
                &mut bet.iter().peekable(),
                state.inside_nbsp(),
            )?),
            BlockExprNode::Char(' ') if state.inside_nbsp => {
                out.push(BlockExprNode::Char('\u{a0}'))
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
            )?)),
            BlockExprNode::Italic(bet) => out.push(BlockExprNode::Italic(bet_pass(
                &mut bet.iter().peekable(),
                state,
            )?)),
            BlockExprNode::Underline(bet) => out.push(BlockExprNode::Underline(bet_pass(
                &mut bet.iter().peekable(),
                state,
            )?)),
            BlockExprNode::Strikethrough(bet) => out.push(BlockExprNode::Strikethrough(bet_pass(
                &mut bet.iter().peekable(),
                state,
            )?)),
            BlockExprNode::Link(url, maybe_bet) => out.push(BlockExprNode::Link(
                url.to_string(),
                match maybe_bet.as_ref() {
                    Some(bet) => Some(bet_pass(&mut bet.iter().peekable(), state)?),
                    None => None,
                },
            )),
            other => out.push(other.clone()),
        }
    }

    Ok(out)
}
