use manyhow::{bail, manyhow, Result};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use quote_use::{
    parse_quote_use as parse_quote, quote_spanned_use as quote_spanned, quote_use as quote,
};

macro_rules! todo {
    () => {
        std::todo! {"{}:{}", file!(), line!()}
    };
    ($lit:literal $($tt:tt)*) => {
        std::todo! {concat!(file!(), line!(), $lit) $($tt)*}
    };
}

mod htmx;
// TODO split the two syntaxes => think of two names...
// HTML based syntax:
// html! {<div attr="hello"> { rust blocks } "literals" <_/>}
// more rusty kind of typst syntax:
// rtml! {div(attr: "hello") [ {rust block}, "literals"]}
#[manyhow(proc_macro)]
pub use htmx::html::html;
#[manyhow(proc_macro)]
pub use htmx::rusty::rtml;

// js!{  }

mod css;
#[manyhow(proc_macro)]
pub use css::css;

mod component;
#[manyhow(item_as_dummy, proc_macro_attribute)]
pub use component::component;

#[manyhow(proc_macro_derive(WriteHtml))]
pub fn write_html(
    syn::ItemStruct {
        ident,
        mut generics,
        fields,
        ..
    }: syn::ItemStruct,
) -> Result {
    let Some(field) = fields.iter().next() else {
        bail!(fields, "field for html buffer required for WriteHtml")
    };
    let field_type = &field.ty;
    let field = field
        .ident
        .as_ref()
        .map(ToTokens::to_token_stream)
        .unwrap_or_else(|| quote!(0));
    let where_clause = generics.make_where_clause();
    where_clause.predicates.push(parse_quote!(#field_type: ::htmx::WriteHtml));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::htmx::WriteHtml for #ident #ty_generics #where_clause {
            fn write_str(&mut self, s: &str) {
                self.#field.write_str(s);
            }

            fn write_char(&mut self, c: char) {
                self.#field.write_char(c);
            }

            fn write_fmt(&mut self, a: ::std::fmt::Arguments) {
                self.#field.write_fmt(a);
            }
        }
    })
}
