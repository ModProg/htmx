use manyhow::{manyhow, Result};
use proc_macro2::{Ident, TokenStream};
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
pub use htmx::html;

#[manyhow(proc_macro)]
pub use htmx::rtml;

// js!{  }

mod css;
#[manyhow(proc_macro)]
pub use css::css;

mod component;
#[manyhow(item_as_dummy, proc_macro_attribute)]
pub use component::component;
