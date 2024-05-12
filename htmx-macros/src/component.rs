use attribute_derive::{FlagOrValue, FromAttr};
use manyhow::{bail, ensure, error_message, Result};
use quote::ToTokens;
use syn::{
    parse2, Attribute, Expr, FnArg, Ident, ItemFn, ItemStruct, Pat, PatIdent, PatTupleStruct,
    PatType, Signature, Type,
};

use crate::*;

pub fn component(_input: TokenStream, item: TokenStream) -> Result {
    Ok(if let Ok(mut item) = parse2::<ItemStruct>(item.clone()) {
        // Generate
        // #[derive(typed_builder::TypedBuilder)]
        // #[builder(crate_module_path=htmx::__private::typed_builder)]
        // #[builder(build_method(into = Html))]

        // Set some defaults
        for field in item.fields.iter_mut() {
            type_attrs(
                field.ident.as_ref().ok_or_else(|| {
                    error_message!("Only structs with named fields are supported")
                })?,
                &field.ty,
                &mut field.attrs,
            )?;
        }
        quote! {
            #use ::htmx::Html;
            #use ::htmx::__private::typed_builder::{self, TypedBuilder};
            #[derive(TypedBuilder)]
            #[builder(crate_module_path = typed_builder)]
            #[builder(build_method(into = Html))]
            #item
        }
    } else if let Ok(ItemFn {
        attrs,
        vis,
        sig:
            Signature {
                ident,
                generics,
                inputs,
                output,
                ..
            },
        block,
    }) = parse2::<ItemFn>(item.clone())
    {
        // #[derive(typed_builder::TypedBuilder)]
        // #[builder(crate_module_path=::typed_builder)]
        // #[builder(build_method(into = Html))]
        // struct MyFnComponent {
        //     a: bool,
        //     b: String,
        // }
        //
        // impl Into<Html> for MyFnComponent {
        //     fn into(self) -> Html {
        //         let Self { a, b } = self;
        //         htmx! {crate
        //             <button disabled=a> {b} </button>
        //         }
        //     }
        // }
        ensure!(generics.params.is_empty(), "generics are not supported");
        let output = match output {
            syn::ReturnType::Default => quote!(::htmx::Html),
            syn::ReturnType::Type(_, ty) => ty.to_token_stream(),
        };

        let (fields, fieldpats): (Vec<_>, Vec<_>) = inputs
            .into_iter()
            .map(|arg| {
                // hi
                ensure!(let FnArg::Typed(PatType { mut attrs, pat, ty, .. }) = arg,
                    arg, "`self` is not supported");

                let ident = match &*pat {
                    Pat::Ident(PatIdent { ident, .. }) => ident,
                    // On tuples with a single field, take its ident
                    Pat::TupleStruct(PatTupleStruct { elems, .. })
                        if elems.len() == 1 && matches!(elems.first().unwrap(), Pat::Ident(_)) =>
                    {
                        let Some(Pat::Ident(PatIdent { ident, .. })) = elems.first() else {
                            unreachable!("pat should contain a single ident")
                        };
                        ident
                    }
                    pat => bail!(pat, "only named arguments and new type patterns are allowed";
                    help = "use `ident @ {}`", pat.into_token_stream();),
                };

                type_attrs(ident, &ty, &mut attrs)?;

                Ok((quote!(#(#attrs)* pub #ident: #ty,), quote!(#ident: #pat,)))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .unzip();

        // #attrs #vis struct
        quote! {
            #use ::htmx::__private::typed_builder::{self, TypedBuilder};
            #[derive(TypedBuilder)]
            #[builder(crate_module_path=typed_builder)]
            #[builder(build_method(into=#output))]
            #(#attrs)*
            #vis struct #ident {
                #(#fields)*
            }

            impl From<#ident> for #output {
                #[allow(non_shorthand_field_patterns)]
                fn from(#ident{ #(#fieldpats)* }: #ident) -> #output #block
            }
        }
    } else {
        bail!(item, "only functions and structs are supported")
    })
}
// todo derive macro
// #[component]
// fn MyFnComponent(a: bool, b: String) -> Html {
//     htmx! {crate
//         <button disabled=a> {b} </button>
//     }
// }
//
// // Generates
//
// #[derive(typed_builder::TypedBuilder)]
// #[builder(crate_module_path=::typed_builder)]
// #[builder(build_method(into = Html))]
// struct MyFnComponent {
//     a: bool,
//     b: String,
// }
//
// impl Into<Html> for MyFnComponent {
//     fn into(self) -> Html {
//         let Self { a, b } = self;
//         htmx! {crate
//             <button disabled=a> {b} </button>
//         }
//     }
// }
//
// // Using only struct
// #[derive(Component)]
// struct MyStructComponent {
//     a: bool,
//     b: String,
// }
// impl Into<Html> for MyStructComponent {
//     fn into(self) -> Html {
//         let Self { a, b } = self;
//         htmx! {crate
//             <button disabled=a> {b} </button>
//         }
//     }
// }
//
//

#[derive(FromAttr)]
#[attribute(ident = default)]
struct DefaultAttr(FlagOrValue<Expr>);

#[derive(FromAttr)]
#[attribute(ident = default)]
struct ChildrenAttr(bool);

fn type_attrs(name: &Ident, ty: &Type, attrs: &mut Vec<Attribute>) -> Result<()> {
    let DefaultAttr(default) = DefaultAttr::remove_attributes(attrs)?;
    let ChildrenAttr(children) = ChildrenAttr::remove_attributes(attrs)?;
    if let Type::Path(path) = ty {
        // TODO strip Option
        if default.is_flag() || path.path.is_ident("bool") || path.path.is_ident("Option") {
            attrs.push(parse_quote!(#[builder(default)]))
        }
        if let Some(default) = default.as_value() {
            attrs.push(parse_quote!(#[builder(default = #default)]))
        }
        if children
            || path.path.is_ident("Children")
            || path.path == parse_quote!(htmx::Children)
            || path.path == parse_quote!(::htmx::Children)
        {
            attrs.push(parse_quote! {#[builder(via_mutators, mutators(
                pub fn child(&mut self, __child: impl ::htmx::ToHtml) {
                    self.#name.push(__child);
                }
            ))]});
            attrs.push(parse_quote!(#[allow(missing_docs)]));
        }
    }
    attrs.push(parse_quote!(#[builder(setter(into))]));
    Ok(())
}
