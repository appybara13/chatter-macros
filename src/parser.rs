use std::collections::HashMap;

use crate::UNEXPECTED_ERROR;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

pub(crate) struct Choice {
    pub text: String,
    pub tags: Vec<String>,
    pub goto: Option<String>,
}

pub(crate) enum Chat {
    Line {
        text: String,
        tags: Vec<String>,
        goto: Option<String>,
    },
    Choices(Vec<Choice>),
}

#[derive(Parser)]
#[grammar = "chatter.pest"]
struct ChatterParser;

pub(crate) fn parse(input: &str) -> (Vec<Chat>, HashMap<String, usize>) {
    let chatter = ChatterParser::parse(Rule::chatter, input).unwrap();

    let mut chat = Vec::new();
    let mut branches = HashMap::new();

    for pair in chatter {
        match pair.as_rule() {
            Rule::line => chat.push(parse_line(pair)),
            Rule::choices => chat.push(parse_choices(pair)),
            Rule::branch => {
                let branch = pair.into_inner().next().unwrap().as_str().to_string();
                branches.insert(branch, chat.len());
            }
            Rule::goto => {
                let branch = pair.into_inner().next().unwrap().as_str().to_string();

                let last_chat = chat.last_mut().unwrap();
                match last_chat {
                    Chat::Line {
                        text: _,
                        tags: _,
                        goto,
                    } => {
                        if goto.is_none() {
                            *goto = Some(branch.clone())
                        }
                    }
                    Chat::Choices(choices) => {
                        for choice in choices {
                            if choice.goto.is_none() {
                                choice.goto = Some(branch.clone())
                            }
                        }
                    }
                }
            }
            Rule::EOI => break,
            _ => {
                panic!("{}", UNEXPECTED_ERROR)
            }
        }
    }

    (chat, branches)
}

fn parse_choices(input: Pair<Rule>) -> Chat {
    let mut choices = Vec::new();

    for pair in input.into_inner() {
        choices.push(parse_choice(pair));
    }

    Chat::Choices(choices)
}

fn parse_choice(choice: Pair<Rule>) -> Choice {
    let mut text = None;
    let mut tags = Vec::new();
    let mut goto = None;

    for pair in choice.into_inner() {
        match pair.as_rule() {
            Rule::goto => goto = Some(pair.into_inner().next().unwrap().as_str().to_string()),
            Rule::tags => {
                for tag in pair.into_inner() {
                    tags.push(tag.as_str().to_string())
                }
            }
            Rule::text => text = Some(pair.as_str().to_string()),
            _ => panic!("{}", UNEXPECTED_ERROR),
        }
    }

    match text {
        Some(text) => Choice { text, tags, goto },
        None => panic!("{}", UNEXPECTED_ERROR),
    }
}

fn parse_line(line: Pair<Rule>) -> Chat {
    let mut text = None;
    let mut tags = Vec::new();
    let mut goto = None;

    for pair in line.into_inner() {
        match pair.as_rule() {
            Rule::goto => goto = Some(pair.into_inner().next().unwrap().as_str().to_string()),
            Rule::tags => {
                for tag in pair.into_inner() {
                    tags.push(tag.as_str().to_string())
                }
            }
            Rule::text => text = Some(pair.as_str().to_string()),
            _ => panic!("{}", UNEXPECTED_ERROR),
        }
    }

    match text {
        Some(text) => Chat::Line { text, tags, goto },
        None => panic!("{}", UNEXPECTED_ERROR),
    }
}
