use htmx_script::{Script, ToJs};
use manyhow::{ensure, Result};
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use rstml::atoms::{CloseTag, OpenTag};
use rstml::node::{
    AttributeValueExpr, KeyedAttribute, KeyedAttributeValue, NodeAttribute, NodeBlock, NodeElement,
    NodeName,
};
use rstml::recoverable::Recoverable;
use syn::spanned::Spanned;
use syn::{parse2, Expr, ExprLit, ExprPath, Lit, LitStr, Stmt};

use crate::special_components::Node;
use crate::*;

pub fn htmx(input: TokenStream) -> Result {
    // https://github.com/rust-lang/rust-analyzer/issues/15572
    // let htmx = match (
    //     proc_macro_crate::crate_name("htmx"),
    //     std::env::var("CARGO_CRATE_NAME").as_deref(),
    // ) { (Ok(FoundCrate::Itself), Ok("htmx")) => quote!(crate),
    //   (Ok(FoundCrate::Name(name)), _) => { let ident = Ident::new(&name,
    //   Span::call_site()); quote!(::#ident) } _ => quote!(::htmx),
    // };
    let htmx = htmx_crate();

    let mut input = input.into_iter().peekable();

    let htmx = match input.peek() {
        Some(TokenTree::Ident(ident)) if ident == "crate" => {
            input.next();
            quote!(crate)
        }
        _ => htmx,
    };

    if input.peek().is_none() {
        return Ok(quote!(#htmx::Html::new()));
    }

    let nodes = rstml::Parser::new(
        rstml::ParserConfig::new()
            .recover_block(true)
            .element_close_use_default_wildcard_ident(false)
            .raw_text_elements(["script"].into())
            .custom_node(),
    )
    // TODO parse_recoverable
    .parse_simple(input.collect::<TokenStream>())?
    .into_iter()
    .map(|n| expand_node(n, &htmx, false))
    .collect::<Result>()?;

    Ok(quote! {
        #use #htmx::{ToHtml, Html, IntoHtmlElements};
        {
            use #htmx::native::*;
            let mut $htmx = Html::new();
            #nodes
            $htmx
        }
    })
}

pub fn expand_node(node: Node, htmx: &TokenStream, child: bool) -> Result {
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
            let (name, custom) = name_to_struct(name, htmx)?;
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
                let Some(Node::RawText(script)) = children.first() else {
                    unreachable!("script always raw text")
                };
                let script = script.into_token_stream();
                if let Ok(script) = parse2::<LitStr>(script.clone()) {
                    quote!($node = $node.child(#script);)
                } else if let Ok(block) =
                    parse2::<Recoverable<NodeBlock>>(script.clone()).map(Recoverable::inner)
                {
                    quote!($node = $node.child({#[allow(unused_braces)] #block});)
                } else {
                    let script: Script = parse2(script)?;
                    let script = script.to_java_script();
                    quote!($node = $node.child(#script);)
                }
            } else {
                expand_nodes(children, htmx, true)?
            };
            let main = quote!({let mut $node = #name #(.#attributes)*; #children $node.build()});

            let main = match close_tag {
                Some(CloseTag {
                    name: name @ NodeName::Path(_),
                    ..
                }) if !name.is_wildcard() => {
                    // If close_tag was specified, use it so coloring happens
                    quote!({let _: #name; #main})
                }
                _ => main,
            };
            if child {
                quote!($node = $node.child(&#main);)
            } else {
                quote!(#htmx::ToHtml::write_to_html(&#main, &mut $htmx);)
            }
        }
        Node::Block(_) | Node::Text(_) => {
            if child {
                quote!($node = $node.child(&{#[allow(unused_braces)] #node});)
            } else {
                quote!(#htmx::ToHtml::write_to_html(&{#[allow(unused_braces)] #node}, &mut $htmx);)
            }
        }
        Node::RawText(_) => todo!("{}", line!()),
        Node::Custom(c) => c.expand_node(htmx, child)?,
    })
}

pub fn expand_nodes(nodes: Vec<Node>, htmx: &TokenStream, child: bool) -> Result {
    nodes
        .into_iter()
        .map(|n| expand_node(n, htmx, child))
        .collect()
}

fn name_to_struct(name: NodeName, htmx: &TokenStream) -> Result<(TokenStream, bool)> {
    match name {
        NodeName::Path(path) => Ok((quote!(#path::builder()), false)),
        name @ NodeName::Punctuated(_) => {
            let name = name.to_string();
            ensure!(name.to_ascii_lowercase().chars().all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
         "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
        );
            Ok((quote!(#htmx::CustomElement::new_unchecked(#name)), true))
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
                let name = name.value();
                ensure!(name.to_ascii_lowercase().chars().all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
                 "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
                );
                Ok((quote!(#htmx::CustomElement::new_unchecked(#name)), true))
            } else {
                Ok((quote!(#htmx::CustomElement::new(#name)), true))
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
