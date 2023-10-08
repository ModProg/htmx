#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::wildcard_imports)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! Library for doing server side rendering of HTML using a macro.
//!
//! # `htmx!` macro
//!
//! The [`htmx!`] macro allows to write HTML inside your rust code allowing
//! to include rust values and instantiate [custom
//! components](#custom-components).
//!
//! ```
//! # use htmx::htmx;
//! let link = "example.com";
//! # insta::assert_display_snapshot!("doc-1",
//! htmx! {
//!     <div>
//!         "Some literal text "
//!         // In attributes, expressions can be used directly.
//!         <a href=link>
//!             // In bodies braces are required.
//!             {link}
//!         </a>
//!         if 1 < 2 {
//!             <p>
//!                 // Whitespace must be inside `"strings"`.
//!                 <code> "if" </code> ", " <code> "for" </code>
//!                 ", and " <code> "while" </code> " can be used as well."
//!             </p>
//!         }
//!     </_> // Closing tags can be inferred.
//! }
//! # );
//! ```
//! Will result in *(with some added whitespace for readability)*.
//! ```html
//! <!DOCTYPE html>
//! <div>
//!     Some literal text <a href="example.com">example.com</a>
//!     <p> <code>if</code>, <code>for</code>, and
//!     <code>while</code> can be used as well. </p>
//! </div>
//! ```
//! <div style="border: 1pt solid currentColor; padding: .5em; margin: .5em">
//! Some literal text <a href="example.com">example.com</a>
//! <p><code>if</code>, <code>for</code>, and <code>while</code> can be used as
//! well.</p> </div>
//!
//! # Custom Components
//!
//! The most powerful feature of this crate are custom components. Using the
//! [`component`] macro, they can be created, based on structs or functions.
//! Similarly to [react](https://react.dev/) or [leptos](https://docs.rs/leptos/).
//!
//! ```
//! # use htmx::{component, htmx};
//!
//! #[component]
//! fn Custom(name: String, link: bool) {
//!     htmx! {
//!         if link {
//!             <a href=format!("example.com/{name}")>{name}</a>
//!         } else {
//!             {name}
//!         }
//!     }
//! }
//! # insta::assert_display_snapshot!("doc-2",
//! htmx! {
//!     <Custom name="link" link/>
//!     " "
//!     <Custom name="normal"/>
//! }
//! # );
//! ```
//! Will result in `<a href="example.com/link">link</a> normal`:
//! <div style="border: 1pt solid currentColor; padding: .5em; margin: .5em">
//!     <a href="example.com/link">link</a> normal
//! </div>
//!
//! For more documentation see [`htmx!`], [`component`] and the [examples](https://github.com/ModProg/htmx/tree/main/example).

// This makes `::htmx` work in the proc-macro expansions.
extern crate self as htmx;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;

use attributes::{AnyAttributeValue, ValueOrFlag};
use derive_more::Display;
use html_escape::encode_double_quoted_attribute;
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
/// The `htmx!` macro allows constructing [`Html`] using an HTML like syntax.
///
/// The native HTML elements in [`native`] are always available and do
/// not require manual imports. This also means, that custom components cannot
/// share their name with native elements. For [custom components](component)
/// therefor `PascalCase` names are recommended, but names containing
/// underscores can also be used.
///
/// Tags that are not valid rust identifiers (most importantly those containing
/// `-`, as those are used for [Web Components](https://developer.mozilla.org/en-US/docs/Web/API/Web_components))
/// will be created as [`CustomElement`]. This is also true when using blocks
/// for tag names, e.g., `<{"tagname"}>`.
///
/// To create an element, `htmx!` calls `TagName::builder()`, then it calls
/// `TagName::attribute_name(attribute_value)` for each
/// `attribute_name=attribute_value`. When the attribute name is not a valid
/// rust identifier, e.g., `attribute-name=...` or `{"string-name"}=...`, the
/// macro tries to use `TagName::custom_attr("attribute-name", ...)`.
///
/// When the attribute starts with `hx::` and is a valid path, it will be
/// translated from e.g., `hx::disabled_elt` to `hx-disabled-elt`.
/// To not accidentally mess up attributes i.e., when they are supposed to
/// contain `::` or `_`, any other paths are not modified.
///
/// These are automatically generated when using the [`component`] attribute
/// macro.
///
/// ```
/// # use htmx::htmx;
/// let link = "example.com";
/// let mut chars = link.chars();
/// # insta::assert_display_snapshot!("doc-3",
/// htmx! {
///     <div>
///         "Literal text is put directly into HTML though <html> escaping is performed."
///         " All whitespace that should be preserved needs to be inside a string literal."
///         // In attributes, expressions can be used directly.
///         <a href=link>
///             // In bodies braces are required.
///             {link}
///         </a>
///         // Tag names with `-` are not typechecked and support any attribute.
///         <web-component some_attr = "hello"/>
///         // This can be enforced by using braces as well.
///         <{"string_name"}> "Custom elements also accept children" </_>
///         // Braces can also be used to add custom attributes to elements.
///         <div {"custom-attr"}/>
///         // Control flow works as expected from rust.
///         if 1 < 2 {
///             while let Some(c) = chars.next() { {c} ", " }
///             <br/>
///             for c in link.chars() { {c} ", " }
///         }
///     </_> // Closing tags can be inferred.
/// }
/// # );
/// ```
pub use htmx_macros::htmx;
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

/// Trait used with the custom Rust like JS in `<script>` tags using the
/// [`htmx!`] macro.
///
/// It is not used per fully qualified syntax, so you are able to provide a
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

/// Allows creating an element with arbitary tag name and attributes.
///
/// This can be used for unofficial elements and web-components.
///
/// The [`htmx!`] macro uses them for all tags that contain `-` making it
/// possible to use web-components.
#[must_use]
pub struct CustomElement {
    name: Cow<'static, str>,
    attributes: HashMap<Cow<'static, str>, ValueOrFlag>,
    inner: Html,
}

impl CustomElement {
    /// Creates a new HTML element with the specified `name`.
    /// # Panics
    /// Panics on [invalid element names](https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname).
    /// Only the character classes are enforced, not the existence of a `-`.
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        assert!(name.to_ascii_lowercase().chars().all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
         "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
        );
        Self::new_unchecked(name)
    }

    /// Creates a new HTML element with the specified `name`.
    ///
    /// Note: This function does contain the check for [invalid element names](https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname)
    /// only in debug builds, failing to ensure valid keys can lead to broken
    /// HTML output. Only the character classes are enforced, not the
    /// existence of a `-`.
    pub fn new_unchecked(name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        debug_assert!(name.to_ascii_lowercase().chars().all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
         "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
        );
        Self {
            name,
            attributes: HashMap::default(),
            inner: Html::default(),
        }
    }

    /// Sets the attribute `key`, this does not do any typechecking and allows
    /// [`AnyAttributeValue`].
    ///
    /// # Panics
    /// Panics on [invalid attribute names](https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0).
    pub fn custom_attr(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl AnyAttributeValue,
    ) -> Self {
        let key = key.into();
        assert!(!key.chars().any(|c| c.is_whitespace()
            || c.is_control()
            || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
        self.attributes.insert(key, value.into_attribute());
        self
    }

    /// Sets the attribute `key`, this does not do any typechecking and allows
    /// [`AnyAttributeValue`], without checking for invalid characters.
    ///
    /// Note: This function does contain the check for [invalid attribute names](https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0) only in debug builds, failing to ensure valid keys can lead to broken HTML output.
    pub fn custom_attr_unchecked(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl AnyAttributeValue,
    ) -> Self {
        let key = key.into();
        debug_assert!(!key.chars().any(|c| c.is_whitespace()
            || c.is_control()
            || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
        self.attributes.insert(key, value.into_attribute());
        self
    }

    /// Adds child element.
    pub fn child(mut self, child: impl ToHtml) -> Self {
        child.write_to_html(&mut self.inner);
        self
    }

    #[doc(hidden)]
    pub fn build(self) -> Self {
        self
    }
}

impl ToHtml for CustomElement {
    fn write_to_html(&self, out: &mut Html) {
        write!(out.0, "<{}", self.name).unwrap();
        #[cfg(feature = "sorted_attributes")]
        let attributes = {
            let mut attributes: Vec<_> = self.attributes.iter().collect();
            attributes.sort_by_key(|e| e.0);
            attributes
        };
        #[cfg(not(feature = "sorted_attributes"))]
        let attributes = &self.attributes;
        for (key, value) in attributes {
            //
            debug_assert!(
                !key.chars().any(|c| c.is_whitespace()
                    || c.is_control()
                    || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')),
                "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0"
            );
            match value {
                ValueOrFlag::Value(value) => {
                    write!(
                        out.0,
                        " {key}=\"{}\"",
                        encode_double_quoted_attribute(value)
                    )
                    .unwrap();
                }
                ValueOrFlag::Flag => write!(out.0, " {key}").unwrap(),
                ValueOrFlag::Unset => continue,
            }
        }
        write!(out.0, ">").unwrap();
        self.inner.write_to_html(out);
        write!(out.0, "</{}>", self.name).unwrap();
    }
}
