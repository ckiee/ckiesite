use std::collections::HashMap;

use combine::{
    attempt, between, many1,
    parser::choice::choice,
    parser::{
        char::{alpha_num, spaces, string},
        repeat::many,
        token::token,
    },
    satisfy,
    stream::easy,
    ParseError, Parser, Stream,
};

#[derive(PartialEq, Debug, Default)]
pub struct Org {
    directives: HashMap<String, String>,
}

fn directive<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        between(string("#+"), token(':'), many(satisfy(|c| c != ':'))),
        spaces(),
        many1(alpha_num()),
    )
        .map(|(key, _, value)| (key, value))
}

pub fn org<Input>() -> impl Parser<Input, Output = Org>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many(directive())
    ).map(|directives| Org { directives })
}
