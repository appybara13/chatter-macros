/*!
The proc macro for parsing chatter into the structs
provided by [Chatter](https://github.com/appybara13/chatter).
*/

use std::collections::HashMap;

use parser::{parse, Choice};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_str;

mod parser;

const UNEXPECTED_ERROR: &'static str =
    "Unexpected error encountered. Please create an issue, including the input that caused the error, at https://github.com/appybara13/chatter-macros.";

fn get_contents(input: proc_macro::TokenStream) -> String {
    const ERROR: &'static str = "Input is expected to be a raw string literal.";

    input
        .to_string()
        .strip_prefix("r#\"")
        .expect(ERROR)
        .strip_suffix("\"#")
        .expect(ERROR)
        .to_string()
}

fn next_position(
    current_position: usize,
    goto: &Option<String>,
    branches: &HashMap<String, usize>,
) -> usize {
    match goto {
        Some(branch) => *branches.get(branch).expect(UNEXPECTED_ERROR),
        None => current_position + 1,
    }
}

fn next_tokens(
    current_position: usize,
    content_length: usize,
    goto: &Option<String>,
    branches: &HashMap<String, usize>,
) -> TokenStream {
    let next_position = next_position(current_position, goto, branches);

    match next_position < content_length {
        true => quote! {Some(#next_position)},
        false => quote! {None},
    }
}

fn raw_string_literal_tokens(string: &String) -> TokenStream {
    parse_str(&format!("r#\"{}\"#", string)).expect(UNEXPECTED_ERROR)
}

fn tags_tokens(tags: &Vec<String>) -> TokenStream {
    let mut inner = TokenStream::new();

    for tag in tags {
        let tag = raw_string_literal_tokens(tag);

        inner = match inner.is_empty() {
            true => quote! {#tag},
            false => quote! {#inner, #tag},
        }
    }

    quote! {&[#inner]}
}

fn line_tokens(
    current_position: usize,
    content_length: usize,
    branches: &HashMap<String, usize>,
    text: &String,
    tags: &Vec<String>,
    goto: &Option<String>,
) -> TokenStream {
    let next = next_tokens(current_position, content_length, goto, branches);

    let text = raw_string_literal_tokens(text);

    let tags = tags_tokens(tags);

    quote! {chatter::Line::new(#text, #tags, #next)}
}

fn choice_tokens(
    current_position: usize,
    content_length: usize,
    branches: &HashMap<String, usize>,
    choice: &Choice,
) -> TokenStream {
    let next = next_tokens(current_position, content_length, &choice.goto, branches);

    let text = raw_string_literal_tokens(&choice.text);

    let tags = tags_tokens(&choice.tags);

    quote! {chatter::Choice::new(#text, #tags, #next)}
}

fn choices_tokens(
    current_position: usize,
    content_length: usize,
    branches: &HashMap<String, usize>,
    choices: &Vec<Choice>,
) -> TokenStream {
    let mut inner = TokenStream::new();

    for choice in choices {
        let choice = choice_tokens(current_position, content_length, branches, choice);

        inner = match inner.is_empty() {
            true => quote! {#choice},
            false => quote! {#inner, #choice},
        }
    }

    quote! {chatter::Choice::new_group(vec![#inner])}
}

#[proc_macro]
pub fn chatter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (content, branches) = parse(get_contents(input).as_str());

    let mut inner = TokenStream::new();

    for (position, value) in content.iter().enumerate() {
        let value = match value {
            parser::Chat::Line { text, tags, goto } => {
                line_tokens(position, content.len(), &branches, text, tags, goto)
            }
            parser::Chat::Choices(choices) => {
                choices_tokens(position, content.len(), &branches, choices)
            }
        };

        inner = match inner.is_empty() {
            true => quote! {#value},
            false => quote! {#inner, #value},
        }
    }

    quote! {chatter::Chat::new(vec!{#inner})}.into()
}
