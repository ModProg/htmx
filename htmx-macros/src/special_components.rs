use std::borrow::Borrow;
use std::iter;

use manyhow::Result;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use rstml::node::CustomNode;
use rstml::recoverable::{ParseRecoverable, RecoverableContext};
use syn::parse::{ParseBuffer, ParseStream};
use syn::token::Brace;
use syn::{Expr, Token};
use syn_derive::ToTokens;

use crate::htmx::{expand_node, expand_nodes};
use crate::*;
pub type Node = rstml::node::Node<Special>;

macro_rules! braced {($name:ident in $parser:expr, $input:expr) => {{
    let brace = $parser.save_diagnostics((|| {
        let content;
        let brace = syn::braced!(content in $input);
        Ok((brace, content))
        })())?;
    $name = brace.1;
    brace.0
}};}

fn parse_nodes<'a>(
    parser: &mut RecoverableContext,
    input: impl Borrow<ParseBuffer<'a>>,
) -> Vec<Node> {
    iter::from_fn(|| parser.parse_recoverable(input.borrow())).collect()
}

///// Unsure how to end the `if`, e.g., in the case of and `else` / `else if`
// <if a> ... </if>
// <if let Some(a) = a> ... </if>
// <if a> ... <else> ... </if>
// <if a> ... <else if b> ... </if>

// <for a in b> ... </for>
// <while a> ... </while>
// <while let Some(a) = b.next()> ... </while>

///// Unsure what syntax to use for match arms
// <match a>
//     <Some(b)> ... </>
//     <None> ... </>
//     <_> ... </>
///// OR
//     <case Some(b)> ... </case>
//     <case None> ... </case>
//     <default> ... </default>
// </match>

// TODO consider using non tag control flow

#[derive(Debug, ToTokens)]
pub enum Special {
    If(If),
    For(For),
}
impl Special {
    pub(crate) fn expand_node(self, htmx: &TokenStream, child: bool) -> Result {
        match self {
            Special::If(if_) => if_.expand_node(htmx, child),
            Special::For(for_) => for_.expand_node(htmx, child),
        }
    }
}

impl CustomNode for Special {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        ToTokens::to_tokens(self, tokens)
    }

    fn peek_element(input: ParseStream) -> bool {
        input.peek(Token![if]) || input.peek(Token![for])
    }

    fn parse_element(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        match () {
            () if input.peek(Token![if]) => parser.parse_recoverable(input).map(Self::If),
            () if input.peek(Token![for]) => parser.parse_recoverable(input).map(Self::For),
            _ => unreachable!("`peek_element` should only peek valid keywords"),
        }
    }
}

#[derive(Debug, ToTokens)]
pub struct If {
    pub if_token: Token![if],
    pub condition: Expr,
    #[syn(braced)]
    pub brace: Brace,
    #[syn(in = brace)]
    #[to_tokens(TokenStreamExt::append_all)]
    pub then_branch: Vec<Node>,
    pub else_branch: ElseBranch,
}

impl If {
    fn expand_node(self, htmx: &TokenStream, child: bool) -> Result {
        let If {
            if_token,
            condition,
            then_branch,
            else_branch,
            ..
        } = self;
        let body = then_branch
            .into_iter()
            .map(|n| expand_node(n, htmx, child))
            .collect::<Result>()?;
        let else_branch = else_branch.expand_node(htmx, child)?;
        Ok(quote! {
            #if_token #condition {
                #body
            } #else_branch
        })
    }
}

#[derive(Debug, ToTokens)]
pub enum ElseBranch {
    None,
    Else {
        else_token: Token![else],
        #[syn(braced)]
        brace: Brace,
        #[syn(in = brace)]
        #[to_tokens(TokenStreamExt::append_all)]
        body: Vec<Node>,
    },
    ElseIf {
        else_token: Token![else],
        body: Box<If>,
    },
}
impl ElseBranch {
    fn expand_node(self, htmx: &TokenStream, child: bool) -> Result {
        Ok(match self {
            ElseBranch::None => quote!(),
            ElseBranch::Else {
                else_token, body, ..
            } => {
                let body = expand_nodes(body, htmx, child)?;
                quote!( #else_token {#body} )
            }
            ElseBranch::ElseIf { else_token, body } => {
                let body = body.expand_node(htmx, child)?;
                quote!(#else_token #body)
            }
        })
    }
}

impl ParseRecoverable for If {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        let body;
        Some(Self {
            if_token: parser.parse_simple(input)?,
            condition: parser.save_diagnostics(Expr::parse_without_eager_brace(input))?,
            brace: braced!(body in parser, input),
            then_branch: parse_nodes(parser, body),
            else_branch: if let Ok(else_token) = input.parse() {
                if input.peek(Token![if]) {
                    ElseBranch::ElseIf {
                        else_token,
                        body: Box::new(parser.parse_recoverable(input)?),
                    }
                } else {
                    let body;
                    ElseBranch::Else {
                        else_token,
                        brace: braced!(body in parser, input),
                        body: parse_nodes(parser, body),
                    }
                }
            } else {
                ElseBranch::None
            },
        })
    }
}

#[derive(Debug, ToTokens)]
pub struct For {
    pub for_token: Token![for],
    pub pat: syn::Pat,
    pub in_token: Token![in],
    pub expr: Expr,
    #[syn(braced)]
    pub brace: Brace,
    #[syn(in = brace)]
    #[to_tokens(TokenStreamExt::append_all)]
    pub body: Vec<Node>,
}
impl For {
    fn expand_node(self, htmx: &TokenStream, child: bool) -> Result {
        let Self {
            for_token,
            pat,
            in_token,
            expr,
            body,
            ..
        } = self;
        let body = expand_nodes(body, htmx, child)?;
        Ok(quote!(#for_token #pat #in_token #expr { #body }))
    }
}

impl ParseRecoverable for For {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        let body;
        Some(Self {
            for_token: parser.parse_simple(input)?,
            pat: parser.save_diagnostics(syn::Pat::parse_multi_with_leading_vert(input))?,
            in_token: parser.parse_simple(input)?,
            expr: parser.save_diagnostics(Expr::parse_without_eager_brace(input))?,
            brace: braced!(body in parser, input),
            body: parse_nodes(parser, body),
        })
    }
}
