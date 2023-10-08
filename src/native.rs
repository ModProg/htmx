//! Native html elements
#![allow(non_camel_case_types)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;

use forr::{forr, iff};
use html_escape::encode_double_quoted_attribute;

use crate::attributes::{
    AnyAttributeValue, DateTime, FlagOrAttributeValue, IntoAttribute, Number, TimeDateTime,
    ValueOrFlag,
};
use crate::{Html, ToHtml};

forr! {$type:ty in [&str, String, Cow<'_, str>]$*
    impl ToHtml for $type {
        fn write_to_html(&self, out: &mut Html) {
            write!(out.0, "{}", html_escape::encode_text(&self)).unwrap();
        }
    }
}

impl ToHtml for char {
    fn write_to_html(&self, out: &mut Html) {
        write!(out.0, "{}", html_escape::encode_text(&self.to_string())).unwrap();
    }
}

struct ScriptContent(Cow<'static, str>);

impl ToHtml for ScriptContent {
    fn write_to_html(&self, out: &mut Html) {
        write!(out.0, " {} ", html_escape::encode_script(&self.0)).unwrap();
    }
}

struct StyleContent(Cow<'static, str>);

impl ToHtml for StyleContent {
    fn write_to_html(&self, out: &mut Html) {
        write!(out.0, " {} ", html_escape::encode_style(&self.0)).unwrap();
    }
}

macro_rules! attribute {
    ($elem:ident|$name:ident<FlagOrAttributeValue>) => {
        attribute!($elem, $name, stringify!($name), impl FlagOrAttributeValue);
    };
    ($elem:ident|$name:ident<TimeDateTime>) => {
        attribute!($elem, $name, stringify!($name), impl TimeDateTime);
    };
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
    (a, [download<FlagOrAttributeValue>, href, hreflang, ping, referrerpolicy/*no-referrer|no-referrer-when-downgrade|origin|origin-when-cross-origin|same-origin|strict-origin|strict-origin-when-cross-origin|unsafe-url*/, rel, target/*_self|_blank|_parent|_top|...*/, type_="type"]),
    (area, [alt, coords, download<FlagOrAttributeValue>, href, ping, referrerpolicy/*no-referrer|no-referrer-when-downgrade|origin|origin-when-cross-origin|same-origin|strict-origin|strict-origin-when-cross-origin|unsafe-url*/, rel, shape, target]),
    (audio, [autoplay<bool>, controls<bool>, crossorigin/*anonymous, use-credentials*/, loop_="loop", muted<bool>, preload/*none,metadata,auto*/, src]),
    (base, [href, target/*_self|_blank|_parent|_top|...*/]),
    (blockquote, [cite]),
    (body, [onafterprint, onbefroeprint, onbeforeunload, onhashchange, onlanguagechange, onmessage, onoffline, ononline, onpopstate, onstorage, onundo, onunload]),
    (form, [accept_charset="accept-charset", autocomplete/*off|on*/, name, rel/*enum[]*/, action, enctype/*application/x-www-form-urlencoded, multipart/form-data, text/plain*/, method/*post|get|dialogp*/, novalidate<bool>, target/*_self|_blank|_parent|_top|...*/]),
    (button, [disabled<bool>, form, formaction, formenctype/*^^*/, formmethod/*^^*/, formnovalidate<bool>, formtarget/*^^*/, name, popovertarget, popovertargetaction/*hide|show|toggle*/, type_="type"/*submit|reset|button*/, value]),
    (canvas, [height<Number>, width<Number>]),
    (col, [span<Number>]),
    (colgroup, [span<Number>]),
    (data, [value]),
    (del, [cite, datetime<DateTime>]),
    (details, [open<bool>]),
    (dialog, [open<bool>]),
    (embeded, [height<Number>, src, type_="type", width<Number>]),
    (fieldset, [disabled<bool>, form, name]),
    (html, [xmlns]),
    (iframe, [allow, height<Number>, loading/*eager, lazy*/, name, referrerpolicy/*no-referrer|no-referrer-when-downgrade|origin|origin-when-cross-origin|same-origin|strict-origin|strict-origin-when-cross-origin|unsafe-url*/, sandbox/*allow-downloads,allow-forms,allow-modals,allow-orientation-lock,allow-pointer-lock,allow-popups,allow-popups-to-escape-sandbox,allow-presentation,allow-same-origin,allow-scripts,allow-top-navigation,allow-top-navigation-by-user-activation,allow-top-navigation-to-custom-protocols*/, src, srcdoc, width<Number>]),
    (img, [crossorigin/*anonymous, use-credentials*/, decoding/*sync,async,auto*/,elementtiming,height<Number>,ismap<bool>, loading/*eager, lazy*/, referrerpolicy/*no-referrer|no-referrer-when-downgrade|origin|origin-when-cross-origin|same-origin|strict-origin|strict-origin-when-cross-origin|unsafe-url*/, sizes, src, srcset, width, usemap]),
    // TODO consider differentiating types
    (input, [accept, alt, autocomplete, capture, checked, disabled<bool>, form, formaction, formenctype/*^^*/, formmethod/*^^*/, formnovalidate<bool>, formtarget/*^^*/, height, max, maxlength, min, minlength, multiple, name, pattern, placeholder, popovertarget, popovertargetaction/*hide|show|toggle*/, readonly<bool>, required<bool>, size, src, step, type_="type"/*submit|reset|button*/, value, width]),
    (ins, [cite, datetime<DateTime>]),
    (label, [for_="for"]),
    (li, [value]),
    (link, [as_="as", crossorigin/*anonymous, use-credentials*/, disabled, href, hreflang, imagesizes, imagesrcset, integrity, media, referrerpolicy/*no-referrer,no-referrer-when-downgrade,origin,origin-when-cross-origin,unsafe-url*/, rel, type_="type"]),
    (map, [name]),
    (meta, [charset, content, http_equiv="http-equiv"/*content-security-policy,content-type,default-style,x-ua-compatible,refresh*/, name]),
    (meter, [value<Number>, min<Number>, max<Number>, low<Number>, high<Number>, optimum<Number>, form]),
    (object, [data, form, height<Number>, name, type_="type", usemap, width<Number>]),
    (ol, [reversed<bool>, start<Number>, type_="type"/*a,A,i,I,1*/]),
    (optgroup, [disabled<bool>, label]),
    (option, [disabled<bool>, label, selected, value]),
    (output, [for_="for", form, name]),
    (progress, [max<Number>, value<Number>]),
    (q, [cite]),
    (script, [async_="async"<bool>, crossorigin/*anonymous|use-credentials*/, defer<bool>, integrity, nomodule<bool>, referrerpolicy/*no-referrer|no-referrer-when-downgrade|origin|origin-when-cross-origin|same-origin|strict-origin|strict-origin-when-cross-origin|unsafe-url*/, src, type_="type"/*importmap|module|Mime*/]),
    (select, [ autocomplete, disabled<bool>, form, name, required<bool>, size]),
    (slot, [name]),
    (source, [type_="type", src, srcset, sizes, media, height<Number>, width<Number>]),
    (style, [media]),
    (td, [colspan<Number>, headers, rowspan<Number>]),
    (textarea, [autocomplete, autocorrect/*on,off*/, cols<Number>, dirname, disabled<bool>, form, maxlength, minlength, name, placeholder, readonly<bool>, required<bool>, rows, wrap/*hard,soft,off*/]),
    (th, [colspan<Number>, headers, rowspan<Number>, scope/*row,col,rowgroup,colgroup*/]),
    (time, [datetime<TimeDateTime>]),
    (track, [default<bool>, kind/*subtitles,captions,descriptions,chapters,metadata*/, label, src, srclang]),
    (video, [autoplay<bool>, controls<bool>, crossorigin/*anonymous, use-credentials*/, height<Number>, loop_="loop"<bool>, muted<bool>, playsinline<bool>, poster, preload/*none,metadata,auto*/, src, width<Number>])
] $*
    impl $type {
        forr! { $attr:ty in $attrs $*
            attribute!($type|$attr);
        }
    }
}

forr! { $type:ty in [a, abbr, address, area, article, aside, audio, b, base, bdi, bdo, blockquote, body, br, button, canvas, caption, cite, code, col, colgroup, data, datalist, dd, del, details, dfn, dialog, dl, dt, em, embeded, div, fieldset, figcaption, figure, footer, form, h1, h2, h3, h4, h5, h6, head, header, hgroup, hr, html, i, iframe, img, input, ins, kbd, label, legend, li, link, main, map, mark, menu, meta, meter, nav, noscript, object, ol, optgroup, option, output, p, picture, pre, progress, q, rp, rt, ruby, s, samp, script, search, section, select, slot, small, source, span, strong, style, sub, summary, sup, table, tbody, td, template, textarea, tfoot, th, thead, time, title, tr, track, u, ul, var, video, wbr, xmp] $*

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
                debug_assert!(!key.chars().any(|c| c.is_whitespace()
                    || c.is_control()
                    || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
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
                write!(out.0, concat!("</", stringify!($type), ">")).unwrap();
            }
        }
    }

    impl $type {
        /// `builder()` is only present to be compatible with the builder
        /// instructions used for custom components, it just returns [`Self::default()`].
        #[doc(hidden)]
        pub fn builder() -> Self {
            Default::default()
        }

        /// `build()` is only present to be compatible with the builder
        /// instructions used for custom components, it is a noop.
        #[doc(hidden)]
        pub fn build(self) -> Self {
            self
        }

        iff! {!equals_any($type)[(script), (style), (area), (base), (br), (col), (embeded), (hr), (input), (link), (meta), (source), (track), (wbr)] $:
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

        iff! {equals($type)(style) $:
            /// Adds a child component or element.
            pub fn child(mut self, child: impl Into<Cow<'static, str>>) -> Self {
                ScriptContent(child.into()).write_to_html(&mut self.inner);
                self
            }
        }

        /// Sets a custom attribute.
        ///
        /// Useful for setting, e.g., `data-{key}`.
        ///
        /// # Panics
        /// Panics on [invalid attribute names](https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0).
        pub fn custom_attr(mut self, key: impl Into<Cow<'static, str>>, value: impl AnyAttributeValue) -> Self
        {
            let key = key.into();
        assert!(!key.chars().any(|c| c.is_whitespace()
            || c.is_control()
            || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
            self.attributes.insert(key.to_string().into(), value.into_attribute());
            self
        }

        /// Sets a custom attribute, without checking for valid keys.
        ///
        /// Useful for setting, e.g., `data-{key}`.
        ///
        /// Note: This function does contain the check for [invalid attribute names](https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0) only in debug builds, failing to ensure valid keys can lead to broken HTML output.
        pub fn custom_attr_unchecked(mut self, key: impl Into<Cow<'static, str>>, value: impl AnyAttributeValue) -> Self
        {
            let key = key.into();
        assert!(!key.chars().any(|c| c.is_whitespace()
            || c.is_control()
            || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
            self.attributes.insert(key.to_string().into(), value.into_attribute());
            self
        }


        // Global attributes
        forr! { $attr:ty in [
            // TODO ARIA: https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes
            accesskey<char>, autocapitalize/*off/none, on/sentence, words, characters*/, autofocus<bool>, class, contenteditable/*true, false, plaintext-only*/, dir/*ltr,rtl,auto*/, draggable/*true,false*/, enterkeyhint,hidden<FlagOrAttributeValue>/*hidden|until-found*/, id, inert<bool>, inputmode/*none,text,decimal,numeric,tel,search,email,url*/, is, itemid, itemprop, itemref, itemscope, itemtype, lang, nonce, part, popover, rolle, slot, spellcheck<FlagOrAttributeValue>/*true,false*/, style, tabindex, title, translate/*yes,no*/, virtualkeyboardpolicy/*auto,manual*/] $*
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
