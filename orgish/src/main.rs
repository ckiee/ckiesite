use anyhow::Result;
use html_minifier::HTMLMinifier;
use std::{
    env,
    fs::File,
    io::{self, Read},
};

use ckiesite::{parse::parse_n_pass, treewalk::ast_to_html_string, template::make_article_html};

fn main() -> Result<()> {
    let mut args = env::args().into_iter().peekable();
    let mut print_ast = false;
    let mut maybe_file: Option<String> = None;

    // index 0 is undefined
    args.next();

    while let Some(arg) = args.next() {
        match &arg[..] {
            "--print-ast" => {
                print_ast = true;
            }
            file if args.peek().is_none() => {
                maybe_file = Some(file.to_string());
            }
            arg => panic!("unknown arg: {}", arg),
        }
    }

    let buf = Box::leak(Box::new(String::new()));
    match maybe_file {
        Some(file) => File::open(file)?.read_to_string(buf)?,
        None => io::stdin().read_to_string(buf)?,
    };

    let ast = parse_n_pass(buf)?;
    if print_ast {
        eprintln!("{:#?}", ast);
    }

    let html = ast_to_html_string(&ast)?;
    let templated = make_article_html(&html);
    let minified = {
        let mut mf = HTMLMinifier::new();
        mf.digest(&templated)?;
        String::from_utf8(mf.get_html().to_vec())?
    };

    println!("{}", minified);

    Ok(())
}
