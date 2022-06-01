use anyhow::{Result, anyhow};
use std::{iter::Peekable, slice::Iter, sync::Arc, sync::Weak, vec::IntoIter};

compile_error!("TODO maybe remove this pass, give up on Weak, use a index into the vec, ..");

use super::{AstNode, BackreferencedAst, BackreferencedAstNode};

fn add_backreferences_internal(
    nodes: &mut Peekable<IntoIter<BackreferencedAstNode>>,
    parent: Option<Weak<BackreferencedAstNode>>,
) -> Result<Vec<Arc<BackreferencedAstNode>>> {
    let mut out: Vec<Arc<BackreferencedAstNode>> = vec![];
    while let Some(mut node) = nodes.next() {
        node.parent = parent.clone();
        let node_arc = Arc::new(node);
        if let AstNode::Heading { children, .. } = &mut node.inner {
            let orig_ch = children.clone();
            *children = add_backreferences_internal(
                &mut orig_ch.into_iter().peekable(),
                Some(Arc::downgrade(&node_arc)),
            )?;
        }
        out.push(node_arc);
    }

    Ok(out)
}

pub fn add_backreferences(
 nopodes: &mut Peekable<IntoIter<BackreferencedAstNode>>,
) -> Result<Vec<BackreferencedAstNode>> {
    let res = add_backreferences_internal(nodes, None)?;
    let out = Vec::new();

    for arc in res {
        out.push(Arc::try_unwrap(arc).map_err(|_| anyhow!("try_unwrap failed"))?);
    }

    Ok(out)
}
