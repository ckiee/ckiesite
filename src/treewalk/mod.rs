use syntect::{
    highlighting::ThemeSet,
    html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use typed_html::{dom::DOMTree, html};

///
/// This module walks the AST and outputs HTML.
///
use crate::parse::{AbstractSyntaxTree, AstNode, BlockExprNode, BlockExprTree, BlockType};

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
        // HACK A bit messy. We should probably have a separate pass for directives and an out-of-AST map
        AstNode::Directive(title, value) if title.eq_ignore_ascii_case("title") => {
            Some(format!("<h1>{}</h1>", value))
        }
        AstNode::Directive(_, _) => None, // TODO impl
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
        AstNode::Block(BlockType::Block, bet) => {
            Some(format!("<p>{}</p>", bet_to_html_string(bet)))
        }
        AstNode::Block(BlockType::Inline, bet) => Some(bet_to_html_string(bet)),
        AstNode::HorizRule => Some("<hr>".to_string()),
        AstNode::SourceBlock { language, code } => {
            let syntax_set = SyntaxSet::load_defaults_newlines();
            let syntect_lang = match language {
                x if x == "rust" => "Rust",
                _ => language
            };
            let syntax = syntax_set
                .find_syntax_by_name(syntect_lang)
                .expect("didn't find src language syntax"); // TODO don't unwrap
            let class_style = ClassStyle::SpacedPrefixed { prefix: "syntect-" }; // TODO move this out to a Document struct
            let mut html_generator =
                ClassedHTMLGenerator::new_with_class_style(&syntax, &syntax_set, class_style);
            for line in LinesWithEndings::from(code) {
                html_generator.parse_html_for_line_which_includes_newline(&line);
            }
            let output_html = html_generator.finalize();
            let output_css = {
                let theme_set = ThemeSet::load_defaults();
                // TODO configurable, use Document struct same as above
                css_for_theme_with_class_style(
                    theme_set.themes.get("base16-ocean.light").unwrap(),
                    class_style,
                )
            };
            Some(format!(
                r#"<code>
<style>{}</style>
{}</code>"#,
                output_css, output_html
            ))
        }
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
        BlockExprNode::Bold(bet) => Some(format!("<strong>{}</strong>", bet_to_html_string(bet))),
        BlockExprNode::Char(c) => Some(c.to_string()),
        BlockExprNode::Linespace => {
            panic!("illegal node; parser pass should have eliminated all linespaces")
        }
        BlockExprNode::Italic(bet) => Some(format!("<em>{}</em>", bet_to_html_string(bet))),
        BlockExprNode::Underline(bet) => Some(format!(
            "<span class=\"underline\">{}</span>",
            bet_to_html_string(bet)
        )),
        BlockExprNode::Strikethrough(bet) => {
            Some(format!("<del>{}</del>", bet_to_html_string(bet)))
        },
        BlockExprNode::Code(verbatim) => Some(format!(r#"<span class="code">{}</span>"#, verbatim))
    }
}
