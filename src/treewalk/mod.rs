use anyhow::Result;
use syntect::{
    highlighting::ThemeSet,
    html::{
        highlighted_html_for_string,
    },
    parsing::SyntaxSet,
};

///
/// This module walks the AST and outputs HTML.
///
use crate::parse::{AbstractSyntaxTree, AstNode, BlockExprNode, BlockExprTree, BlockType};

pub fn ast_to_html_string(nodes: &AbstractSyntaxTree) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    for node in nodes {
        buf.push_str(&ast_node_to_html_string(node)?);
    }
    Ok(buf)
}

fn ast_node_to_html_string(node: &AstNode) -> Result<String> {
    Ok(match node {
        // HACK A bit messy. We should probably have a separate pass for directives and an out-of-AST map
        AstNode::Directive(title, value) if title.eq_ignore_ascii_case("title") => {
            format!("<h1>{}</h1>", value)
        }
        AstNode::Directive(_, _) => unimplemented!(), // TODO impl
        AstNode::Heading {
            children,
            level,
            title,
        } => format!(
            // In HTML headings do not have children as in our AST.
            "<h{level}>{title}</h{level}>{children}",
            level = level,
            title = bet_to_html_string(title)?,
            children = ast_to_html_string(children)?
        ),
        AstNode::Block(BlockType::Block, bet) => {
            format!("<p>{}</p>", bet_to_html_string(bet)?)
        }
        AstNode::Block(BlockType::Inline, bet) => bet_to_html_string(bet)?,
        AstNode::HorizRule => "<hr>".to_string(),
        AstNode::SourceBlock { language, code } => {
            let syntax_set = SyntaxSet::load_defaults_newlines();
            let syntect_lang = match language {
                x if x == "rust" => "Rust",
                _ => language,
            };
            let syntax = syntax_set
                .find_syntax_by_name(syntect_lang)
                .expect("didn't find src language syntax"); // TODO don't unwrap

            let theme_set = ThemeSet::load_defaults();
            let theme = theme_set.themes.get("base16-ocean.light").unwrap();

            
            highlighted_html_for_string(code, &syntax_set, syntax, theme)
        }
    })
}

// block expr tree
fn bet_to_html_string(nodes: &BlockExprTree) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    for node in nodes {
        buf.push_str(&block_expr_to_html_string(node)?);
    }
    Ok(buf)
}

fn block_expr_to_html_string(node: &BlockExprNode) -> Result<String> {
    Ok(match node {
        BlockExprNode::Bold(bet) => format!("<strong>{}</strong>", bet_to_html_string(bet)?),
        BlockExprNode::Char(c) => c.to_string(),
        BlockExprNode::Linespace => {
            panic!("illegal node; parser pass should have eliminated all linespaces")
        }
        BlockExprNode::Italic(bet) => format!("<em>{}</em>", bet_to_html_string(bet)?),
        BlockExprNode::Underline(bet) => format!(
            "<span class=\"underline\">{}</span>",
            bet_to_html_string(bet)?
        ),
        BlockExprNode::Strikethrough(bet) => {
            format!("<del>{}</del>", bet_to_html_string(bet)?)
        }
        BlockExprNode::Code(verbatim) => format!(r#"<span class="code">{}</span>"#, verbatim),
    })
}
