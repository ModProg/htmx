#![allow(unused)]
use manyhow::bail;
use proc_macro2::Group;
use proc_macro_utils::TokenStream2Ext;
use quote::ToTokens;

use crate::*;

// TODO this is not good enough, we need to fully parse css due to idents
// containing `-` that should not have spaces and expressions that need them.
pub fn css(input: TokenStream) -> Result<TokenStream> {
    let output = css_transform(input)?;
    let string = output.to_string();
    Ok(quote!(htmx::Css(string.into())))
}

pub fn css_transform(input: TokenStream) -> Result<TokenStream> {
    let mut output = TokenStream::new();
    let mut input = input.parser();
    while !input.is_empty() {
        if let Some(use_) = input.next_keyword("use") {
            let Some(path) = input.next_string() else {
                if let Some(unexp) = input.next() {
                    bail!(unexp, "expected string path");
                } else {
                    bail!(use_, "expected to be followed by string path");
                }
            };
            if input.next_tt_semi().is_none() {
                if let Some(unexp) = input.next() {
                    bail!(unexp, ";");
                } else {
                    bail!(use_, "expected to be followed by string path");
                }
            }
        } else if let Some(group) = input.next_group() {
            output.push(Group::new(group.delimiter(), css(group.stream())?).into())
        } else {
            output.extend(input.next())
        }
    }
    Ok(output)
}

// Maybe I should reconsider... feels like I'm reimplementing all of scss :D
// @rules
// @charset "<charser>";
// @color-profile <ident> {<parameters>}
// @container <container-condition> {<stylesheet>}
#[derive(derive_more::Display)]
enum AtRule {
    Charset(String)
}
