#![warn(clippy::pedantic/* TODO , missing_docs */)]
// TODO
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::wildcard_imports)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! Library for doing server side rendering of HTML using a macro.
//!
//! # `html!` macro
//!
//! The [`html!`] macro allows to write HTML inside your rust code allowing
//! to include rust values and instantiate [custom
//! components](#custom-components).
//!
//! ```
//! # use htmx::html;
//! let link = "example.com";
//! # insta::assert_display_snapshot!("doc-1",
//! html! {
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
//! # use htmx::{component, html};
//!
//! #[component]
//! fn Custom(name: String, link: bool) {
//!     html! {
//!         if link {
//!             <a href=format!("example.com/{name}")>{name}</a>
//!         } else {
//!             {name}
//!         }
//!     }
//! }
//! # insta::assert_display_snapshot!("doc-2",
//! html! {
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
//! For more documentation see [`html!`], [`component`] and the [examples](https://github.com/ModProg/htmx/tree/main/example).

// This makes `::htmx` work in the proc-macro expansions.
extern crate self as htmx;

use std::borrow::Cow;
use std::fmt;
use std::fmt::Write;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use attributes::{Any, ToAttribute};
use derive_more::{DerefMut, Display};
use forr::forr;
use html_escape::encode_double_quoted_attribute;
use serde::Serialize;

pub mod attributes;
pub mod native;
mod utils;
pub use utils::*;

#[cfg(feature = "actix-web")]
mod actix;

#[cfg(feature = "axum")]
mod axum;

#[doc(hidden)]
pub mod __private {
    pub trait Unused {
        fn unused() {}
    }
    impl<T> Unused for T {}

    pub trait Settable<T> {
        fn get_or_default(self) -> T
        where
            T: Default;
    }

    pub struct Unset;
    impl<T> Settable<T> for Unset {
        fn get_or_default(self) -> T
        where
            T: Default,
        {
            T::default()
        }
    }

    #[ghost::phantom]
    pub struct Empty<I>;
    impl<I> Iterator for Empty<I> {
        type Item = I;

        fn next(&mut self) -> Option<Self::Item> {
            None
        }
    }

    pub struct Set<T>(pub T);
    impl<T> Settable<T> for Set<T> {
        fn get_or_default(self) -> T
        where
            T: Default,
        {
            self.0
        }
    }
    impl<T: IntoIterator> IntoIterator for Set<T> {
        type IntoIter = T::IntoIter;
        type Item = T::Item;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}

/// Allows to make a component from a function.
// # Struct
// A struct needs to have an <code>[Into]<[Html]></code> implementation.
// ```
// # use htmx::{component, html, Html};
// #[component]
// struct Component {
//     a: bool,
//     b: String,
// }
//
// impl From<Component> for Html {
//     fn from(Component { a, b }: Component) -> Self {
//         html! {
//             <button disabled=a>{b}</button>
//         }
//     }
// }
//
// html! {
//     <Component a b="Disabled Button"/>
//     <Component a=true b="Disabled Button"/>
//     <Component a=false b="Enabled Button"/>
//     <Component b="Enabled Button"/>
// };
// ```
//
// In the case of struct components, all the [`#[component]`](component) macro
// does, is generating a derive for [`typed_builder::TypedBuilder`], setting
// some default attributes, like making [`bool`s](bool) optional and making the
// builder accept [`Into`].
/// # Function
/// Instead of structs function components are more succinct.
///
/// By convention function components are `PascalCase` as well, ensuring they
/// cannot conflict with native always lowercase elements.
/// ```
/// # use htmx::{component, html, Html};
/// #[component]
/// fn Component(a: bool, b: String) {
///     html! {
///         <button disabled=a>{b}</button>
///     }
/// }
///
/// html! {
///     <Component a b="Disabled Button"/>
///     <Component a=true b="Disabled Button"/>
///     <Component a=false b="Enabled Button"/>
///     <Component b="Enabled Button"/>
/// };
/// ```
/// The [`#[component]`](component) macro on functions, generates the struct and
/// [`Into`] implementation [above](#struct), making the two equivalent.
pub use htmx_macros::component;
/// The `html!` macro allows constructing [`Html`] using an HTML like syntax.
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
/// To create an element, `html!` calls `TagName::builder()`, then it calls
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
/// # use htmx::html;
/// let link = "example.com";
/// let mut chars = link.chars();
/// # insta::assert_display_snapshot!("doc-3",
/// html! {
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
/// .into_string()
/// # );
/// ```
pub use htmx_macros::html;
// TODO docs
pub use htmx_macros::rtml;

const DOCTYPE: &str = "<!DOCTYPE html>";

/// Trait used with the custom Rust like JS in `<script>` tags using the
/// [`html!`] macro.
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
///     // `html!` will prefere this function.
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

/// HTML
///
/// Can be returned from HTTP endpoints or converted to a string.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
#[must_use]
pub struct Html(String);

impl Html {
    fn write_str(&mut self, s: &str) {
        self.0.push_str(s);
    }

    fn write_char(&mut self, c: char) {
        self.0.push(c);
    }

    fn write_fmt(&mut self, a: fmt::Arguments) {
        self.0.write_fmt(a).unwrap();
    }

    fn write_quote(&mut self) {
        self.write_char('"');
    }

    fn write_gt(&mut self) {
        self.write_char('>');
    }

    fn write_open_tag_unchecked(&mut self, name: impl Display) {
        debug_assert!(name.to_string().to_ascii_lowercase().chars().all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
         "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
        );
        write!(self, "<{name}");
    }

    fn write_close_tag_unchecked(&mut self, name: impl Display) {
        debug_assert!(name.to_string().to_ascii_lowercase().chars().all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
         "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
        );
        write!(self, "</{name}>");
    }

    fn write_attr_value_unchecked(&mut self, value: impl Display) {
        write!(self, "=\"{value}\"");
    }

    fn write_attr_value_inner_unchecked(&mut self, value: impl Display) {
        write!(self, "{value}");
    }

    fn write_attr_value_encoded(&mut self, value: impl Display) {
        self.write_attr_value_unchecked(encode_double_quoted_attribute(&value.to_string()));
    }

    fn write_attr_value_inner_encoded(&mut self, value: impl Display) {
        self.write_attr_value_inner_unchecked(encode_double_quoted_attribute(&value.to_string()));
    }
}

impl Html {
    /// Creates a piece of HTML.
    pub fn new() -> Self {
        Self(DOCTYPE.into())
    }

    pub fn child_expr(mut self, child: impl ToHtml) -> Self {
        child.to_html(&mut self);
        self
    }

    pub fn child<C>(self, child: impl FnOnce(Self) -> C) -> C {
        child(self)
    }
}

impl Default for Html {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: WriteHtml + ?Sized> WriteHtml for &mut T {
    fn write_str(&mut self, s: &str) {
        T::write_str(self, s);
    }

    fn write_char(&mut self, c: char) {
        T::write_char(self, c);
    }

    fn write_fmt(&mut self, a: fmt::Arguments) {
        T::write_fmt(self, a);
    }
}

pub use htmx_macros::WriteHtml;
pub trait WriteHtml {
    fn write_str(&mut self, s: &str);

    fn write_char(&mut self, c: char);

    fn write_quote(&mut self) {
        self.write_char('"');
    }

    fn write_gt(&mut self) {
        self.write_char('>');
    }

    fn write_open_tag_unchecked(&mut self, name: impl Display) {
        debug_assert!(name.to_string().to_ascii_lowercase().chars().all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
         "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
        );
        write!(self, "<{name}");
    }

    fn write_close_tag_unchecked(&mut self, name: impl Display) {
        debug_assert!(name.to_string().to_ascii_lowercase().chars().all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
         "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
        );
        write!(self, "</{name}>");
    }

    fn write_attr_value_unchecked(&mut self, value: impl Display) {
        write!(self, "=\"{value}\"");
    }

    fn write_attr_value_inner_unchecked(&mut self, value: impl Display) {
        write!(self, "{value}");
    }

    fn write_attr_value_encoded(&mut self, value: impl Display) {
        self.write_attr_value_unchecked(encode_double_quoted_attribute(&value.to_string()));
    }

    fn write_attr_value_inner_encoded(&mut self, value: impl Display) {
        self.write_attr_value_inner_unchecked(encode_double_quoted_attribute(&value.to_string()));
    }

    fn write_fmt(&mut self, a: fmt::Arguments);
}

impl<T: WriteHtml> WriteHtml for ManuallyDrop<T> {
    fn write_str(&mut self, s: &str) {
        self.deref_mut().write_str(s);
    }

    fn write_char(&mut self, c: char) {
        self.deref_mut().write_char(c);
    }

    fn write_fmt(&mut self, a: fmt::Arguments) {
        self.deref_mut().write_fmt(a);
    }
}

/// Allows creating an element with arbitrary tag name and attributes.
///
/// This can be used for unofficial elements and web-components.
///
/// The [`html!`] macro uses them for all tags that contain `-` making it
/// possible to use web-components.
#[must_use = "call close or body"]
pub struct CustomElement<'html, S: ElementState> {
    html: &'html mut Html,
    name: Cow<'html, str>,
    state: PhantomData<S>,
}

impl<'html> CustomElement<'html, Tag> {
    /// Creates a new HTML element with the specified `name`.
    /// # Panics
    /// Panics on [invalid element names](https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname).
    /// Only the character classes are enforced, not the existence of a `-`.
    pub fn new(html: &'html mut Html, name: impl Into<Cow<'html, str>>) -> Self {
        let name = name.into();
        assert!(name.to_ascii_lowercase().chars().all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
         "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
        );
        Self::new_unchecked(html, name)
    }

    /// Creates a new HTML element with the specified `name`.
    ///
    /// Note: This function does contain the check for [invalid element names](https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname)
    /// only in debug builds, failing to ensure valid keys can lead to broken
    /// HTML output. Only the character classes are enforced, not the
    /// existence of a `-`.
    pub fn new_unchecked(html: &'html mut Html, name: impl Into<Cow<'html, str>>) -> Self {
        let name = name.into();
        debug_assert!(name.to_ascii_lowercase().chars().all(|c| matches!(c, '-' | '.' | '0'..='9' | '_' | 'a'..='z' | '\u{B7}' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')),
         "invalid tag name `{name}`, https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname"
        );
        write!(html, "<{name}");
        Self {
            html,
            name,
            state: PhantomData,
        }
    }

    /// Sets the attribute `key`, this does not do any type checking and allows
    /// [`IntoAttribute<Any>`].
    ///
    /// # Panics
    /// Panics on [invalid attribute names](https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0).
    pub fn custom_attr(self, key: impl Display, value: impl ToAttribute<Any>) -> Self {
        assert!(!key.to_string().chars().any(|c| c.is_whitespace()
            || c.is_control()
            || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
        self.custom_attr_unchecked(key, value)
    }

    /// Sets the attribute `key`, this does not do any type checking and allows
    /// [`AnyAttributeValue`], without checking for invalid characters.
    ///
    /// Note: This function does contain the check for [invalid attribute names](https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0) only in debug builds, failing to ensure valid keys can lead to broken HTML output.
    pub fn custom_attr_unchecked(self, key: impl Display, value: impl ToAttribute<Any>) -> Self {
        debug_assert!(!key.to_string().chars().any(|c| c.is_whitespace()
            || c.is_control()
            || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
        write!(self.html, " {key}");
        value.write(self.html);
        self
    }

    // TODO, use closure like body
    // pub fn custom_attr_composed(self, key: impl Display) -> CustomElement<Html,
    // CustomAttr> {     assert!(!key.to_string().chars().any(|c|
    // c.is_whitespace()         || c.is_control()
    //         || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
    //     self.custom_attr_composed_unchecked(key)
    // }

    // pub fn custom_attr_composed_unchecked(
    //     mut self,
    //     key: impl Display,
    // ) -> CustomElement<Html, CustomAttr> {
    //     debug_assert!(!key.to_string().chars().any(|c| c.is_whitespace()
    //         || c.is_control()
    //         || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
    //     write!(self.html, " {key}=\"");
    //     self.change_state()
    // }

    pub fn body(self, body: impl IntoHtml) -> impl IntoHtml {
        Tag::close_tag(self.html);
        body.into_html(self.html);
        self.html.write_close_tag_unchecked(self.name.as_ref());

        Fragment::EMPTY
    }

    pub fn close(self) -> impl IntoHtml {
        self.body(Fragment::EMPTY)
    }
}

/// Puts content directly into HTML (or CSS/JS), bypassing HTML-escaping.
///
/// ```
/// # use htmx::{html, RawSrc};
/// # insta::assert_display_snapshot!("doc-RawSrc",
/// html! {
///     "this < will be > escaped "
///     <RawSrc("This < will > not")/>
/// }
/// # );
/// ```
pub struct RawSrc<'a>(pub Cow<'a, str>);

impl<'a> RawSrc<'a> {
    /// Creates a new `RawHtml`.
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Self(content.into())
    }
}

pub struct Fragment<F>(pub F);

impl Fragment<fn(&mut Html)> {
    pub const EMPTY: Self = Self(|_| {});
}

impl<F: FnOnce(&mut Html)> From<Fragment<F>> for Html {
    fn from(val: Fragment<F>) -> Self {
        let mut html = Html::new();
        val.into_html(&mut html);
        html
    }
}

impl<F: FnOnce(&mut Html)> Fragment<F> {
    pub fn into_string(self) -> String {
        Html::from(self).0
    }

    pub fn into_html(self, html: &mut Html) {
        self.0(html);
    }
}

impl<F: Fn(&mut Html)> Display for Fragment<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut html = Html::new();
        self.0(&mut html);
        html.fmt(f)
    }
}

impl<F: FnOnce(&mut Html)> IntoHtml for Fragment<F> {
    fn into_html(self, html: &mut Html) {
        self.0(html);
    }
}
impl<F: FnOnce(&mut Html)> IntoStyle for Fragment<F> {
    fn into_style(self, html: &mut Html) {
        self.0(html);
    }
}
impl<F: FnOnce(&mut Html)> IntoScript for Fragment<F> {
    fn into_script(self, html: &mut Html) {
        self.0(html);
    }
}

pub trait IntoHtml {
    fn into_html(self, html: &mut Html);
}

impl<T: ToHtml> IntoHtml for T {
    fn into_html(self, html: &mut Html) {
        self.to_html(html);
    }
}

pub trait ToHtml {
    fn to_html(&self, html: &mut Html);
}

impl<T: ToHtml> ToHtml for &T {
    fn to_html(&self, html: &mut Html) {
        T::to_html(self, html);
    }
}

impl<T: ToHtml> ToHtml for Option<T> {
    fn to_html(&self, html: &mut Html) {
        if let Some(it) = self {
            it.to_html(html);
        }
    }
}

impl ToHtml for RawSrc<'_> {
    fn to_html(&self, html: &mut Html) {
        html.write_str(&self.0);
    }
}

impl ToScript for RawSrc<'_> {
    fn to_script(&self, html: &mut Html) {
        html.write_str(&self.0);
    }
}

impl ToStyle for RawSrc<'_> {
    fn to_style(&self, html: &mut Html) {
        html.write_str(&self.0);
    }
}

/// CSS that can both be put [`html!`] or returned from an endpoint.
pub struct Css<'a>(pub Cow<'a, str>);

impl ToHtml for Css<'_> {
    fn to_html(&self, _html: &mut Html) {
        todo!()
        // TODO: style::new(html).child(self.0.as_ref()).close();
    }
}

pub struct Tag;

impl ElementState for Tag {
    fn close_tag(html: &mut Html) {
        html.write_gt();
    }
}

forr! { $ty:ty in [CustomAttr, StyleAttr, ClassesAttr] $*
    pub struct $ty;

    impl ElementState for $ty {
        fn close_tag(html: &mut Html) {
            html.write_quote();
            html.write_gt();
        }
    }
}

pub struct Body;

impl ElementState for Body {
    fn close_tag(_: &mut Html) {}
}

pub trait ElementState {
    fn close_tag(html: &mut Html);
}

forr! {$type:ty in [&str, String, Cow<'_, str>]$*
    impl ToHtml for $type {
        fn to_html(&self, out: &mut Html) {
            write!(out, "{}", html_escape::encode_text(&self));
        }
    }

    impl ToScript for $type {
        fn to_script(&self, out: &mut Html) {
            write!(out, "{}", html_escape::encode_script(&self));
        }
    }

    impl ToStyle for $type {
        fn to_style(&self, out: &mut Html) {
            write!(out, "{}", html_escape::encode_style(&self));
        }
    }
}

impl ToHtml for char {
    fn to_html(&self, out: &mut Html) {
        write!(out, "{}", html_escape::encode_text(&self.to_string()));
    }
}

pub trait ToScript {
    fn to_script(&self, out: &mut Html);
}

impl<T: ToScript> ToScript for &T {
    fn to_script(&self, out: &mut Html) {
        T::to_script(self, out);
    }
}

pub trait IntoScript {
    fn into_script(self, html: &mut Html);
}

impl<T: ToScript> IntoScript for T {
    fn into_script(self, html: &mut Html) {
        self.to_script(html);
    }
}

pub trait ToStyle {
    fn to_style(&self, out: &mut Html);
}

impl<T: ToStyle> ToStyle for &T {
    fn to_style(&self, out: &mut Html) {
        T::to_style(self, out);
    }
}

pub trait IntoStyle {
    fn into_style(self, html: &mut Html);
}

impl<T: ToStyle> IntoStyle for T {
    fn into_style(self, html: &mut Html) {
        self.to_style(html);
    }
}
