use anyhow::Result;
use std::{iter::Peekable, rc::Weak, slice::Iter};

use super::{AstNode, BackreferencedAst, BackreferencedAstNode};

pub fn add_backreferences(
    nodes: &mut Peekable<Iter<BackreferencedAstNode>>,
    parent: Option<Weak<BackreferencedAstNode>>,
) -> Result<BackreferencedAst> {
    let mut out: BackreferencedAst = vec![];
    let tl_parent = parent;
    while let Some(node) = nodes.next() {
        match node {
            BackreferencedAstNode {
                inner:
                    AstNode::Heading {
                        children,
                        level,
                        title,
                        routing,
                    },
                parent: _,
            } => {
                out.push(BackreferencedAstNode {
                    parent: tl_parent.clone(),
                    inner: AstNode::Heading {
                        level: *level,
                        title: title.clone(),
                        routing: routing.clone(),
                        children: add_backreferences(
                            &mut children.iter().peekable(), tl_parent.clone(),
                        )?,
                    },
                })
            }
            other => {
                let mut new = other.clone();
                new.parent = tl_parent.clone();
                out.push(new);
            }
        }
    }

    Ok(out)
}
