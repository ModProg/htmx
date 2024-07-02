use std::borrow::Borrow;
use std::iter;

use manyhow::Result;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use rstml::node::CustomNode;
use rstml::recoverable::{ParseRecoverable, RecoverableContext};
use syn::parse::{ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Paren};
use syn::{Expr, ExprPath, Token};
use syn_derive::ToTokens;

use super::html::{expand_node, expand_nodes};
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

macro_rules! parenthesized {($name:ident in $parser:expr, $input:expr) => {{
    let paren = $parser.save_diagnostics((|| {
        let content;
        let paren = syn::parenthesized!(content in $input);
        Ok((paren, content))
        })())?;
    $name = paren.1;
    paren.0
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
    While(While),
    FunctionCall(FunctionCall),
}

fn map_vec(value: Vec<Node>) -> Result<Vec<super::Node>> {
    value.into_iter().map(super::Node::try_from).collect()
}

impl TryFrom<Special> for super::Node {
    type Error = manyhow::Error;

    fn try_from(value: Special) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            Special::If(if_) => super::Node::If(if_.try_into()?),
            Special::For(For {
                pat, expr, body, ..
            }) => super::Node::For(super::For {
                pat: pat.into_token_stream(),
                expr: expr.into_token_stream(),
                body: map_vec(body)?,
            }),
            Special::While(While { expr, body, .. }) => super::Node::While(super::While {
                expr: expr.into_token_stream(),
                body: map_vec(body)?,
            }),
            Special::FunctionCall(FunctionCall { function, args, .. }) => {
                super::Node::FunctionCall(super::FunctionCall {
                    function: function.into_token_stream(),
                    args: args.into_iter().map(ToTokens::into_token_stream).collect(),
                })
            }
        })
    }
}

impl Special {
    pub(crate) fn expand_node(self) -> Result {
        match self {
            Special::If(if_) => if_.expand_node(),
            Special::For(for_) => for_.expand_node(),
            Special::While(while_) => while_.expand_node(),
            Special::FunctionCall(function_call) => function_call.expand_node(),
        }
    }
}

impl CustomNode for Special {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        ToTokens::to_tokens(self, tokens)
    }

    fn peek_element(input: ParseStream) -> bool {
        let fork = input.fork();
        input.peek(Token![if])
            || input.peek(Token![for])
            || input.peek(Token![while])
            || fork.parse::<Token![<]>().is_ok()
                && fork.parse::<ExprPath>().is_ok()
                && fork.peek(Paren)
    }

    fn parse_element(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        match () {
            () if input.peek(Token![if]) => parser.parse_recoverable(input).map(Self::If),
            () if input.peek(Token![for]) => parser.parse_recoverable(input).map(Self::For),
            () if input.peek(Token![while]) => parser.parse_recoverable(input).map(Self::While),
            () if input.peek(Token![<]) => parser.parse_recoverable(input).map(Self::FunctionCall),
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

impl TryFrom<If> for super::If {
    type Error = manyhow::Error;

    fn try_from(
        If {
            condition,
            then_branch,
            else_branch,
            ..
        }: If,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(super::If {
            condition: condition.into_token_stream(),
            then_branch: map_vec(then_branch)?,
            else_branch: match else_branch {
                ElseBranch::None => super::ElseBranch::None,
                ElseBranch::Else { body, .. } => super::ElseBranch::Else(map_vec(body)?),
                ElseBranch::ElseIf { body, .. } => {
                    super::ElseBranch::ElseIf(Box::new((*body).try_into()?))
                }
            },
        })
    }
}

impl If {
    fn expand_node(self) -> Result {
        let If {
            if_token,
            condition,
            then_branch,
            else_branch,
            ..
        } = self;
        let body = then_branch
            .into_iter()
            .map(expand_node)
            .collect::<Result>()?;
        let else_branch = else_branch.expand_node()?;
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
    fn expand_node(self) -> Result {
        Ok(match self {
            ElseBranch::None => quote!(),
            ElseBranch::Else {
                else_token, body, ..
            } => {
                let body = expand_nodes(body)?;
                quote!( #else_token {#body} )
            }
            ElseBranch::ElseIf { else_token, body } => {
                let body = body.expand_node()?;
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
    fn expand_node(self) -> Result {
        let Self {
            for_token,
            pat,
            in_token,
            expr,
            body,
            ..
        } = self;
        let body = expand_nodes(body)?;
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

#[derive(Debug, ToTokens)]
pub struct While {
    pub while_token: Token![while],
    pub expr: Expr,
    #[syn(braced)]
    pub brace: Brace,
    #[syn(in = brace)]
    #[to_tokens(TokenStreamExt::append_all)]
    pub body: Vec<Node>,
}

impl While {
    fn expand_node(self) -> Result {
        let Self {
            while_token,
            expr,
            body,
            ..
        } = self;
        let body = expand_nodes(body)?;
        Ok(quote!(#while_token #expr { #body }))
    }
}

impl ParseRecoverable for While {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        let body;
        Some(Self {
            while_token: parser.parse_simple(input)?,
            expr: parser.save_diagnostics(Expr::parse_without_eager_brace(input))?,
            brace: braced!(body in parser, input),
            body: parse_nodes(parser, body),
        })
    }
}

#[derive(Debug, ToTokens)]
pub struct FunctionCall {
    pub open_token: Token![<],
    pub function: ExprPath,
    #[syn(parenthesized)]
    pub paren: Paren,
    #[syn(in = paren)]
    #[to_tokens(TokenStreamExt::append_all)]
    pub args: Punctuated<Expr, Token![,]>,
    pub slash: Token![/],
    pub gt_token: Token![>],
}

impl FunctionCall {
    fn expand_node(self) -> Result {
        let Self { function, args, .. } = self;
        let args = args.into_iter();
        Ok(quote!(::htmx::ToHtml::to_html(&#function(#(Into::into(#args),)*), &mut __html);))
    }
}

impl ParseRecoverable for FunctionCall {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        let args;
        Some(Self {
            open_token: parser.parse_simple(input)?,
            function: parser.parse_simple(input)?,
            paren: parenthesized!(args in parser, input),
            args: parser.save_diagnostics(Punctuated::parse_terminated(&args))?,
            slash: parser.parse_simple(input)?,
            gt_token: parser.parse_simple(input)?,
        })
    }
}
