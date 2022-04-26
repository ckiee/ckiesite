use std::collections::HashMap;

use include_dir::{include_dir, Dir};

extern crate proc_macro;
use proc_macro::{TokenStream, Span};
use quote::quote;
use site_common::document::Document;
use syn::LitByteStr;

static ORG_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../data/org");

// TODO: parse `ts`, take single string literal in and pass to include dir maybe?
#[proc_macro]
pub fn org_docs(_ts: TokenStream) -> TokenStream {
    let doc_map = build_org_docs();
    let literal = LitByteStr::new(&bincode::serialize(&doc_map).unwrap(), proc_macro2::Span::call_site());

    quote! {
        ::bincode::deserialize(&#literal[..]).unwrap()
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
