use htmx_script::{Script, ToJs};
use manyhow::{bail, Result};
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use rstml::atoms::OpenTag;
use rstml::node::{
    AttributeValueExpr, KeyedAttribute, KeyedAttributeValue, Node, NodeAttribute, NodeBlock,
    NodeElement, NodeName,
};
use rstml::recoverable::Recoverable;
use syn::spanned::Spanned;
use syn::{parse2, ExprPath, LitStr};

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

    let nodes = rstml::Parser::new(
        rstml::ParserConfig::new()
            .recover_block(true)
            .element_close_use_default_wildcard_ident(false)
            .raw_text_elements(["script"].into()),
    )
    // TODO parse_recoverable
    .parse_simple(input.collect::<TokenStream>())?
    .into_iter()
    .map(expand_node)
    .collect::<Result<Vec<TokenStream>>>()?;
    let mut nodes = nodes.into_iter().peekable();

    Ok(if nodes.peek().is_some() {
        quote! {
        #use #htmx::{ToHtml, Html, IntoHtmlElements};
        {
            use #htmx::native::*;
            let mut $htmx = Html::new();
            #(
                ToHtml::write_to_html(&#nodes, &mut $htmx);
            )*
            $htmx
        }}
    } else {
        quote!(#htmx::Html::new())
    })
}

fn expand_node(node: Node) -> Result {
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
            let name = name_to_struct(name)?;
            let attributes = attributes
                .into_iter()
                .map(|attribute| match attribute {
                    NodeAttribute::Block(_) => {
                        bail!(attribute, "dynamic attribute names not supported")
                    }
                    NodeAttribute::Attribute(KeyedAttribute {
                        key,
                        possible_value,
                    }) => match possible_value {
                        KeyedAttributeValue::Binding(_) => todo!("{}", line!()),
                        KeyedAttributeValue::Value(AttributeValueExpr { value, .. }) => {
                            attribute_key_to_fn(key, value)
                        }
                        KeyedAttributeValue::None => attribute_key_to_fn(key, true),
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
                    quote!(.child(#script))
                } else if let Ok(block) =
                    parse2::<Recoverable<NodeBlock>>(script.clone()).map(Recoverable::inner)
                {
                    quote!( .child({#[allow(unused_braces)] #block}))
                } else {
                    let script: Script = parse2(script)?;
                    let script = script.to_java_script();
                    quote!(.child(#script))
                }
            } else {
                let children = children
                    .into_iter()
                    .map(expand_node)
                    .collect::<Result<Vec<_>>>()?;
                quote!(#(.child(#children))*)
            };
            let main = quote!(#name::builder() #(.#attributes)* #children .build());
            match close_tag.map(|tag| name_to_struct(tag.name)) {
                // If close_tag was specified, use it so coloring happens
                Some(Ok(close_tag)) if close_tag == name => quote!({let _ :#close_tag;#main}),
                _ => main,
            }
        }
        Node::Block(_) | Node::Text(_) => quote!( {#[allow(unused_braces)] #node}),
        Node::RawText(_) => todo!("{}", line!()),
        Node::Custom(c) => match c {},
    })
}

fn name_to_struct(name: NodeName) -> Result<ExprPath> {
    match name {
        NodeName::Path(path) => Ok(path),
        // This {...}
        NodeName::Punctuated(_) | NodeName::Block(_) => {
            bail!(name, "Only normal identifiers are allowd as node names")
        }
    }
}

fn attribute_key_to_fn(name: NodeName, value: impl ToTokens) -> Result {
    match name {
        NodeName::Path(ExprPath { path, .. }) => Ok({
            let sident = path
                .segments
                .iter()
                .map(|i| i.ident.to_string().replace('_', "-"))
                .collect::<Vec<_>>()
                .join("-");
            if let Some(sident) = sident.strip_prefix("data-") {
                quote_spanned!(path.span()=> data(#sident, #value))
            } else if sident.starts_with("hx-") {
                quote_spanned!(path.span()=> data(#sident, #value))
            } else if let Some(ident) = path.get_ident() {
                quote!(#ident(#value))
            } else {
                bail!(path, "only `data::` or `hx::` are allowed as path prefix");
            }
        }),
        // This {...}
        NodeName::Punctuated(_) => {
            todo!("handle data-...")
        }
        NodeName::Block(_) => {
            bail!(
                name,
                "Only normal identifiers are allowd as attribute names"
            )
        }
    }
}
