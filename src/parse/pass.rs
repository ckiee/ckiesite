use std::{iter::Peekable, slice::Iter};

use super::{data::AstNode, AbstractSyntaxTree};

pub enum StopAt {
    NextHeadingWithLevel(u16),
    Eof,
}
pub fn flat_nodes_to_tree(nodes: &mut Peekable<Iter<AstNode>>, stop_at: StopAt) -> AbstractSyntaxTree {
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
                title: title.clone(),
                children: flat_nodes_to_tree(nodes, StopAt::NextHeadingWithLevel(*level)),
            }),
            other => out.push(other.clone()),
        }
    }

    out
}
