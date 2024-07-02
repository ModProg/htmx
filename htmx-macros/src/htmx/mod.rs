#![allow(unused)]
pub mod html;
mod special_components;

pub mod rusty;

use html_escape::{encode_safe, encode_script};
use manyhow::ensure;
use proc_macro2::{Literal, Span};
use syn::spanned::Spanned;
use syn::LitStr;

use super::*;

// pub fn html(input: TokenStream) -> Result {
//     if input.is_empty() {
//         return Ok(quote!(::htmx::Html::new()));
//     }

//     let mut fork = input.clone().into_iter();

//     let first = fork.next();
//     let second = fork.next();

//     // TODO figure out actual differentiator
//     // probably would be, starts with `<` or starts with `{}` or `""` not
// followed     // by `,`

//     if matches!(input.peek(), Some(TokenTree::Punct(punct)) if
// punct.as_char() == '<') {         html::html(input.collect())
//     } else {
//         rusty::html(input.collect())
//     }
// }

fn try_into_iter<T>(
    input: impl IntoIterator<Item = impl TryInto<T, Error = manyhow::Error>>,
) -> Result<Vec<T>> {
    input.into_iter().map(TryInto::try_into).collect()
}

fn expand_nodes(
    nodes: impl IntoIterator<Item = impl TryInto<Node, Error = manyhow::Error>>,
) -> Result {
    let nodes = nodes
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<Node>>>()?;
    Ok(quote! {
        ::htmx::Fragment(move |mut __html: &mut ::htmx::Html| {
            #[allow(unused_braces)]
            {
                use ::htmx::native::*;
                use ::htmx::IntoHtml as _;
                #(#nodes)*
            };
        })
    })
}

enum Node {
    String(LitStr),
    Block(TokenStream),
    If(If),
    For(For),
    While(While),
    FunctionCall(FunctionCall),
    Element(Element),
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Node::String(lit) => {
                let value = lit.value();
                let value = encode_safe(&value);
                let mut value = Literal::string(&value);
                value.set_span(lit.span());
                quote!(::htmx::IntoHtml::into_html(::htmx::RawSrc::new(#value), &mut __html);).to_tokens(tokens)
            }
            Node::Block(block) => {
                quote!(::htmx::IntoHtml::into_html({#[allow(unused_braces)] {#block}}, &mut __html);).to_tokens(tokens)
            }
            Node::If(if_) => if_.to_tokens(tokens),
            Node::For(for_) => for_.to_tokens(tokens),
            Node::While(while_) => while_.to_tokens(tokens),
            Node::FunctionCall(call) => call.to_tokens(tokens),
            Node::Element(element) => element.to_tokens(tokens)
        }
    }
}

struct If {
    condition: TokenStream,
    then_branch: Vec<Node>,
    else_branch: ElseBranch,
}

impl ToTokens for If {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            condition,
            then_branch,
            else_branch,
        } = self;
        quote! {
            if #condition {
                #(#then_branch)*
            } #else_branch
        }
        .to_tokens(tokens)
    }
}

enum ElseBranch {
    None,
    Else(Vec<Node>),
    ElseIf(Box<If>),
}

impl ToTokens for ElseBranch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ElseBranch::None => {}
            ElseBranch::Else(nodes) => quote!(else {#(#nodes)*}).to_tokens(tokens),
            ElseBranch::ElseIf(if_) => quote!(else #if_).to_tokens(tokens),
        }
    }
}

struct For {
    pat: TokenStream,
    expr: TokenStream,
    body: Vec<Node>,
}

impl ToTokens for For {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { pat, expr, body } = self;
        quote! {
            for #pat in #expr {
                #(#body)*
            }
        }
        .to_tokens(tokens)
    }
}

struct While {
    expr: TokenStream,
    body: Vec<Node>,
}

impl ToTokens for While {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { expr, body } = self;
        quote! {
            while #expr {
                #(#body)*
            }
        }
        .to_tokens(tokens)
    }
}

struct FunctionCall {
    function: TokenStream,
    args: Vec<TokenStream>,
}

impl ToTokens for FunctionCall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { function, args } = self;
        quote!(::htmx::IntoHtml::into_html(#function(#(Into::into(#args),)*), &mut __html);)
            .to_tokens(tokens)
    }
}

struct Element {
    open_tag: OpenTag,
    close_tag: Option<TokenStream>,
    attributes: Vec<Attribute>,
    body: ElementBody,
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            open_tag,
            close_tag,
            attributes,
            body,
        } = self;

        let close_tag = close_tag.iter();

        let mut attributes = attributes.clone();

        if !matches!(open_tag, OpenTag::Path(_)) {
            for attribute in &mut attributes {
                if let AttributeKey::Fn(name) = &attribute.key {
                    attribute.key = AttributeKey::from_str(name.to_string(), name.span())
                        .expect("idents should be valid attribute keys")
                }
            }
        };

        quote! {
            {{
                #( use ::htmx::__private::Unused; #close_tag::unused(); )*
                #open_tag #(#attributes)* #body
            }.into_html(&mut __html)}
        }
        .to_tokens(tokens)
    }
}

enum OpenTag {
    Path(TokenStream),
    String(String, Span),
    Expr(TokenStream),
}

impl OpenTag {
    fn from_str(name: String, span: Span) -> Result<OpenTag> {
        ensure!(
            name.to_ascii_lowercase().chars()
                .all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
            span,
            "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
            // TODO similar function but with css error: https://drafts.csswg.org/css-syntax-3/#non-ascii-ident-code-point
        );
        Ok(OpenTag::String(name, span))
    }
}

impl ToTokens for OpenTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            OpenTag::Path(path) => quote!(#path::new(&mut __html)),
            OpenTag::String(name, span) => {
                let name = quote_spanned!(*span=> #name);
                quote!(::htmx::CustomElement::new_unchecked(&mut __html, #name))
            }
            OpenTag::Expr(name) => quote!(quote!(::htmx::CustomElement::new(&mut __html, #name)),),
        }
        .to_tokens(tokens)
    }
}

#[derive(Clone)]
struct Attribute {
    key: AttributeKey,
    // TODO value encoding
    value: Option<TokenStream>,
}

impl ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { key, value } = self;
        let value = value.clone().unwrap_or_else(|| quote!(true));
        match key {
            AttributeKey::Fn(fun) => quote!(.#fun(#value)),
            AttributeKey::String(key, span) => {
                let key = quote_spanned!(*span => #key);
                quote!(.custom_attr_unchecked(#key, #value))
            }
            AttributeKey::Expr(key) => quote!(.custom_attr(#key, #value)),
        }
        .to_tokens(tokens);
    }
}

#[derive(Clone)]
enum AttributeKey {
    Fn(TokenStream),
    String(String, Span),
    Expr(TokenStream),
}

impl AttributeKey {
    fn from_str(value: String, span: Span) -> Result<AttributeKey> {
        ensure!(
            !value.to_string().chars().any(|c| c.is_whitespace()
                || c.is_control()
                || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')),
            span,
            "invalid key `{value}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0"
        );
        Ok(AttributeKey::String(value, span))
    }
}

enum ElementBody {
    Script(ScriptBody),
    Children(Vec<Node>),
}

impl ToTokens for ElementBody {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ElementBody::Script(script) => quote!(.body(#script)),
            ElementBody::Children(children) if children.is_empty() => {
                quote!(.close())
            }
            ElementBody::Children(children) => {
                quote!(.body(::htmx::Fragment(|mut __html: &mut ::htmx::Html| {#(#children)*})))
            }
        }
        .to_tokens(tokens)
    }
}

enum ScriptBody {
    String(LitStr),
    Expr(TokenStream),
}

impl ToTokens for ScriptBody {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ScriptBody::String(lit) => {
                let value = lit.value();
                let value = encode_script(&value);
                let mut value = Literal::string(&value);
                value.set_span(lit.span());
                quote!(RawSrc(#value)).to_tokens(tokens)
            }
            ScriptBody::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}
