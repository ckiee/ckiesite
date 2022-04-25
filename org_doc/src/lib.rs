use std::collections::HashMap;

use anyhow::{Result,anyhow};
use include_dir::{include_dir, Dir};

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use site_common::document::Document;

static ORG_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../data/org");


// Generate something like this..
//
// fn get_org_doc(p: String) -> Option<Document> {
//     match p {
//         "hello" => Some(Document {
//             ast: todo!(),
//             id: todo!(),
//             title: todo!(),
//         }),
//         _ => None,
//     }
// }

// TODO: parse `ts`, take single string literal in and pass to include dir maybe?
#[proc_macro]
pub fn org_docs(ts: TokenStream) -> TokenStream {
    let doc_map = build_org_docs();
    let doc_ts = doc_map.iter().enumerate().map(|(path, doc)| { quote! {
        #path => // TODO finish this, serialize `doc` into TokenStream
    }});

    quote! {
        fn get_org_doc(p: AsRef<str>) -> Option<Document> {
            match p.as_ref() {
                _ => None,
                #(#doc_ts),*
            }
        }
    }.into()
}

fn build_org_docs() -> HashMap<String, Document> {
    let mut map = HashMap::with_capacity(ORG_DIR.files().count());
    for file in ORG_DIR.files() {
        eprintln!("{file:?}");
        let doc = Document::from_org_id_file(file).expect(&format!("while parsing {:?}", file.path()));
        map.insert(file.path().to_str().expect("to_str").to_string(), doc);
    }
    map
}
