use std::{iter::Peekable, slice::Iter};

use anyhow::Result;

use super::{
    data::AstNode, AbstractSyntaxTree, BackrefAstNode, BlockExprNode, BlockExprTree, Directive,
    Route,
};

#[derive(PartialEq, Debug, Clone)]
pub enum StopReq {
    NextHeadingWithLevel(u16),
    NextListWithLevel(u16),
    AnyHeading,
    Linespace
}

/// Transform a flat node stream into a tree by recursing.
///
/// Back references to the parent nodes do not exist in this function's output.
/// Instead, they are created in a separate pass
#[tracing::instrument]
pub fn flat_nodes_to_tree(
    nodes: &mut Peekable<Iter<AstNode>>,
    stop_reqs: Vec<StopReq>,
) -> Result<AbstractSyntaxTree> {
    let mut out: AbstractSyntaxTree = vec![];

    while let Some(node) = {
        let fulfilled_reqs = stop_reqs.iter().filter(|req| match req {
            StopReq::NextHeadingWithLevel(target_level) => match nodes.peek() {
                Some(AstNode::Heading { level, .. }) if level <= target_level => true,
                Some(_) | None => false,
            },
            StopReq::NextListWithLevel(target_level) => match nodes.peek() {
                Some(AstNode::ListItem(level, _)) if level <= target_level => true,
                Some(_) | None => false,
            },
            StopReq::AnyHeading => match nodes.peek() {
                Some(AstNode::Heading {..}) => true,
                _ => false
            },
            StopReq::Linespace => match nodes.peek() {
                Some(AstNode::Block((_, bet))) if bet == &vec![BlockExprNode::Linespace] => true,
                _ => false
            },
        }).count();

        if fulfilled_reqs != 0 {
            None
        } else {
            nodes.next()
        }
    } {
        match node {
            AstNode::Heading {
                children: _,
                level,
                title,
                routing: _,
            } => {
                // we have to do one more mini-pass to find this goddarn HeaderRouting thing
                // because this is a bit more convenient for the user (:/path: can be at the end of the header title)
                // TODO move into pass2 maybe?
                let routing = title.iter().find_map::<Route, _>(|n| match n {
                    BlockExprNode::HeaderRouting(route) => Some(route.clone()),
                    _ => None,
                });

                let mut title_bet = bet_pass(
                    &mut title.iter().peekable(),
                    &mut BetPassState::new_with_ast_node(node.clone()),
                )?;

                // XXX: Since we do not parse that nicely, we have
                // to do this little hack..
                //
                // For example,
                //  ** Blah blah :world:
                // where BET is "Blah blah " instead of "Blah blah"
                if let Some(BlockExprNode::Char(' ')) = title_bet.last() {
                    title_bet.pop();
                }

                let mut new_stop_reqs = stop_reqs.clone();
                new_stop_reqs.push(StopReq::NextHeadingWithLevel(*level));
                out.push(AstNode::Heading {
                    level: *level,
                    title: title_bet,
                    // Backreferences in children do not exist yet. We do that in another phase.
                    children: flat_nodes_to_tree(nodes, new_stop_reqs)?
                        .into_iter()
                        .map(BackrefAstNode::new_unref)
                        .collect::<Vec<_>>(),
                    routing,
                })
            }

            AstNode::ListItem(level, _) => {
                let mut new_stop_reqs = stop_reqs.clone();
                new_stop_reqs.push(StopReq::NextListWithLevel(*level));
                new_stop_reqs.push(StopReq::AnyHeading);
                new_stop_reqs.push(StopReq::Linespace);

                out.push(AstNode::ListItem(
                    *level,
                    // Backreferences in children do not exist yet. We do that in another phase.
                    flat_nodes_to_tree(nodes, new_stop_reqs)?
                        .into_iter()
                        .map(BackrefAstNode::new_unref)
                        .collect::<Vec<_>>(),
                ))
            }

            // Optimization: Linespace is not very useful in the final AST,
            // but it is used in this pass for ListItem termination.
            AstNode::Block((_, bet)) if bet == &vec![BlockExprNode::Linespace] => {}

            AstNode::Block((ty, bet)) => out.push(AstNode::Block((
                ty.clone(),
                bet_pass(
                    &mut bet.iter().peekable(),
                    &mut BetPassState::new_with_ast_node(node.clone()),
                )?,
            ))),

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

#[derive(Debug)]
struct BetPassState {
    inside_nbsp: bool,
    #[allow(unused)]
    // keeping this for future things, doesn't really hurt anyone and it's quite annoying to remove it just to add it again later
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

#[tracing::instrument]
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
                url.clone(),
                match maybe_bet.as_ref() {
                    Some(bet) => Some(bet_pass(&mut bet.iter().peekable(), state)?),
                    None => None,
                },
            )),
            // we discard HeaderRoutings since they should have already been looked ahead for
            BlockExprNode::HeaderRouting(..) => {}
            other => out.push(other.clone()),
        }
    }

    Ok(out)
}
