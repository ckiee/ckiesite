use anyhow::{anyhow, Result};
use orgish::{
    parse::{parse_n_pass, AstNode, Directive},
    treewalk::ast_to_html_string,
};
use serde::{Serialize, Deserialize};
use typed_html::{dom::DOMTree, html, text, unsafe_text};

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    ast: Vec<AstNode>,
    id: String,
    title: Option<String>,
}

impl Document {
    pub fn from_org_id_file(file: &include_dir::File<'static>) -> Result<Self> {
        let ast = parse_n_pass(file.contents_utf8().ok_or(anyhow!("expected utf8"))?)?;
        let title = ast
            .iter().find(|n| matches!(n, AstNode::Directive(Directive::Title(_))))
            .map(|n| match n {
                AstNode::Directive(Directive::Title(t)) => t.clone(),
                _ => unreachable!(),
            });

        Ok(Document {
            ast,
            id: file
                .path()
                .file_stem()
                .expect("the file we opened to have a name")
                .to_owned()
                .into_string()
                .map_err(|_| anyhow!("into_string failed"))?,
            title,
        })
    }

    pub fn html_string(&self) -> Result<String> {
        ast_to_html_string(&self.ast)
    }

    pub fn render_page_html(&self) -> Result<String> {
        let doc: DOMTree<String> = html!(
            <html>
                <head>
                <title>{ text!(self.title.as_ref().unwrap_or(&self.id)) }</title>
                <link rel="stylesheet" href="/static/style.css"></link>
                </head>

                <body>
                    <header>
                        <p>"hi"</p>
                    </header>

                    <main>
                        { unsafe_text!(self.html_string()?) }
                    </main>
                </body>

            </html>
        );

        Ok(format!("<!DOCTYPE html>{}", doc))
    }
}
