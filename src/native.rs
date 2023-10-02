//! Native html elements
#![allow(non_camel_case_types)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;

use derive_more::Display;
use forr::{forr, iff};
use html_escape::encode_double_quoted_attribute;

use crate::attributes::{AnyAttributeValue, FlagOrAttributeValue, IntoAttribute, ValueOrFlag};
use crate::{Html, ToHtml};

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
    ($elem:ident|$name:ident) => {
        attribute!($elem|$name<String>);
    };
    ($elem:ident|$name:ident=$actual:tt) => {
        attribute!($elem|$name=$actual<String>);
    };
    ($elem:ident|$name:ident < $type:ty >) => {
        attribute!($elem, $name, stringify!($name), impl IntoAttribute<Target = $type>);
    };
    ($elem:ident|$name:ident=$actual:tt< $type:ty >) => {
        attribute!($elem, $name, $actual, impl IntoAttribute<Target = $type>);
    };
    ($elem:ident|$name:ident?) => {
        attribute!($elem, $name, stringify!($name), impl FlagOrAttributeValue);
    };
    (global, $name:ident, $actual:expr, $type:ty) => {
        attr_fn!(concat!("Sets the [`", $actual, "`](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/", $actual, ") attribute."), $name, $actual, $type);
    };
    (event, $name:ident, $actual:expr, $type:ty) => {
        attr_fn!(concat!("Sets the `", $actual, "` [event handler](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes#event_handler_attributes) attribute."), $name, $actual, $type);
    };
    ($elem:ident, $name:ident, $actual:expr, $type:ty) => {
        attr_fn!(concat!("Sets the `", $actual, "` attribute on the [`<", stringify!($elem),">`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/", stringify!($elem), "#attributes) element."), $name, $actual, $type);
    };
    ($($doc:expr)?, $name:ident, $actual:tt, $type:ty) => {
        $(#[doc = $doc])?
        pub fn $name(mut self, value: $type) -> Self {
            self.attributes
                .insert($actual.into(), value.into_attribute());
            self
        }
    };
}

macro_rules! attr_fn{
    ($($doc:expr)?, $name:ident, $actual:tt, $type:ty) => {
        $(#[doc = $doc])?
        pub fn $name(mut self, value: $type) -> Self {
            self.attributes
                .insert($actual.into(), value.into_attribute());
            self
        }
    }
}

// Attributes that take values
forr! { ($type:ty, $attrs:tt) in [
    (a, [download?, href, hreflang, ping, referrerpolicy, rel, target, type_="type"]),
    (body, [onafterprint, onbefroeprint, onbeforeunload, onhashchange, onlanguagechange, onmessage, onoffline, ononline, onpopstate, onstorage, onundo, onunload]),
    (form, [accept_charset="accept-charset", autocomplete/*off|on*/, name, rel/*enum[]*/, action, enctype/*application/x-www-form-urlencoded, multipart/form-data, text/plain*/, method/*post|get|dialogp*/, novalidate<bool>, target/*_self|_blank|_parent|_top|...*/]),
    (button, [disabled<bool>, form, formaction, formenctype/*^^*/, formmethod/*^^*/, formnovalidate<bool>, formtarget/*^^*/, name, popovertarget, popovertargetaction/*hide|show|toggle*/, type_="type"/*submit|reset|button*/, value]),
    // TODO consider differentiating types
    (input, [accept, alt, autocomplete, capture, checked, disabled<bool>, form, formaction, formenctype/*^^*/, formmethod/*^^*/, formnovalidate<bool>, formtarget/*^^*/, height, max, maxlength, min, minlength, multiple, name, pattern, placeholder, popovertarget, popovertargetaction/*hide|show|toggle*/, readonly<bool>, required<bool>, size, src, step, type_="type"/*submit|reset|button*/, value, width]),
    (link, [as_="as", crossorigin/*anonymous, use-credentials*/, disabled, href, hreflang, imagesizes, imagesrcset, integrity, media, referrerpolicy/*no-referrer,no-referrer-when-downgrade,origin,origin-when-cross-origin,unsafe-url*/, rel, type_="type"]),
    (html, [xmlns]),
    (meta, [charset, content, http_equiv="http-equiv"/*content-security-policy,content-type,default-style,x-ua-compatible,refresh*/, name]),
    (script, [async_="async"<bool>, crossorigin/*anonymous|use-credentials*/, defer<bool>, integrity, nomodule<bool>, referrerpolicy/*no-referrer|no-referrer-when-downgrade|origin|origin-when-cross-origin|same-origin|strict-origin|strict-origin-when-cross-origin|unsafe-url*/, src, type_="type"/*importmap|module|Mime*/])
] $*
    impl $type {
        forr! { $attr:ty in $attrs $*
            attribute!($type|$attr);
        }
    }
}

forr! { $type:ty in [a, body, div, h1, h2, h3, h4, h5, h6, form, button, input, link, head, html, meta, script, title] $*

    #[doc = concat!("The [`<", stringify!($type), ">`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/", stringify!($type), ") element.")]
    #[must_use]
    #[derive(Default)]
    pub struct $type {
        attributes: HashMap<Cow<'static, str>, ValueOrFlag>,
        inner: Html,
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
            self.inner.write_to_html(out);
            iff!{!equals_any($type)[(area), (base), (br), (col), (embeded), (hr), (input), (link), (meta), (source), (track), (wbr)] $:
                write!(out.0, concat!("</", stringify!($type) , ">")).unwrap();
            }
        }
    }

    impl $type {
        /// `builder()` is only present to be compatible with the builder
        /// instructions used for custom components, it just returns [`Self::default()`].
        #[doc(hidden)]
        pub fn builder() -> Self {
            Self::default()
        }

        /// `build()` is only present to be compatible with the builder
        /// instructions used for custom components, it is a noop.
        #[doc(hidden)]
        pub fn build(self) -> Self {
            self
        }

        iff! {!(equals($type)(script) || equals_any($type)[(area), (base), (br), (col), (embeded), (hr), (input), (link), (meta), (source), (track), (wbr)]) $:
            /// Adds a child component or element.
            pub fn child(mut self, child: impl ToHtml) -> Self {
                child.write_to_html(&mut self.inner);
                self
            }
        }

        iff! {equals($type)(script) $:
            /// Adds a child component or element.
            pub fn child(mut self, child: impl Into<Cow<'static, str>>) -> Self {
                ScriptContent(child.into()).write_to_html(&mut self.inner);
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
            accesskey<char>, autocapitalize/*off/none, on/sentence, words, characters*/, autofocus<bool>, class, contenteditable/*true, false, plaintext-only*/, dir/*ltr,rtl,auto*/, draggable/*true,false*/, enterkeyhint,hidden?/*hidden|until-found*/, id, inert<bool>, inputmode/*none,text,decimal,numeric,tel,search,email,url*/, is, itemid, itemprop, itemref, itemscope, itemtype, lang, nonce, part, popover, rolle, slot, spellcheck?/*true,false*/, style, tabindex, title, translate/*yes,no*/, virtualkeyboardpolicy/*auto,manual*/] $*
            attribute!(global|$attr);
        }
        // Event handlers
        forr! { $attr:ty in [
            onabort, onautocomplete, onautocompleteerror, onblur, oncancel, oncanplay, oncanplaythrough, onchange, onclick, onclose, oncontextmenu, oncuechange, ondblclick, ondrag, ondragend, ondragenter, ondragleave, ondragover, ondragstart, ondrop, ondurationchange, onemptied, onended, onerror, onfocus, oninput, oninvalid, onkeydown, onkeypress, onkeyup, onload, onloadeddata, onloadedmetadata, onloadstart, onmousedown, onmouseenter, onmouseleave, onmousemove, onmouseout, onmouseover, onmouseup, onmousewheel, onpause, onplay, onplaying, onprogress, onratechange, onreset, onresize, onscroll, onseeked, onseeking, onselect, onshow, onsort, onstalled, onsubmit, onsuspend, ontimeupdate, ontoggle, onvolumechange, onwaiting
        ] $*
            attribute!(event|$attr);
        }
    }
}
