use std::path::Path;

use anyhow::{anyhow, Result};
use orgish::{
    parse::{parse_n_pass, AstNode, Directive},
    treewalk::ast_to_html_string,
};
use tokio::{fs::File, io::AsyncReadExt};
use typed_html::{dom::DOMTree, html, text, unsafe_text};

pub struct Document {
    ast: Vec<AstNode>,
    id: String,
    title: Option<String>,
}

impl Document {
    pub async fn from_org_file(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path.as_ref()).await?;
        let mut buf = String::new();
        file.read_to_string(&mut buf).await?;
        let ast = parse_n_pass(&buf)?;
        let title = ast
            .iter()
            .filter(|n| matches!(n, AstNode::Directive(Directive::Title(_))))
            .next()
            .map(|n| match n {
                AstNode::Directive(Directive::Title(t)) => t.clone(),
                _ => unreachable!(),
            });

        Ok(Document {
            ast,
            id: path
                .as_ref()
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
                <style>

"body {
    margin: 2vw auto;
    max-width: 650px;
    line-height: 1.6;
    font-size: 18px;
    padding: 0 10px;
}
html {
    color: #444;
    background: #EEEEEE;
}
h1, h2, h3, h4, h5, h6 {
    line-height: 1.2;
}
span.underline {
    text-decoration: underline;
}
span.code {
    font-family: monospace;
}
header {
    display: flex;
    gap: 1em;
}
"

                </style>
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
