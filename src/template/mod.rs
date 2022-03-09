use typed_html::{dom::DOMTree, html, unsafe_text};

pub fn make_article_html(content: &str) -> String {
    let doc: DOMTree<String> = html!(
        <html>
            <head>
                <title>"TODO"</title>
            <style>
"body {
  margin: 40px auto;
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
"
            </style>
            </head>
            <body>
                <main>
                    { unsafe_text!(content) }
                </main>
            </body>
        </html>
    );

    format!("<!DOCTYPE html>{}", doc.to_string())
}
