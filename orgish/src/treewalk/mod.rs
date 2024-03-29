///
/// This module walks the AST and outputs HTML.
/// It is all BAD and EVIL since it assumes the input is safe. This is okay for now, but TODO.
///
use anyhow::{anyhow, bail, Result};
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};
use lazy_static::lazy_static;
use tracing::trace;

use crate::parse::{
    AstNode, BackrefAstNode, BlockExprNode, BlockExprTree, BlockType, Directive, LinkTarget,
    OutputTo, PassedSyntaxTree, RenderGroup, Route,
};

#[derive(Default, Debug)]
pub struct ParseBuffers {
    pub main: String,
    pub nav: Vec<String>,
}

pub fn ast_to_html_string(nodes: &PassedSyntaxTree, to: OutputTo) -> Result<ParseBuffers> {
    let mut buffers = ParseBuffers::default();
    for node in nodes {
        let (mut htmls, res_to) = match ast_node_to_html_string(node, to)? {
            NodeToHtmlResult::Single(html, res_to) => (vec![html], res_to),
            NodeToHtmlResult::Many(htmls, res_to) => (htmls, res_to),
        };

        match res_to {
            OutputTo::Main => buffers.main.push_str(&htmls.join("")),
            OutputTo::Nav => buffers.nav.append(&mut htmls),
        };
    }

    Ok(buffers)
}

// TODO : remove?
pub enum NodeToHtmlResult {
    Single(String, OutputTo),
    Many(Vec<String>, OutputTo),
}

impl ParseBuffers {
    fn output(self, to: &OutputTo) -> String {
        match to {
            OutputTo::Main => self.main,
            OutputTo::Nav => self.nav.join(""),
        }
    }
}


lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

#[tracing::instrument]
fn ast_node_to_html_string(node: &BackrefAstNode, to: OutputTo) -> Result<NodeToHtmlResult> {
    let defr = to.is_using_default_rendering();
    let nav = to == OutputTo::Nav;

    Ok(match &node.inner {
        // generic
        AstNode::Heading {
            children,
            level,
            title,
            routing, // TODO use this to link?
        } => match OutputTo::from_route(routing.clone()) {
            Some(OutputTo::Main) | None => NodeToHtmlResult::Single(
                format!(
                    // In HTML headings do not have children as in our AST.
                    "<h{level} {id}>{title}</h{level}>{children}",
                    level = level,
                    title = bet_to_html_string(title)?,
                    children =
                        ast_to_html_string(children, OutputTo::Main)?.output(&OutputTo::Main),
                    id = match routing {
                        Some(_route) => "TODO".to_string(),
                        None => "".to_string(),
                    }
                ),
                OutputTo::Main,
            ),
            Some(OutputTo::Nav) => NodeToHtmlResult::Many(
                ast_to_html_string(children, OutputTo::Nav)?.nav,
                OutputTo::Nav,
            ),
        },

        //  nav; special navbar rendering
        AstNode::ListItem(_, ast) if nav => NodeToHtmlResult::Single(
            format!(
                "{}",
                ast_to_html_string(ast, to)?.output(&to)
            ),
            to,
        ),

        AstNode::Block((_, bet)) if nav => {
            NodeToHtmlResult::Single(bet_to_html_string(bet)?, to)
        }


        //  main; normal html rendering
        AstNode::Directive(d) if defr => NodeToHtmlResult::Single(
            match d {
                Directive::Raw(_, _) => unreachable!(),
                // TODO Meh, maybe return Result<Option<String>>
                _ => "".to_string(),
            },
            to,
        ),

        AstNode::Block((BlockType::Block, bet)) if defr => {
            NodeToHtmlResult::Single(format!("<p>{}</p>", bet_to_html_string(bet)?), to)
        }

        AstNode::Block((BlockType::Inline, bet)) if defr => {
            NodeToHtmlResult::Single(bet_to_html_string(bet)?, to)
        }

        AstNode::ListItem(_, ast) if defr => NodeToHtmlResult::Single(
            format!(
                "<ul><li>{}</li></ul>",
                ast_to_html_string(ast, to)?.output(&to)
            ),
            to,
        ),

        AstNode::WarningBlock((_, bet)) if defr => {
            NodeToHtmlResult::Single(bet_to_html_string(bet)?, to)
        }

        AstNode::HorizRule if defr => NodeToHtmlResult::Single("<hr>".to_string(), to),

        AstNode::SourceBlock { language, code } if defr => {
            let syntect_lang = match language {
                x if x == "rust" => "Rust",
                _ => language,
            };

            let syntax = SYNTAX_SET
                .find_syntax_by_name(syntect_lang)
                .or_else(|| SYNTAX_SET.find_syntax_by_extension(syntect_lang))
                // Nothing else worked, so we fall back to plain text..
                .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

            let theme = THEME_SET.themes.get("base16-ocean.light").unwrap();

            NodeToHtmlResult::Single(
                highlighted_html_for_string(code, &SYNTAX_SET, syntax, theme),
                to,
            )
        }
        _ => bail!("unimplemented HTML generation for node: {:#?}", node.inner),
    })
}

// block expr tree
#[tracing::instrument]
pub fn bet_to_html_string(nodes: &BlockExprTree) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    for node in nodes {
        buf.push_str(&block_expr_to_html_string(node)?);
    }
    Ok(buf)
}

fn block_expr_to_html_string(node: &BlockExprNode) -> Result<String> {
    let unreachable = Err(anyhow!(
        "illegal node {:?}; parser pass should have eliminated this",
        node
    ));
    match node {
        BlockExprNode::Bold(bet) => Ok(format!("<strong>{}</strong>", bet_to_html_string(bet)?)),
        BlockExprNode::Char(c) => Ok(c.to_string()),
        BlockExprNode::Linespace | BlockExprNode::NonbreakingSpace(_) => unreachable,
        BlockExprNode::Italic(bet) => Ok(format!("<em>{}</em>", bet_to_html_string(bet)?)),
        BlockExprNode::Underline(bet) => Ok(format!(
            "<span class=\"underline\">{}</span>",
            bet_to_html_string(bet)?
        )),
        BlockExprNode::Strikethrough(bet) => Ok(format!("<del>{}</del>", bet_to_html_string(bet)?)),
        BlockExprNode::Code(verbatim) => Ok(format!(r#"<span class="code">{}</span>"#, verbatim)),
        BlockExprNode::Link(url, maybe_bet) => Ok(format!(
            r#"<a href="{}">{}</a>"#,
            match url {
                LinkTarget::External(u) => u,
                LinkTarget::Heading { title: _t } => "TODO",
            },
            match maybe_bet {
                Some(bet) => bet_to_html_string(bet)?,
                None => panic!("unimplemented"),
            }
        )),
        BlockExprNode::HeaderRouting(_hr) => unreachable,
        BlockExprNode::Warning(bet) => Ok(format!(
            r#"<div class="warning">{}</div>"#,
            bet_to_html_string(bet)?
        )),
        BlockExprNode::FloatToggle(bet) => Ok(format!(
            r#"<span class="float">{}</span>"#,
            bet_to_html_string(bet)?
        )),
    }
}
