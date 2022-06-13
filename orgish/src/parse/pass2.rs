use anyhow::Result;
use std::{iter::Peekable, vec::IntoIter};

use super::{AbstractSyntaxTree, AstNode, BackrefAstNode, PassedSyntaxTree, RenderGroup, Route};

fn add_backreferences_internal(
    nodes: &mut Peekable<IntoIter<BackrefAstNode>>,
    parent_idx: usize,
    render_group: Option<RenderGroup>,
) -> Result<PassedSyntaxTree> {
    let mut out: Vec<BackrefAstNode> = vec![];
    for mut node in nodes.by_ref() {
        node.parent_idx = parent_idx;
        node.render_group = render_group.clone();
        if let AstNode::Heading {
            children,
            ref routing,
            ..
        } = &mut node.inner
        {
            let orig_ch = children.clone();

            // If this heading has a new RenderGroup, apply it
            // Otherwise, just use the render_group we were called with
            // as above.
            let new_rg = if let Some(Route::RenderGroup(rg)) = routing {
                Some(rg.clone())
            } else {
                node.render_group
            };

            node.render_group = new_rg.clone();

            *children = add_backreferences_internal(
                &mut orig_ch.into_iter().peekable(),
                parent_idx,
                new_rg,
            )?;
        }
        out.push(node);
    }

    Ok(out)
}

// Maybe it's right but the type inference reallly doesn't like it.
#[allow(clippy::needless_collect)]
pub fn add_backreferences(nodes: AbstractSyntaxTree) -> Result<PassedSyntaxTree> {
    let backref_ready_nodes = nodes
        .into_iter()
        .map(BackrefAstNode::new_unref)
        .collect::<Vec<_>>();

    add_backreferences_internal(&mut backref_ready_nodes.into_iter().peekable(), 0, None)
}
