use htmx_script::{Script, ToJs};
use manyhow::{ensure, ErrorMessage, Result};
use proc_macro2::TokenStream;
use proc_macro_utils::TokenStream2Ext;
use quote::ToTokens;
use rstml::atoms::{CloseTag, OpenTag};
use rstml::node::{
    AttributeValueExpr, KeyedAttribute, KeyedAttributeValue, NodeAttribute, NodeBlock, NodeElement,
    NodeName,
};
use rstml::recoverable::Recoverable;
use syn::spanned::Spanned;
use syn::{parse2, Expr, ExprLit, ExprPath, Lit, LitStr, Stmt};

use super::special_components::{Node, Special};
use crate::*;

pub fn html(input: TokenStream) -> Result {
    let mut parser = input.clone().parser();
    let (input, expr) =
        if let ( Some(expr), Some(_)) = (parser.next_expression(), parser.next_tt_fat_arrow()) {
            (parser.into_token_stream(), quote!(&mut #expr))
        } else {
            (input, quote!(::htmx::Html::new()))
        };

    let nodes = rstml::Parser::new(
        rstml::ParserConfig::new()
            .recover_block(true)
            .element_close_use_default_wildcard_ident(false)
            .custom_node::<Special>()
            .raw_text_elements(["script"].into()),
    )
    // TODO parse_recoverable
    .parse_simple(input)?
    .into_iter()
    .map(expand_node)
    .collect::<Result>()?;

    Ok(quote! {
        {
            use ::htmx::native::*;
            let mut __html = #expr;
            #nodes
            __html
        }
    })
}

pub fn expand_node(node: Node) -> Result {
    Ok(match node {
        Node::Comment(_) => todo!("{}", line!()),
        Node::Doctype(_) => todo!("{}", line!()),
        Node::Fragment(_) => todo!("{}", line!()),
        Node::Element(NodeElement {
            open_tag: OpenTag {
                name, attributes, ..
            },
            children,
            close_tag,
            ..
        }) => {
            let script = name.to_string() == "script";
            let (name, custom) = name_to_struct(name)?;
            let attributes = attributes
                .into_iter()
                .map(|attribute| match attribute {
                    NodeAttribute::Block(attr) => Ok(quote!(custom_attr(#attr, true))),
                    NodeAttribute::Attribute(KeyedAttribute {
                        key,
                        possible_value,
                    }) => match possible_value {
                        KeyedAttributeValue::Binding(_) => todo!("{}", line!()),
                        KeyedAttributeValue::Value(AttributeValueExpr { value, .. }) => {
                            attribute_key_to_fn(key, value, custom)
                        }
                        KeyedAttributeValue::None => attribute_key_to_fn(key, true, custom),
                    },
                })
                .collect::<Result<Vec<_>>>()?;
            let children = if children.is_empty() {
                quote!()
            } else if script {
                // TODO scripts
                let Some(Node::RawText(script)) = children.first() else {
                    unreachable!("script always raw text")
                };
                let script = script.into_token_stream();
                if let Ok(script) = parse2::<LitStr>(script.clone()) {
                    quote!(__html.child_expr(#script);)
                } else if let Ok(block) =
                    parse2::<Recoverable<NodeBlock>>(script.clone()).map(Recoverable::inner)
                {
                    quote!(__html.child_expr({#[allow(unused_braces)] #block});)
                } else {
                    let script: Script = parse2(script)?;
                    let script = script.to_java_script();
                    quote!(__html.child(#script);)
                }
            } else {
                expand_nodes(children)?
            };
            let body = (!children.is_empty()).then(|| quote!(let mut __html = __html.body();));
            let main = quote!({let mut __html = #name #(__html.#attributes;)* #body; #children});

            match close_tag {
                Some(CloseTag {
                    name: name @ NodeName::Path(_),
                    ..
                }) if !name.is_wildcard() => {
                    // If close_tag was specified, use it so coloring happens
                    quote!({#name::unused(); #main})
                }
                _ => main,
            }
        }
        Node::Block(_) | Node::Text(_) => {
            quote!(::htmx::ToHtml::to_html(&{#[allow(unused_braces)] #node}, &mut __html);)
        }
        Node::RawText(_) => todo!("{}", line!()),
        Node::Custom(c) => c.expand_node()?,
    })
}

pub fn expand_nodes(nodes: Vec<Node>) -> Result {
    nodes.into_iter().map(expand_node).collect()
}

pub fn ensure_tag_name(name: String, span: impl ToTokens) -> Result<String, ErrorMessage> {
    ensure!(
        name.to_ascii_lowercase().chars()
            .all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
        span,
        "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
        // TODO similar function but with css error: https://drafts.csswg.org/css-syntax-3/#non-ascii-ident-code-point
    );
    Ok(name)
}

fn name_to_struct(name: NodeName) -> Result<(TokenStream, bool)> {
    match name {
        NodeName::Path(path) => Ok((quote!(#path::new(&mut __html);), false)),
        name @ NodeName::Punctuated(_) => {
            let name = ensure_tag_name(name.to_string(), name)?;
            Ok((
                quote!(::htmx::CustomElement::new_unchecked(&mut __html, #name);),
                true,
            ))
        }
        // This {...}
        NodeName::Block(name) => {
            if let [
                Stmt::Expr(
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(name),
                        ..
                    }),
                    None,
                ),
            ] = &name.stmts[..]
            {
                let name = ensure_tag_name(name.value(), name)?;
                Ok((
                    quote!(::htmx::CustomElement::new_unchecked(&mut __html, #name);),
                    true,
                ))
            } else {
                Ok((
                    quote!(::htmx::CustomElement::new(&mut __html, #name);),
                    true,
                ))
            }
        }
    }
}

fn attribute_key_to_fn(name: NodeName, value: impl ToTokens, custom: bool) -> Result {
    Ok(match name {
        NodeName::Path(ExprPath { path, .. }) if !custom && path.get_ident().is_some() => {
            quote!(#path(#value))
        }
        NodeName::Path(ExprPath { path, .. })
            if path.segments.first().is_some_and(|hx| hx.ident == "hx") =>
        {
            {
                let sident = path
                    .segments
                    .iter()
                    .map(|i| i.ident.to_string().replace('_', "-"))
                    // hx::swap::oob
                    .collect::<Vec<_>>()
                    .join("-");
                quote_spanned!(path.span()=> custom_attr(#sident, #value))
            }
        }
        // This {...}
        name @ (NodeName::Punctuated(_) | NodeName::Path(_)) => {
            let sname = name.to_string();
            quote_spanned!(name.span()=>  custom_attr(#sname, #value))
        }
        name @ NodeName::Block(_) => quote_spanned!(name.span()=>  custom_attr(#name, #value)),
    })
}
