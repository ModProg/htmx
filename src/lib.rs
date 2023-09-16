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
//!
//! ```
//!
//! # Custom Components
//!
//! For more documentation see the individual items and the [examples](https://github.com/ModProg/htmx/tree/main/example).
use std::fmt::Write;

use derive_more::Display;
use serde::Serialize;

pub mod attributes;
mod htmx_utils;
pub mod native;

pub use htmx_macros::*;
pub use htmx_utils::*;

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

impl<T: ToHtml> ToHtml for &T {
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
