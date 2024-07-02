use attribute_derive::{FlagOrValue, FromAttr};
use manyhow::{bail, ensure, Result};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, ToTokens, TokenStreamExt};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Paren};
use syn::{
    AssocType, Attribute, Expr, FnArg, GenericArgument, Generics, Ident, Lifetime, Pat, PatIdent,
    PatTupleStruct, PatType, PathArguments, ReturnType, Token, Type, TypeImplTrait, TypeParamBound,
    Visibility,
};
use syn_derive::ToTokens;

use crate::*;

enum Arg {
    Body(Ident),
    Field(Field),
}

#[derive(Debug)]
struct Field {
    name: Ident,
    ty: Type,
    pat: Pat,
    default: FlagOrValue<Expr>,
    default_type: Option<Type>,
    doc_attrs: TokenStream,
}

impl Field {
    fn generic(&self) -> Ident {
        Ident::new(
            &ident_case::RenameRule::PascalCase.apply_to_field(self.name.to_string()),
            self.name.span(),
        )
    }

    fn field(&self) -> TokenStream {
        let name = &self.name;
        let generic = self.generic();
        quote!(#name: #generic)
    }

    fn name(&self) -> &Ident {
        &self.name
    }

    fn unset(&self) -> TokenStream {
        if let Some(default_type) = &self.default_type {
            return quote!(#default_type);
        }
        if self.is_optional() {
            if let Type::ImplTrait(TypeImplTrait { bounds, .. }) = &self.ty {
                for t in bounds {
                    if let TypeParamBound::Trait(t) = t {
                        let t = t.path.segments.last().unwrap();
                        if t.ident == "IntoIterator" {
                            if let PathArguments::AngleBracketed(t) = &t.arguments {
                                for t in &t.args {
                                    if let GenericArgument::AssocType(t) = &t {
                                        if t.ident == "Item" {
                                            let t = &t.ty;
                                            return quote!(::htmx::__private::Empty::<#t>);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        quote!(::htmx::__private::Unset)
    }

    fn unset_value(&self) -> TokenStream {
        if let Some(default_type) = &self.default_type {
            quote!(<#default_type>::default())
        } else {
            self.unset()
        }
    }

    fn is_optional(&self) -> bool {
        if let Type::Path(path) = &self.ty {
            path.path.is_ident("bool")
                || path.path.segments.len() == 1
                    && path.path.segments.first().unwrap().ident == "Option"
                || !self.default.is_none()
        } else {
            !self.default.is_none()
        }
    }

    fn destructure(&self) -> TokenStream {
        let name = &self.name;
        let pat = &self.pat;
        match &self.default {
            FlagOrValue::Value(default) => quote!(let #pat = #name.get_or_else(|| #default);),
            _ if self.is_impl_trait() && self.is_optional() => quote! {},
            _ if self.is_impl_trait() => quote!(let ::htmx::__private::Set(#name) = #name;),
            _ if self.is_optional() => quote!(let #pat = #name.get_or_default();),
            _ => quote!(let ::htmx::__private::Set(#name) = #name;),
        }
    }

    fn get_generics(&self, base: &Ident) -> Option<TokenStream> {
        if let Type::ImplTrait(ty) = &self.ty {
            let mut tokens = TokenStream::new();
            desugar_impl(&mut tokens, ty.clone(), base);
            Some(tokens)
        } else {
            None
        }
    }

    fn is_impl_trait(&self) -> bool {
        matches!(self.ty, Type::ImplTrait(_))
    }
}

fn desugar_impl(tokens: &mut TokenStream, ty: TypeImplTrait, base: &Ident) {
    let mut count = 0;
    let mut bounds = ty.bounds;
    for bound in &mut bounds {
        if let TypeParamBound::Trait(bound) = bound {
            for segment in &mut bound.path.segments {
                if let PathArguments::AngleBracketed(arguments) = &mut segment.arguments {
                    for argument in &mut arguments.args {
                        if let GenericArgument::Type(ty)
                        | GenericArgument::AssocType(AssocType { ty, .. }) = argument
                        {
                            if let Type::ImplTrait(tr) = ty {
                                let gen_ident = format_ident!("{base}_{count}");
                                count += 1;
                                desugar_impl(tokens, tr.clone(), &gen_ident);
                                tokens.extend(quote!(,));
                                *ty = parse_quote!(#gen_ident);
                            };
                        }
                    }
                }
            }
        }
    }
    tokens.extend(quote!(#base: #bounds));
}

impl TryFrom<FnArg> for Arg {
    type Error = manyhow::Error;

    fn try_from(arg: FnArg) -> std::result::Result<Self, Self::Error> {
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
            pat => bail!(
                pat, "only named arguments and new type patterns are allowed";
                help = "use `ident @ {}`", pat.into_token_stream();
            ),
        };

        if ident == "body" {
            return Ok(Arg::Body(ident.clone()))
        }

        let DefaultAttr(mut default) = DefaultAttr::remove_attributes(&mut attrs)?;
        let DefaultType(default_type) = DefaultType::remove_attributes(&mut attrs)?;
        // let ChildrenAttr(children) = ChildrenAttr::remove_attributes(attrs)?;

        if default_type.is_some() && default.is_none() {
            default = FlagOrValue::Flag;
        }

        let doc_attrs = attrs
            .into_iter()
            .filter(|a| a.path().is_ident("doc"))
            .map(ToTokens::into_token_stream)
            .collect();

        Ok(Arg::Field(Field {
            name: ident.clone(),
            pat: *pat,
            ty: *ty,
            default,
            default_type,
            doc_attrs,
        }))
        // Ok((quote!(#(#attrs)* pub #ident: #ty,), quote!(#ident: #pat,)))
    }
}

#[derive(syn_derive::Parse, ToTokens)]
pub struct Component {
    #[parse(Attribute::parse_outer)]
    #[to_tokens(|ts, t| ts.append_all(t))]
    attrs: Vec<Attribute>,
    vis: Visibility,
    fn_token: Token![fn],
    name: Ident,
    generics: Generics,
    #[syn(parenthesized)]
    paren_token: Paren,
    #[syn(in = paren_token)]
    #[parse(Punctuated::parse_terminated)]
    inputs: Punctuated<FnArg, Token![,]>,
    output: ReturnType,
    #[syn(braced)]
    brace_token: Brace,
    #[syn(in = brace_token)]
    body: TokenStream,
}

pub fn component(
    _input: TokenStream,
    Component {
        attrs,
        vis,
        name: struct_name,
        generics,
        inputs,
        output,
        body: fn_body,
        ..
    }: Component,
) -> Result {
    ensure!(generics.params.is_empty(), "generics are not supported");
    if let ReturnType::Type(_, t) = &output {
        if let Type::Tuple(t) = &**t {
            if !t.elems.is_empty() {
                bail!(output, "expected `()` return type");
            }
        } else {
            bail!(output, "expected `()` return type");
        }
    }

    let (body, args) = inputs.into_iter().map(Arg::try_from).try_fold(
        Default::default(),
        |mut acc, arg| -> Result<(Option<Ident>, Vec<Field>)> {
            match arg? {
                Arg::Body(body) => {
                    ensure!(acc.0.is_none(), body, "multiple `body` arguments");
                    acc.0 = Some(body);
                }
                Arg::Field(field) => acc.1.push(field),
            };
            Ok(acc)
        },
    )?;

    let body = body.unwrap_or_else(|| Ident::new("body", Span::call_site()));

    let html_lt = Lifetime::new("'html", Span::call_site());

    let fields = args.iter().map(Field::field);
    let generics = args.iter().map(Field::generic);
    let unsets_types: Vec<_> = args.iter().map(Field::unset).collect();
    let unset_values: Vec<_> = args.iter().map(Field::unset_value).collect();
    let field_names: Vec<_> = args.iter().map(Field::name).collect();

    let optional_gens = args
        .iter()
        .filter(|&f| (f.is_optional() && !f.is_impl_trait()))
        .map(|f| {
            let g = f.generic();
            let ty = &f.ty;
            quote!(#g: ::htmx::__private::Settable<#ty>)
        })
        .chain(args.iter().filter_map(|f| f.get_generics(&f.generic())));

    let mandatory_gens = args.iter().map(|f| {
        if f.is_optional() {
            f.generic().into_token_stream()
        } else if f.is_impl_trait() {
            let gen = f.generic();
            quote!(::htmx::__private::Set<#gen>)
        } else {
            let ty = &f.ty;
            quote!(::htmx::__private::Set<#ty>)
        }
    });

    let field_destructure = args.iter().map(Field::destructure);

    let mut setters = vec![];
    for i in 0..args.len() {
        let mut impl_gens = vec![];
        let mut unset_gens = vec![];
        let mut set_gens = vec![];
        let mut destructure = vec![];
        let mut structure = vec![];

        let field @ Field {
            name: field_name,
            doc_attrs,
            ..
        } = &args[i];
        let gen = field.generic();

        let mut fn_gen = None;

        for (idx, field @ Field { ty, name, .. }) in args.iter().enumerate() {
            let generic = field.generic().to_token_stream();
            if idx == i {
                unset_gens.push(field.unset());
                destructure.push(quote!(#name: _));
                if let Some(bounds) = field.get_generics(&gen) {
                    fn_gen = Some(quote!(#bounds));
                    set_gens.push(quote!(::htmx::__private::Set<#generic>));
                    structure.push(quote!(#name: ::htmx::__private::Set(#name)));
                } else {
                    fn_gen = Some(quote!(#gen: Into<#ty>));
                    set_gens.push(quote!(::htmx::__private::Set<#ty>));
                    structure.push(quote!(#name: ::htmx::__private::Set(#name.into())));
                };
            } else {
                impl_gens.push(quote!(#generic));
                unset_gens.push(quote!(#generic));
                set_gens.push(quote!(#generic));
                destructure.push(quote!(#name));
                structure.push(quote!(#name));
            }
        }

        let already_set_msg = format!("{field_name} was alredy set");
        let already_set_ty = format_ident!("{field_name}_was_alredy_set");

        let extra_gen = field.is_impl_trait().then_some(&gen).into_iter();

        setters.push(quote! {
          impl<#html_lt, #(#impl_gens),*> #struct_name<#html_lt, #(#unset_gens),*> {
              #doc_attrs
              pub fn #field_name<#fn_gen>(self, #field_name: #gen)
                  -> #struct_name<#html_lt, #(#set_gens),*> {
                  let Self {
                      html,
                      #(#destructure),*
                  } = self;
                  #struct_name {
                      html,
                      #(#structure),*
                  }
              }
          }

          #[allow(non_camel_case_types)]
          pub struct #already_set_ty;

          impl<#html_lt, #(#extra_gen,)* #(#impl_gens),*> #struct_name<#html_lt, #(#set_gens),*> {
              #[doc(hidden)]
              #[deprecated = #already_set_msg]
              #[allow(unused)]
              pub fn #field_name<__Gen>(
                  self,
                  #field_name: __Gen, _: #already_set_ty
              ) -> Self {
                  self
              }
          }
        });
    }

    // #attrs #vis struct
    Ok(quote! {
        #use ::htmx::__private::{Set};

        #(#attrs)*
        #[must_use = "call body or close"]
        #vis struct #struct_name<#html_lt, #(#generics),*> {
            html: ::core::marker::PhantomData<&#html_lt ()>,
            #(#fields),*
        }
        const _: () = {
            use ::core::default::Default as _;
            impl<#html_lt> #struct_name<#html_lt, #(#unsets_types),*> {
                pub fn new(_: &mut ::htmx::Html) -> Self {
                    Self {
                        html: ::core::marker::PhantomData,
                        #(#field_names: #unset_values),*
                    }
                }
            }

            #(#setters)*

            impl<#html_lt, #(#optional_gens),*> #struct_name<#html_lt, #(#mandatory_gens),*> {
                pub fn body(self, #body: impl ::htmx::IntoHtml + #html_lt) -> impl ::htmx::IntoHtml + #html_lt {
                    let Self {
                        html: _,
                        #(#field_names),*
                    } = self;

                    #(#field_destructure;)*


                    ::htmx::Fragment(move |__html: &mut ::htmx::Html|(||{#fn_body})().into_html(__html))
                }

                pub fn close(self)  -> impl ::htmx::IntoHtml + #html_lt {
                    self.body(::htmx::Fragment::EMPTY)
                }
            }
        };
    })
}

#[derive(FromAttr)]
#[attribute(ident = default)]
struct DefaultAttr(FlagOrValue<Expr>);

#[derive(FromAttr)]
#[attribute(ident = default_type)]
struct DefaultType(Option<Type>);
