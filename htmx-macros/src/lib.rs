use manyhow::manyhow;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote_use::{
    parse_quote_use as parse_quote, quote_spanned_use as quote_spanned, quote_use as quote,
};

fn htmx_crate() -> TokenStream {
    match proc_macro_crate::crate_name("htmx") {
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        _ => quote!(::htmx),
    }
}

mod htmx;
#[manyhow(proc_macro)]
pub use htmx::htmx;

mod component;
#[manyhow(item_as_dummy, proc_macro_attribute)]
pub use component::component;
