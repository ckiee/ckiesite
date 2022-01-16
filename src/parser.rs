use std::collections::HashMap;

use combine::{
    attempt, between, choice, many1,
    parser::{char::newline, sequence::skip},
    parser::{
        char::{alpha_num, space, spaces, string},
        repeat::many,
        token::token,
    },
    satisfy, skip_many, skip_many1,
    stream::easy,
    ParseError, Parser, Stream,
};

#[derive(PartialEq, Debug)]
pub enum AstNode {
    Directive(String, String),
    Text(String),
    Heading { level: u16, title: Box<AstNode> },
}

fn whitespace<Input>() -> impl Parser<Input, Output = char>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    satisfy(|c: char| c.is_whitespace() && c != '\n')
}


fn whitespaces<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many(whitespace())
}


fn linespace<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let comment = (token('#'), skip_many(satisfy(|c| c != '\n'))).map(|_| ());
    let skipline = newline().map(|_| ());
    many((whitespaces(), skipline.or(comment)).map(|(_, _)| ()))
}

fn directive<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        between(string("#+"), token(':'), many(satisfy(|c| c != ':'))),
        whitespaces(),
        many1(satisfy(|c: char| !c.is_control())),
    )
        .map(|(key, _, value)| AstNode::Directive(key, value))
        .message("while parsing directive")
}

fn text<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(satisfy(|c: char| !c.is_control()))
        .map(|x| AstNode::Text(x))
        .message("while parsing text")
}

fn heading<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        whitespaces(),
        many1::<Vec<_>, _, _>(token('*')).map(|x: Vec<_>| x.len()),
        whitespaces(),
        text().map(|x| Box::new(x)),
    )
        .map(|(_, level, _, title)| AstNode::Heading {
            level: level
                .try_into()
                .expect("the header level to be smaller than the maximum value of usize"),
            title,
        })
        .message("while parsing heading")
}

pub fn ast_node<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice!(heading(), text())
}


pub fn org_file<Input>() -> impl Parser<Input, Output = Vec<AstNode>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many(ast_node().skip(linespace()))
}
