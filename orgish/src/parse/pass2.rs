use anyhow::{Result};
use std::{iter::Peekable, vec::IntoIter};

use super::{AbstractSyntaxTree, AstNode, BackrefAstNode, PassedSyntaxTree};

fn add_backreferences_internal(
    nodes: &mut Peekable<IntoIter<BackrefAstNode>>,
    parent_idx: usize,
) -> Result<PassedSyntaxTree> {
    let mut out: Vec<BackrefAstNode> = vec![];
    for mut node in nodes.by_ref() {
        node.parent_idx = parent_idx;
        if let AstNode::Heading { children, .. } = &mut node.inner {
            let orig_ch = children.clone();
            *children =
                add_backreferences_internal(&mut orig_ch.into_iter().peekable(), parent_idx)?;
        }
        out.push(node);
    }

    Ok(out)
}

pub fn add_backreferences(nodes: AbstractSyntaxTree) -> Result<PassedSyntaxTree> {
    let backref_ready_nodes = nodes
        .into_iter()
        .map(BackrefAstNode::new_unref)
        .collect::<Vec<_>>();

    add_backreferences_internal(
        &mut backref_ready_nodes.into_iter().peekable(),
        0,
    )
}
