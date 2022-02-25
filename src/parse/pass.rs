use std::{iter::Peekable, slice::Iter};

use super::{data::AstNode, AbstractSyntaxTree, BlockExprNode};

pub enum StopAt {
    NextHeadingWithLevel(u16),
    Linespace,
    Eof,
}

impl StopAt {
    /// Returns `true` if the stop at is [`Linespace`].
    ///
    /// [`Linespace`]: StopAt::Linespace
    pub fn is_linespace(&self) -> bool {
        matches!(self, Self::Linespace)
    }
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
            StopAt::Linespace => match nodes.peek() {
                Some(AstNode::BlockExprs(bet)) if bet == &vec![BlockExprNode::Linespace] => None,
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
                title: title.clone(),
                children: flat_nodes_to_tree(nodes, StopAt::NextHeadingWithLevel(*level)),
            }),

            AstNode::BlockExprs(bet)
                if bet.iter().all(|n| n.is_char()) && !stop_at.is_linespace() =>
            {
                out.push(AstNode::Block(
                    vec![
                        vec![node.clone()],
                        flat_nodes_to_tree(nodes, StopAt::Linespace),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
                ))
            }
            // Optimization: Linespace is not very useful in the final AST
            AstNode::BlockExprs(bet) if bet == &vec![BlockExprNode::Linespace] => {},
            other => out.push(other.clone()),
        }
    }

    out
}
