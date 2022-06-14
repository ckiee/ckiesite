use anyhow::Result;
use std::{
    env,
    fs::File,
    io::{self, Read},
};

use orgish::{parse::parse_n_pass, treewalk::{ast_to_html_string, OutputTo::{self}}};

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

    let bufs = ast_to_html_string(&ast, OutputTo::Main)?;

    println!("{:#?}", bufs);

    Ok(())
}
