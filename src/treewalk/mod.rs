///
/// This module walks the AST and outputs HTML.
///
use crate::parse::{AstNode, BlockExprNode, BlockExprTree, AbstractSyntaxTree};

pub fn ast_to_html_string(nodes: &AbstractSyntaxTree) -> String {
    let mut buf = String::with_capacity(4096);
    for node in nodes {
        if let Some(html) = ast_node_to_html_string(node) {
            buf.push_str(&html);
        }
    }
    buf
}

fn ast_node_to_html_string(node: &AstNode) -> Option<String> {
    match node {
        AstNode::Directive(_, _) => None,
        AstNode::Heading {
            children,
            level,
            title,
        } => Some(format!(
            // In HTML headings do not have children as in our AST.
            "<h{level}>{title}</h{level}>{children}",
            level = level,
            title = bet_to_html_string(title),
            children = ast_to_html_string(children)
        )),
        AstNode::BlockExprs(bet) => Some(bet_to_html_string(bet)),
    }
}

// block expr tree
fn bet_to_html_string(nodes: &BlockExprTree) -> String {
    let mut buf = String::with_capacity(4096);
    for node in nodes {
        if let Some(html) = block_expr_to_html_string(node) {
            buf.push_str(&html);
        }
    }
    buf
}

fn block_expr_to_html_string(node: &BlockExprNode) -> Option<String> {
    match node {
        BlockExprNode::Bold(bet) => Some(format!("<b>{}</b>", bet_to_html_string(bet))),
        // TODO implement these two properly with a parser pass to transform into BlockExprNode::Block
        BlockExprNode::Char(c) => Some(c.to_string()),
        BlockExprNode::Linespace => Some("\n".to_string())
    }
}
