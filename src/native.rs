#![allow(non_camel_case_types)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;

use derive_more::Display;
use forr::{forr, iff};
use html_escape::encode_double_quoted_attribute;

use crate::attributes::{AnyAttributeValue, FlagOrAttributeValue, IntoAttribute, ValueOrFlag};
use crate::{Html, IntoHtmlElements, ToHtml};

impl<T: ToHtml> ToHtml for &T {
    fn write_to_html(&self, out: &mut Html) {
        (*self).write_to_html(out);
    }
}

forr! {$type:ty in [&str, String, Cow<'_, str>]$*
    impl ToHtml for $type {
        fn write_to_html(&self, out: &mut Html) {
            write!(out.0, " {} ", html_escape::encode_text(&self)).unwrap()
        }
    }
}

struct ScriptContent(Cow<'static, str>);

impl ToHtml for ScriptContent {
    fn write_to_html(&self, out: &mut Html) {
        write!(out.0, " {} ", html_escape::encode_script(&self.0)).unwrap();
    }
}

macro_rules! attribute {
    ($name:ident) => {
        attribute!($name<String>);
    };
    ($name:ident=$actual:tt) => {
        attribute!($name=$actual<String>);
    };
    ($name:ident < $type:ty >) => {
        attribute!($name=(stringify!($name))<$type>);
    };
    ($name:ident=$actual:tt< $type:ty >) => {
        pub fn $name(mut self, value: impl IntoAttribute<Target = $type>) -> Self {
            self.attributes
                .insert($actual.into(), value.into_attribute());
            self
        }
    };
    ($name:ident?) => {
        pub fn $name(mut self, value: impl FlagOrAttributeValue) -> Self {
            self.attributes
                .insert(stringify!($name).into(), value.into_attribute());
            self
        }
    };
}

// Attributes that take values
forr! { ($type:ty, $attrs:tt) in [
    (a, [download?, href, hreflang, ping, referrerpolicy, rel, target, type_="type"]),
    (form, [accept_charset="accept-charset", autocomplete/*off|on*/, name, rel/*enum[]*/, action, enctype/*application/x-www-form-urlencoded, multipart/form-data, text/plain*/, method/*post|get|dialogp*/, novalidate<bool>, target/*_self|_blank|_parent|_top|...*/]),
    (button, [disabled<bool>, form, formaction, formenctype/*^^*/, formmethod/*^^*/, formnovalidate<bool>, formtarget/*^^*/, name, popovertarget, popovertargetaction/*hide|show|toggle*/, type_="type"/*submit|reset|button*/, value]),
    // TODO consider differentiating types
    (input, [accept, alt, autocomplete, capture, checked, disabled<bool>, form, formaction, formenctype/*^^*/, formmethod/*^^*/, formnovalidate<bool>, formtarget/*^^*/, height, max, maxlength, min, minlength, multiple, name, pattern, placeholder, popovertarget, popovertargetaction/*hide|show|toggle*/, readonly<bool>, required<bool>, size, src, step, type_="type"/*submit|reset|button*/, value, width]),
    (script, [async_="async"<bool>, crossorigin/*anonymous|use-credentials*/, defer<bool>, integrity, nomodule<bool>, nonce, referrerpolicy/*no-referrer|no-referrer-when-downgrade|origin|origin-when-cross-origin|same-origin|strict-origin|strict-origin-when-cross-origin|unsafe-url*/, src, type_="type"/*importmap|module|Mime*/])
] $*
    impl $type {
        forr! { $attr:ty in $attrs $*
            attribute!($attr);
        }
    }
}

forr! { $type:ty in [a, div, h1, h2, h3, h4, h5, h6, form, button, input, head, script] $*

    #[must_use]
    #[derive(Default)]
    pub struct $type {
        attributes: HashMap<Cow<'static, str>, ValueOrFlag>,
        children: Vec<Box<dyn ToHtml>>,
    }

    impl ToHtml for $type {
        fn write_to_html(&self, out: &mut Html) {
            write!(out.0, concat!("<", stringify!($type))).unwrap();
            #[cfg(feature="sorted_attributes")]
            let attributes = {
                let mut attributes: Vec<_> = self.attributes.iter().collect();
                attributes.sort_by_key(|e|e.0);
                attributes
            };
            #[cfg(not(feature="sorted_attributes"))]
            let attributes = &self.attributes;
            for (key, value) in attributes {
                // https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0
                debug_assert!(!key.chars().any(|c| c.is_whitespace()
                    || c.is_control()
                    || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')));
                match value {
                    ValueOrFlag::Value(value) => {
                        write!(out.0, " {key}=\"{}\"", encode_double_quoted_attribute(value)).unwrap();
                    }
                    ValueOrFlag::Flag => write!(out.0, " {key}").unwrap(),
                    ValueOrFlag::Unset => continue,
                }
            }
            write!(out.0, ">").unwrap();
            for child in &self.children {
                child.write_to_html(out);
            }
            write!(out.0, concat!("</", stringify!($type) , ">")).unwrap();
        }
    }

    impl $type {
        /// `builder()` is only present to be compatible with the builder
        /// instructions used for custom components, it just returns [`Self::default()`].
        pub fn builder() -> Self {
            Self::default()
        }

        /// `build()` is only present to be compatible with the builder
        /// instructions used for custom components, it is a noop.
        pub fn build(self) -> Self {
            self
        }

        iff! {!equals($type)(script) $:
            /// Adds a child component or element.
            pub fn child(mut self, child: impl IntoHtmlElements + 'static) -> Self {
                self.children.extend(
                    child
                        .into_elements()
                        .into_iter()
                        .map(|elem| Box::new(elem) as Box<dyn ToHtml>),
                );
                self
            }
        }

        iff! {equals($type)(script) $:
            /// Adds a child component or element.
            pub fn child(mut self, child: impl Into<Cow<'static, str>>) -> Self {
                self.children.push(Box::new(ScriptContent(child.into())));
                self
            }
        }

        /// Sets a `data-{key}` attribute.
        ///
        /// `key` should not start with `data-...` unless the actual html
        /// attribute should be called `data-data-...`
        pub fn data(mut self, key: impl Display, value: impl AnyAttributeValue) -> Self
        {
            self.attributes.insert(format!("data-{}", key).into(), value.into_attribute());
            self
        }

        // Global attributes
        forr! { $attr:ty in [
            // TODO ARIA: https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes
            accesskey<char>, autocapitalize/*off/none, on/sentence, words, characters*/, autofocus<bool>, class, contenteditable/*true, false, plaintext-only*/, dir/*ltr,rtl,auto*/, draggable/*true,false*/,
        ] $*
            attribute!($attr);
        }
    }
}
