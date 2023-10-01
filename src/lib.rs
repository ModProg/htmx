#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::wildcard_imports)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! Library for doing serverside rendering of htm{l,x} using a macro.
//!
//! # `htmx!` macro
//!
//! The [`htmx!`] macro allows to write [`Html`] inside your rust code allowing
//! to include rust values and instantiate [custom
//! components](#custom-components).
//!
//! ```
//! # use htmx::htmx;
//! ```
//!
//! # Custom Components
//!
//! For more documentation see the individual items and the [examples](https://github.com/ModProg/htmx/tree/main/example).

// This makes `::htmx` work in the proc-macro expansions.
extern crate self as htmx;

use std::fmt::Write;

use derive_more::Display;
use serde::Serialize;

pub mod attributes;
pub mod native;
mod utils;

#[doc(hidden)]
pub mod __private {
    pub use typed_builder;
}

/// Allows to make a component from a function or struct.
///
/// # Struct
/// A struct needs to have an <code>[Into]<[Html]></code> implementation.
/// ```
/// # use htmx::{component, htmx, Html};
/// #[component]
/// struct Component {
///     a: bool,
///     b: String,
/// }
///
/// impl From<Component> for Html {
///     fn from(Component { a, b }: Component) -> Self {
///         htmx! {
///             <button disabled=a>{b}</button>
///         }
///     }
/// }
///
/// htmx! {
///     <Component a b="Disabled Button"/>
///     <Component a=true b="Disabled Button"/>
///     <Component a=false b="Enabled Button"/>
///     <Component b="Enabled Button"/>
/// };
/// ```
///
/// In the case of struct components, all the [`#[component]`](component) macro
/// does, is generating a derive for [`typed_builder::TypedBuilder`], setting
/// some default attributes, like making [`bool`s](bool) optional and making the
/// builder accept [`Into`].
///
/// # Function
/// Instead of structs function components are more succinct.
///
/// By convention function components are `PascalCase` as well, ensuring they
/// cannot conflict with native always lowercase elements.
/// ```
/// # use htmx::{component, htmx, Html};
/// #[component]
/// fn Component(a: bool, b: String) -> Html {
///     htmx! {
///         <button disabled=a>{b}</button>
///     }
/// }
///
/// htmx! {
///     <Component a b="Disabled Button"/>
///     <Component a=true b="Disabled Button"/>
///     <Component a=false b="Enabled Button"/>
///     <Component b="Enabled Button"/>
/// };
/// ```
/// The [`#[component]`](component) macro on functions, generates the struct and
/// [`Into`] implementation [above](#struct), making the two equivalent.
pub use htmx_macros::component;
pub use htmx_macros::*;
pub use utils::*;

#[cfg(feature = "actix-web")]
mod actix;
#[cfg(feature = "actix-web")]
pub use actix::*;

#[cfg(feature = "axum")]
mod axum;
#[cfg(feature = "axum")]
pub use axum::*;

const DOCTYPE: &str = "<!DOCTYPE html>";

/// Trait used with the custom rust like js in `<script>` tags using the
/// [`htmx!`] macro.
///
/// It is not used per fully qualified syntax so you are able to provide a
/// custom `to_js()` method on types that implement [`Serialize`].
///
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct CustomToJs(String);
///
/// impl CustomToJs {
///     // returns custom string instead of `Serialize` implementation
///     // `htmx!` will prefere this function.
///     fn to_js(&self) -> String {
///         format!("\"custom: {}\"", self.0)
///     }
/// }
/// ```
pub trait ToJs {
    /// Converts into a string of JS code.
    /// This string should be an expression.
    fn to_js(&self) -> String;
}

impl<T: Serialize> ToJs for T {
    fn to_js(&self) -> String {
        serde_json::to_string(self).expect("Serialization shouldn't fail.")
    }
}

/// Html
///
/// Can be returned from http endpoints or converted to a string.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
#[must_use]
pub struct Html(String);

impl Html {
    /// Creates a piece of Html
    pub fn new() -> Self {
        Self(DOCTYPE.into())
    }
}

impl Default for Html {
    fn default() -> Self {
        Self::new()
    }
}

impl ToHtml for Html {
    fn write_to_html(&self, html: &mut Html) {
        html.0.write_str(&self.0[DOCTYPE.len()..]).unwrap();
    }

    fn to_html(&self) -> Html {
        self.clone()
    }

    fn into_html(self) -> Html
    where
        Self: Sized,
    {
        self
    }
}

impl<T: ToHtml + ?Sized> ToHtml for &T {
    fn write_to_html(&self, out: &mut Html) {
        (*self).write_to_html(out);
    }

    fn to_html(&self) -> Html {
        (*self).to_html()
    }
}

/// Converts to [`Html`], either by appending to existing [`Html`] or by
/// creating a new one.
pub trait ToHtml {
    /// Appends to existing [`Html`].
    ///
    /// Implementers should only implement this method.
    fn write_to_html(&self, html: &mut Html);

    /// Converts to [`Html`].
    fn to_html(&self) -> Html {
        let mut html = Html::default();
        self.write_to_html(&mut html);
        html
    }

    /// Converts to [`Html`].
    fn into_html(self) -> Html
    where
        Self: Sized,
    {
        self.to_html()
    }
}

impl<T: ToHtml> ToHtml for Option<T> {
    fn write_to_html(&self, html: &mut Html) {
        if let Some(t) = self {
            t.write_to_html(html);
        }
    }
}

/// ```
/// use htmx::ToHtml;
/// vec!["1", "2"].to_html();
/// ```
impl<T: ToHtml> ToHtml for [T] {
    fn write_to_html(&self, html: &mut Html) {
        for e in self {
            e.write_to_html(html);
        }
    }
}

impl<T: ToHtml, const N: usize> ToHtml for [T; N] {
    fn write_to_html(&self, html: &mut Html) {
        self.as_slice().write_to_html(html);
    }
}

impl<T: ToHtml> ToHtml for Vec<T> {
    fn write_to_html(&self, html: &mut Html) {
        self.as_slice().write_to_html(html);
    }
}

/// Canonical type to accept children in component.
///
/// ```
/// # use htmx::{htmx, component, Children};
///
/// #[component]
/// fn Component(attr: String, children: Children) -> Html {
///     htmx! {
///         <a href = attr>
///             {children}
///         </a>
///     }
/// }
///
/// htmx! {
///     <Component attr = "https://example.com">
///         "Some content"
///         "and some more"
///     </Component>
/// };
/// ```
#[derive(Default)]
#[must_use]
pub struct Children(Html);

impl Children {
    /// Creates empty [`Children`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds new children.
    pub fn push(&mut self, child: impl ToHtml) {
        child.write_to_html(&mut self.0);
    }
}

impl ToHtml for Children {
    fn write_to_html(&self, html: &mut Html) {
        self.0.write_to_html(html);
    }

    fn to_html(&self) -> Html {
        self.0.clone()
    }

    fn into_html(self) -> Html
    where
        Self: Sized,
    {
        self.0
    }
}

#[test]
fn children_push() {
    let mut children = Children::default();
    children.push("hello");
    children.push(["hello"]);
    children.push(vec!["hello"]);
    children.push(["hello"].as_slice());
}
