use htmx_script::{Script, ToJs};
use manyhow::{ensure, Error, ErrorMessage, Result};
use proc_macro2::TokenStream;
use quote::ToTokens;
use rstml::atoms::{CloseTag, OpenTag};
use rstml::node::{
    AttributeValueExpr, KeyedAttribute, KeyedAttributeValue, NodeAttribute, NodeBlock, NodeElement,
    NodeFragment, NodeName,
};
use rstml::recoverable::Recoverable;
use syn::spanned::Spanned;
use syn::{parse2, Expr, ExprLit, ExprPath, Lit, LitStr, Stmt};

use super::special_components::{Node, Special};
use super::try_into_iter;
use crate::*;

pub fn html(input: TokenStream) -> Result {
    let nodes = rstml::Parser::new(
        rstml::ParserConfig::new()
            .recover_block(true)
            .element_close_use_default_wildcard_ident(false)
            .custom_node::<Special>()
            .raw_text_elements(["script"].into()),
    )
    // TODO parse_recoverable
    .parse_simple(input)?;

    super::expand_nodes(nodes)
}

impl TryFrom<Node> for super::Node {
    type Error = Error;

    fn try_from(value: Node) -> std::result::Result<Self, Self::Error> {
        match value {
            Node::Comment(comment) => bail!(comment, "html comments are not supported"),
            Node::Doctype(doc_type) => bail!(doc_type, "doc typ is set automatically"),
            Node::Fragment(NodeFragment { tag_open, .. }) => bail!(tag_open, "missing tag name"),
            Node::Element(element) => Ok(super::Node::Element(element.try_into()?)),
            Node::Block(block) => Ok(super::Node::Block(block.into_token_stream())),
            Node::Text(text) => Ok(super::Node::String(text.value)),
            Node::RawText(text) => bail!(
                text.into_token_stream().into_iter().next(),
                "expected `<`, `{{` or `\"`"
            ),
            Node::Custom(special) => special.try_into(),
        }
    }
}

impl TryFrom<NodeElement<Special>> for super::Element {
    type Error = Error;

    fn try_from(value: NodeElement<Special>) -> std::result::Result<Self, Self::Error> {
        let NodeElement {
            open_tag,
            children,
            close_tag,
        } = value;
        Ok(super::Element {
            close_tag: close_tag.and_then(|ct| match ct.name {
                NodeName::Path(p) if !ct.name.is_wildcard() => Some(p.into_token_stream()),
                _ => None,
            }),
            attributes: try_into_iter(open_tag.attributes)?,
            body: if !children.is_empty()
                && matches!(&open_tag.name, NodeName::Path(p) if p.path.is_ident("script"))
            {
                let Some(Node::RawText(script)) = children.first() else {
                    unreachable!("script always raw text")
                };
                let script = script.into_token_stream();
                if let Ok(script) = parse2::<LitStr>(script.clone()) {
                    super::ElementBody::Script(super::ScriptBody::String(script))
                } else if let Ok(block) =
                    parse2::<Recoverable<NodeBlock>>(script.clone()).map(Recoverable::inner)
                {
                    super::ElementBody::Script(super::ScriptBody::Expr(block.into_token_stream()))
                } else {
                    let script: Script = parse2(script)?;
                    let script = script.to_java_script();
                    // quote!(__html.body(#script);)
                    super::ElementBody::Script(super::ScriptBody::Expr(script.into_token_stream()))
                }
            } else {
                super::ElementBody::Children(try_into_iter(children)?)
            },
            open_tag: open_tag.name.try_into()?,
        })
    }
}

fn string_from_block(block: &syn::Block) -> Option<&LitStr> {
    if let [
        Stmt::Expr(
            Expr::Lit(ExprLit {
                lit: Lit::Str(lit), ..
            }),
            None,
        ),
    ] = &block.stmts[..]
    {
        Some(lit)
    } else {
        None
    }
}

impl TryFrom<NodeName> for super::OpenTag {
    type Error = Error;

    fn try_from(value: NodeName) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            NodeName::Path(path) => super::OpenTag::Path(path.into_token_stream()),
            name @ NodeName::Punctuated(_) => {
                super::OpenTag::from_str(name.to_string(), name.span())?
            }
            NodeName::Block(name) => {
                if let Some(name) = string_from_block(&name) {
                    super::OpenTag::from_str(name.value(), name.span())?
                } else {
                    super::OpenTag::Expr(name.into_token_stream())
                }
            }
        })
    }
}

impl TryFrom<NodeAttribute> for super::Attribute {
    type Error = Error;

    fn try_from(value: NodeAttribute) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            NodeAttribute::Block(name) => super::Attribute {
                key: if let Some(name) = name.try_block().and_then(string_from_block) {
                    super::AttributeKey::from_str(name.value(), name.span())?
                } else {
                    super::AttributeKey::Expr(name.into_token_stream())
                },
                value: None,
            },
            NodeAttribute::Attribute(attribute) => super::Attribute {
                value: attribute.value().map(ToTokens::into_token_stream),
                key: attribute.key.try_into()?,
            },
        })
    }
}

impl TryFrom<NodeName> for super::AttributeKey {
    type Error = Error;

    fn try_from(value: NodeName) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            NodeName::Path(p) if p.path.get_ident().is_some() => {
                super::AttributeKey::Fn(p.into_token_stream())
            }
            NodeName::Path(p) if p.path.segments.first().is_some_and(|hx| hx.ident == "hx") => {
                let sident = p
                    .path
                    .segments
                    .iter()
                    .map(|i| i.ident.to_string().replace('_', "-"))
                    // hx::swap::oob
                    .collect::<Vec<_>>()
                    .join("-");
                super::AttributeKey::from_str(sident, p.span())?
            }
            key @ (NodeName::Punctuated(_) | NodeName::Path(_)) => {
                super::AttributeKey::from_str(key.to_string(), key.span())?
            }
            NodeName::Block(block) => {
                if let Some(key) = string_from_block(&block) {
                    super::AttributeKey::from_str(key.value(), key.span())?
                } else {
                    super::AttributeKey::Expr(block.into_token_stream())
                }
            }
        })
    }
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
            let (name, node_type) = name_to_struct(name)?;
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
                            attribute_key_to_fn(key, value, matches!(node_type, NodeType::Custom))
                        }
                        KeyedAttributeValue::None => {
                            attribute_key_to_fn(key, true, matches!(node_type, NodeType::Custom))
                        }
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
                    // quote!(__html.body(#script);)
                    quote!(::htmx::ToScript::to_script(&#script, &mut __html);)
                } else if let Ok(block) =
                    parse2::<Recoverable<NodeBlock>>(script.clone()).map(Recoverable::inner)
                {
                    // quote!(__html.body({#[allow(unused_braces)] #block});)
                    quote!(::htmx::ToScript::to_script(&{# [allow(unused_braces)] #block}, &mut __html);)
                } else {
                    let script: Script = parse2(script)?;
                    let script = script.to_java_script();
                    // quote!(__html.body(#script);)
                    quote!(::htmx::ToScript::to_script(&#script, &mut __html);)
                }
            } else {
                expand_nodes(children)?
            };
            let close_arg = if matches!(node_type, NodeType::Component) {
                quote!(&mut __html)
            } else {
                quote!()
            };
            let body = if children.is_empty() {
                quote!(.close(#close_arg))
            } else {
                quote!(.body(::htmx::Fragment(|mut __html: &mut ::htmx::Html| {#children}), #close_arg))
            };
            let main = quote!({{let mut __html = #name #(.#attributes)*; __html}#body;});

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
            quote!(::htmx::IntoHtml::into_html({#[allow(unused_braces)] #node}, &mut __html);)
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

enum NodeType {
    Native,
    Component,
    Custom,
}

fn name_to_struct(name: NodeName) -> Result<(TokenStream, NodeType)> {
    match name {
        NodeName::Path(path)
            if path
                .path
                .get_ident()
                .is_some_and(|i| !i.to_string().contains(char::is_uppercase)) =>
        {
            Ok((quote!(#path::new(&mut __html)), NodeType::Native))
        }
        NodeName::Path(path) => Ok((quote!(#path::new()), NodeType::Component)),
        name @ NodeName::Punctuated(_) => {
            let name = ensure_tag_name(name.to_string(), name)?;
            Ok((
                quote!(::htmx::CustomElement::new_unchecked(&mut __html, #name)),
                NodeType::Custom,
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
                    quote!(::htmx::CustomElement::new_unchecked(&mut __html, #name)),
                    NodeType::Custom,
                ))
            } else {
                Ok((
                    quote!(::htmx::CustomElement::new(&mut __html, #name)),
                    NodeType::Custom,
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
