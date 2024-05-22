use std::mem;

use manyhow::{bail, ensure, Result};
use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, ToTokens};
use syn::ext::IdentExt;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream, Parser, Peek};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren};
use syn::{bracketed, parenthesized, parse2, BinOp, Expr, LitStr, Pat, Path, Token};
use syn_derive::{Parse, ToTokens};

use super::html::ensure_tag_name;
use crate::*;

pub fn html(input: TokenStream) -> Result<proc_macro2::TokenStream, manyhow::Error> {
    let nodes = expand_nodes(Punctuated::<Node, Token![,]>::parse_terminated.parse2(input)?);

    Ok(quote! {
        #use ::htmx::{ToHtml, Html, IntoHtmlElements};
        {
            use ::htmx::native::*;
            let mut __html = Html::new();
            #(#nodes)*
            __html
        }
    })
}

fn expand_nodes(nodes: impl IntoIterator<Item = Node>) -> impl Iterator<Item = TokenStream> {
    nodes.into_iter().map(move |n| n.expand())
}

fn peek_alone(p: impl Peek, input: ParseStream) -> bool {
    input.peek(p)
        && (input.peek2(Token![,]) || {
            input.parse::<TokenTree>().unwrap();
            input.is_empty()
        })
}

#[derive(Debug, Parse)]
enum Node {
    #[parse(peek_func = |input| peek_alone(LitStr, input))]
    String(LitStr),
    #[parse(peek_func = |input| peek_alone(Brace, input))]
    Block(Block),
    #[parse(peek = Token![if])]
    If(If),
    #[parse(peek = Token![for])]
    For(For),
    #[parse(peek = Token![while])]
    While(While),
    // TODO controlflow
    Element(Element),
}

impl Node {
    fn expand(self) -> TokenStream {
        match self {
            Node::String(lit) => {
                quote!(::htmx::ToHtml::to_html(&#lit, &mut __html);)
            }
            Node::Block(block) => {
                quote!(::htmx::ToHtml::to_html(&#block, &mut __html);)
            },
            Node::Element(element) => element.expand(),
            Node::If(node) => node.expand(),
            Node::For(node) => node.expand(),
            Node::While(node) => node.expand(),
        }
    }
}

#[derive(Debug, Parse, ToTokens)]
struct Block {
    #[syn(braced)]
    brace: Brace,
    #[syn(in = brace)]
    content: TokenStream,
}

#[derive(Debug)]
struct If {
    if_token: Token![if],
    condition: Expr,
    #[allow(unused)]
    bracket: Option<Bracket>,
    then_branch: Punctuated<Node, Token![,]>,
    else_branch: ElseBranch,
}

impl If {
    fn expand(self) -> TokenStream {
        let If {
            if_token,
            condition,
            then_branch,
            else_branch,
            ..
        } = self;
        let body = expand_nodes(then_branch);
        let else_branch = else_branch.expand();
        quote! {
            #if_token #condition {
                #(#body)*
            } #else_branch
        }
    }
}

fn expr_before_bracket(input: ParseStream) -> syn::Result<Expr> {
    fn take_tt(input: ParseStream, output: &mut TokenStream) {
        output.extend(input.parse::<TokenTree>())
    }

    let expr = &mut TokenStream::new();
    'outer: while !input.is_empty() {
        if input.peek(Token![<]) {
            let mut turbo_fish: usize = 1;
            take_tt(input, expr);
            while turbo_fish > 0 {
                if input.peek(Token![>]) {
                    turbo_fish -= 1;
                    take_tt(input, expr);
                } else if input.peek(Token![<]) {
                    turbo_fish += 1;
                    take_tt(input, expr);
                } else if input.is_empty() {
                    return Err(input.error("expected `>` to close generic"));
                } else {
                    take_tt(input, expr);
                }
            }
        } else if let Ok(closure) = input.parse::<Token![|]>() {
            closure.to_tokens(expr);
            while !input.peek(Token![|]) {
                if input.is_empty() {
                    return Err(input.error("expected `|` to close closure arguments"));
                } else {
                    take_tt(input, expr)
                }
            }
        }
        while !input.is_empty() {
            if let Ok(path_sep) = input.parse::<Token![::]>() {
                path_sep.to_tokens(expr);
                continue 'outer;
            } else if let Ok(op) = input.parse::<BinOp>() {
                op.to_tokens(expr);
                continue 'outer;
            } else if let Ok(assign) = input.parse::<Token![=]>() {
                assign.to_tokens(expr);
                continue 'outer;
            } else if let Ok(let_) = input.parse::<Token![let]>() {
                let_.to_tokens(expr);
                Pat::parse_multi_with_leading_vert(input)?.to_tokens(expr);
            } else if input.peek(Bracket)
                && (input.peek2(Token![else]) || input.peek2(Token![,]) || peek2_eof(input))
            {
                return parse2(mem::take(expr));
            } else {
                take_tt(input, expr);
            }
        }
    }
    Err(input.error("expected `[` or `{`"))
}

fn peek2_eof(input: ParseStream) -> bool {
    let mut ts = input.fork().parse::<TokenStream>().unwrap().into_iter();
    ts.next();
    ts.next().is_none()
}

fn expr_for_cf(
    input: ParseStream,
) -> syn::Result<(Expr, Option<Bracket>, Punctuated<Node, Token![,]>)> {
    let fork = &input.fork();
    let condition = expr_before_bracket(fork);
    let error = match condition {
        Ok(condition) => {
            input.advance_to(fork);
            let content;
            return Ok((
                condition,
                Some(bracketed!(content in input)),
                Punctuated::parse_terminated(&content)?,
            ));
        }
        result => result.err(),
    };

    {
        let fork = &input.fork();
        if let Ok(condition) = Expr::parse_without_eager_brace(fork) {
            if let Ok(block) = fork.parse() {
                input.advance_to(fork);
                return Ok((condition, None, Punctuated::from_iter([Node::Block(block)])));
            }
        }
    }

    match error {
        Some(error) => bail!(error),
        _ => {
            input.advance_to(fork);
            bail!(input.error("expected `[` or `{`"));
        }
    }
}

impl Parse for If {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let if_token = input.parse()?;
        let (condition, bracket, then_branch) = expr_for_cf(input)?;
        Ok(Self {
            if_token,
            condition,
            bracket,
            then_branch,
            else_branch: input.parse()?,
        })
    }
}

#[derive(Debug)]
enum ElseBranch {
    None,
    Else {
        else_token: Token![else],
        #[allow(unused)]
        bracket: Option<Bracket>,
        body: Punctuated<Node, Token![,]>,
    },
    ElseIf {
        else_token: Token![else],
        body: Box<If>,
    },
}

impl ElseBranch {
    fn expand(self) -> TokenStream {
        match self {
            ElseBranch::None => quote!(),
            ElseBranch::Else {
                else_token, body, ..
            } => {
                let body = expand_nodes(body);
                quote!( #else_token {#(#body)*} )
            }
            ElseBranch::ElseIf { else_token, body } => {
                let body = body.expand();
                quote!(#else_token #body)
            }
        }
    }
}

impl Parse for ElseBranch {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if let Ok(else_token) = input.parse() {
            if input.peek(Token![if]) {
                ElseBranch::ElseIf {
                    else_token,
                    body: Box::new(input.parse()?),
                }
            } else if input.peek(Bracket) {
                let body;
                ElseBranch::Else {
                    else_token,
                    bracket: Some(bracketed!(body in input)),
                    body: Punctuated::parse_terminated(&body)?,
                }
            } else if let Ok(block) = input.parse() {
                ElseBranch::Else {
                    else_token,
                    bracket: None,
                    body: Punctuated::from_iter([Node::Block(block)]),
                }
            } else {
                bail!(input.error("expected `[` or `{`"))
            }
        } else {
            ElseBranch::None
        })
    }
}

#[derive(Debug)]
struct For {
    for_token: Token![for],
    pat: syn::Pat,
    in_token: Token![in],
    expr: Expr,
    #[allow(unused)]
    bracket: Option<Bracket>,
    body: Punctuated<Node, Token![,]>,
}

impl For {
    fn expand(self) -> TokenStream {
        let Self {
            for_token,
            pat,
            in_token,
            expr,
            body,
            ..
        } = self;
        let body = expand_nodes(body);
        quote! {
            #for_token #pat #in_token #expr {
                #(#body)*
            }
        }
    }
}

impl Parse for For {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let for_token = input.parse()?;
        let pat = Pat::parse_single(input)?;
        let in_token = input.parse()?;

        let (expr, bracket, body) = expr_for_cf(input)?;
        Ok(Self {
            for_token,
            pat,
            in_token,
            expr,
            bracket,
            body,
        })
    }
}
#[derive(Debug)]
struct While {
    while_token: Token![while],
    expr: Expr,
    #[allow(unused)]
    bracket: Option<Bracket>,
    body: Punctuated<Node, Token![,]>,
}

impl While {
    fn expand(self) -> TokenStream {
        let Self {
            while_token,
            expr,
            body,
            ..
        } = self;
        let body = expand_nodes(body);
        quote! {
            #while_token #expr {
                #(#body)*
            }
        }
    }
}

impl Parse for While {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let while_token = input.parse()?;

        let (expr, bracket, body) = expr_for_cf(input)?;

        Ok(Self {
            while_token,
            expr,
            bracket,
            body,
        })
    }
}

#[derive(Debug)]
struct Element {
    path: ElementName,
    attrs: Option<Attrs>,
    #[allow(unused)]
    bracket: Option<Bracket>,
    children: Punctuated<Node, Token![,]>,
}
impl Element {
    fn expand(self) -> TokenStream {
        let mut attrs = self.attrs.unwrap_or_default();
        let name = match self.path {
            ElementName::String(name) => {
                quote!(::htmx::CustomElement::new_unchecked(&mut __html, #name);)
            }
            ElementName::Block(block) => quote!(::htmx::CustomElement::new(&mut __html, #block);),
            ElementName::Classes(classes) => {
                attrs.attrs.push(Attr::Classes(classes));
                quote!(div::new(&mut __html))
            }
            ElementName::Path(path) => quote!(#path::new(&mut __html);),
        };

        let mut children = expand_nodes(
            attrs
                .content
                .into_iter()
                .flat_map(|(_, c)| c)
                .chain(self.children),
        )
        .peekable();

        let attrs = attrs.attrs.into_iter().map(Attr::expand);

        let body = children.peek().is_some().then(|| quote!(__html.body(|mut __html| {#(#children)*})));

        quote!({
            let mut __html = #name
            #(__html #attrs;)* 
            #body;
        })
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;
        let attrs = input.peek(Paren).then(|| input.parse()).transpose()?;

        let (bracket, children) = if input.peek(Bracket) {
            let nodes;
            (
                Some(bracketed!(nodes in input)),
                Punctuated::parse_terminated(&nodes)?,
            )
        } else {
            Default::default()
        };

        Ok(Self {
            path,
            attrs,
            bracket,
            children,
        })
    }
}

#[derive(Debug)]
enum ElementName {
    String(LitStr),
    Block(Block),
    Classes(Classes),
    Path(Path),
}

impl Parse for ElementName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(lit) = input.parse::<LitStr>() {
            ensure_tag_name(lit.value(), &lit)?;
            Ok(Self::String(lit))
        } else if let Ok(block) = input.parse::<Block>() {
            parse2::<LitStr>(block.content.clone())
                .ok()
                .map_or(Ok(Self::Block(block)), |name| {
                    ensure_tag_name(name.value(), &name)
                        .map(|_| Self::String(name))
                        .map_err(Into::into)
                })
        } else if input.peek(Token![.]) {
            input.parse().map(Self::Classes)
        } else {
            input.parse().map(Self::Path)
        }
    }
}

#[derive(Debug, Parse)]
struct Classes {
    #[allow(unused)]
    leading: Token![.],
    #[parse(Punctuated::parse_separated_nonempty)]
    classes: Punctuated<Name, Token![.]>,
}

#[derive(Debug, Parse)]
enum Name {
    #[parse(peek = LitStr)]
    String(LitStr),
    #[parse(peek = Brace)]
    Block(Block),
    Ident(#[parse(Ident::parse_any)] Ident),
}

impl Name {
    fn lit_str(&self) -> Option<String> {
        match self {
            Name::String(lit) => Some(lit.value()),
            Name::Block(Block { content, .. }) => parse2::<LitStr>(content.clone())
                .ok()
                .as_ref()
                .map(LitStr::value),
            Name::Ident(_) => None,
        }
    }

    fn attribute(input: ParseStream) -> syn::Result<Self> {
        input.parse().and_then(|value: Self| {
            if let Some(value) = value.lit_str() {
                ensure!(
                    value.chars().any(|c| c.is_whitespace()
                        || c.is_control()
                        || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')),
                    value,
                    "invalid key `{value}`, \
                    https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0"
                );
            }
            Ok(value)
        })
    }
}

impl ToTokens for Name {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Name::String(lit) => lit.to_tokens(tokens),
            Name::Block(block) => block.to_tokens(tokens),
            Name::Ident(ident) => LitStr::new(&ident.to_string(), ident.span()).to_tokens(tokens),
        }
    }
}

#[derive(Debug, Default)]
struct Attrs {
    #[allow(unused)]
    paren: Paren,
    attrs: Punctuated<Attr, Token![,]>,
    content: Option<(Token![;], Punctuated<Node, Token![,]>)>,
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            paren: parenthesized!(content in input),
            attrs: {
                if !(content.peek(Token![;]) || content.is_empty()) {
                    let mut attrs = Punctuated::parse_separated_nonempty(&content)?;
                    if content.peek(Token![,]) {
                        attrs.push_punct(content.parse()?);
                    }
                    attrs
                } else {
                    Default::default()
                }
            },
            content: {
                if content.peek(Token![;]) {
                    Some((content.parse()?, Punctuated::parse_terminated(&content)?))
                } else {
                    None
                }
            },
        })
    }
}

#[derive(Debug)]
#[allow(unused)]
enum Attr {
    Id(Token![#], Name),
    Classes(Classes),
    // TODO Value(Expr),
    // TODO Flag(Name),
    // TODO StructShorthand(Name),
    KeyValue(Name, Token![:], Expr),
    Trailing(Token![..], Expr),
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![#]) {
            Self::Id(input.parse()?, input.parse()?)
        } else if input.peek(Token![..]) {
            // This means, that attribute names need to use parentheses to be lhs open
            // ranges
            Self::Trailing(input.parse()?, input.parse()?)
        } else if input.peek(Token![.]) {
            Self::Classes(input.parse()?)
        } else {
            // let expr = input.parse()?;
            // if input.peek(Token![:]) {
            Self::KeyValue(input.call(Name::attribute)?, input.parse()?, input.parse()?)
            // } else {
            // Self::Value(expr)
            // }
        })
    }
}

fn is_keyword(ident: &Ident) -> bool {
    match ident.to_string().as_str() {
        // Based on https://doc.rust-lang.org/1.65.0/reference/keywords.html
        "abstract" | "as" | "async" | "await" | "become" | "box" | "break" | "const"
        | "continue" | "crate" | "do" | "dyn" | "else" | "enum" | "extern" | "false" | "final"
        | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "macro" | "match" | "mod"
        | "move" | "mut" | "override" | "priv" | "pub" | "ref" | "return" | "Self" | "self"
        | "static" | "struct" | "super" | "trait" | "true" | "try" | "type" | "typeof"
        | "unsafe" | "unsized" | "use" | "virtual" | "where" | "while" | "yield" => true,
        _ => false,
    }
}

impl Attr {
    fn expand(self) -> TokenStream {
        match self {
            Attr::Id(_, id) => quote!(.id(#id)),
            Attr::Classes(classes) => {
                let classes = classes.classes.into_iter();
                quote!(#(.class(#classes))*)
            }
            Attr::KeyValue(name, _, value) => match name {
                Name::Ident(ref name) if is_keyword(name) => {
                    let name = format_ident!("{name}_");
                    quote!(.#name(#value))
                }
                Name::Ident(name) => quote!(.#name(#value)),
                name => {
                    if name.lit_str().is_some() {
                        quote!(.custom_attr_unchecked(#name, #value))
                    } else {
                        quote!(.custom_attr(#name, #value))
                    }
                }
            },
            Attr::Trailing(..) => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use forr::forr;
    use proc_macro_utils::assert_tokens;

    use super::*;

    #[test]
    fn expr_before_bracket() {
        fn helper(input: ParseStream) -> syn::Result<(Expr, TokenStream)> {
            Ok((super::expr_before_bracket(input)?, input.parse()?))
        }

        forr! { $expr:inner in [(1 + 1), (<[u8]>::hello::<[u8], Hello<[u8],> >)] $*
        forr! { $after:inner in [([],), ([] else), ([])] $*
            let tokens = quote!($expr $after);
            let (expr, rest) = helper.parse2(tokens).unwrap();
            assert_tokens!(expr.into_token_stream(), {$expr});
            assert_tokens!(rest.into_token_stream(), {$after});
        }}
    }
}
