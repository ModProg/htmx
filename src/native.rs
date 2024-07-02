//! Native HTML elements
#![allow(non_camel_case_types, clippy::return_self_not_must_use)]

use std::fmt::Display;
use std::marker::PhantomData;

use forr::{forr, iff};

use crate::attributes::{Any, DateTime, FlagOrValue, Number, TimeDateTime, ToAttribute};
use crate::{ElementState, Html, IntoHtml, IntoScript, IntoStyle, Tag, Fragment};

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
        attribute!($elem, $name, stringify!($name), impl ToAttribute<$type>);
    };
    ($elem:ident|$name:ident=$actual:tt< $type:ty >) => {
        attribute!($elem, $name, $actual, impl ToAttribute<$type>);
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
}

macro_rules! attr_fn{
    ($($doc:expr)?, $name:ident, $actual:tt, $type:ty) => {
        $(#[doc = $doc])?
        pub fn $name(mut self, value: $type) -> Self {
            if !value.is_unset() {
                write!(self.html, " {}", $actual);
                value.write(&mut self.html);
            }
            self
        }
    }
}

// Attributes that take values
forr! { ($type:ty, $attrs:tt) in [
    (a, [download<FlagOrValue<String>>, href, hreflang, ping, referrerpolicy/*no-referrer|no-referrer-when-downgrade|origin|origin-when-cross-origin|same-origin|strict-origin|strict-origin-when-cross-origin|unsafe-url*/, rel, target/*_self|_blank|_parent|_top|...*/, type_="type"]),
    (area, [alt, coords, download<FlagOrValue<String>>, href, ping, referrerpolicy/*no-referrer|no-referrer-when-downgrade|origin|origin-when-cross-origin|same-origin|strict-origin|strict-origin-when-cross-origin|unsafe-url*/, rel, shape, target]),
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
    impl $type<'_, Tag> {
        forr! { $attr:ty in $attrs $*
            attribute!($type|$attr);
        }
    }
}

forr! { $type:ty in [a, abbr, address, area, article, aside, audio, b, base, bdi, bdo, blockquote, body, br, button, canvas, caption, cite, code, col, colgroup, data, datalist, dd, del, details, dfn, dialog, dl, dt, em, embeded, div, fieldset, figcaption, figure, footer, form, h1, h2, h3, h4, h5, h6, head, header, hgroup, hr, html, i, iframe, img, input, ins, kbd, label, legend, li, link, main, map, mark, menu, meta, meter, nav, noscript, object, ol, optgroup, option, output, p, picture, pre, progress, q, rp, rt, ruby, s, samp, script, search, section, select, slot, small, source, span, strong, style, sub, summary, sup, table, tbody, td, template, textarea, tfoot, th, thead, time, title, tr, track, u, ul, var, video, wbr, xmp] $*

    #[doc = concat!("The [`<", stringify!($type), ">`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/", stringify!($type), ") element.")]
    pub struct $type<'html, Attr: ElementState> {
        html: &'html mut Html,
        state: PhantomData<Attr>
    }

    impl $type<'_, Tag> {
        #[doc(hidden)]
        pub fn unused() {}
    }

    impl<'html> $type<'html, Tag> {

        pub fn new(html: &'html mut Html) -> Self {
            html.write_open_tag_unchecked(stringify!($type));
            Self {
                html: html,
                state: PhantomData
            }
        }


        // iff! {equals($type)(script) $:
        //     /// Adds JS code to the script.
        //     pub fn child(mut self, child: impl Into<Cow<'a, str>>) -> Self {
        //         ScriptContent(child.into()).write_to_html(&mut self.inner);
        //         self
        //     }
        // }

        // iff! {equals($type)(style) $:
        //     /// Adds CSS to the style.
        //     pub fn child(mut self, child: impl Into<Cow<'a, str>>) -> Self {
        //         ScriptContent(child.into()).write_to_html(&mut self.inner);
        //         self
        //     }
        // }

        /// Sets a custom attribute.
        ///
        /// Useful for setting, e.g., `data-{key}`.
        ///
        /// # Panics
        /// Panics on [invalid attribute names](https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0).
        pub fn custom_attr( self, key: impl Display, value: impl ToAttribute<Any>) -> Self {
            assert!(!key.to_string().chars().any(|c| c.is_whitespace()
                || c.is_control()
                || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
            self.custom_attr_unchecked(key, value)
        }

        // TODO
        // /// Sets a custom attribute.
        // ///
        // /// Useful for setting, e.g., `data-{key}`.
        // ///
        // /// # Panics
        // /// Panics on [invalid attribute names](https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0).
        // pub fn custom_attr_list(self, key: impl Display) -> $type<CustomAttr>
        // {
        //     assert!(!key.to_string().chars().any(|c| c.is_whitespace()
        //         || c.is_control()
        //         || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
        //     self.custom_attr_list_unchecked(key)
        // }


        /// Sets a custom attribute, without checking for valid keys.
        ///
        /// Useful for setting, e.g., `data-{key}`.
        ///
        /// Note: This function does contain the check for [invalid attribute names](https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0) only in debug builds, failing to ensure valid keys can lead to broken HTML output.
        pub fn custom_attr_unchecked(mut self, key: impl Display, value: impl ToAttribute<Any>) -> Self
        {
            debug_assert!(!key.to_string().chars().any(|c| c.is_whitespace()
                || c.is_control()
                || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
            write!(self.html, " {key}");
            value.write(&mut self.html);
            self
        }

        // TODO
        // /// Sets a custom attribute, without checking for valid keys.
        // ///
        // /// Useful for setting, e.g., `data-{key}`.
        // ///
        // /// Note: This function does contain the check for [invalid attribute names](https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0) only in debug builds, failing to ensure valid keys can lead to broken HTML output.
        // pub fn custom_attr_list_unchecked(mut self, key: impl Display) -> $type<T, CustomAttr>
        // {
        //     debug_assert!(!key.to_string().chars().any(|c| c.is_whitespace()
        //         || c.is_control()
        //         || matches!(c, '\0' | '"' | '\'' | '>' | '/' | '=')), "invalid key `{key}`, https://www.w3.org/TR/2011/WD-html5-20110525/syntax.html#attributes-0");
        //     write!(self.html, " {key}=\"");
        //     self.change_state()
        // }

        // TODO
        // /// Adds classes to the element.
        // pub fn class(mut self, value: impl ToAttribute<) -> $type<T, ClassesAttr> {
        //     write!(self.html, " classes=\"");
        //     self.change_state()
        // }

        // TODO
        // /// Adds styles to the element.
        // pub fn style(mut self) -> $type<T, StyleAttr> {
        //     write!(self.html, " style=\"");
        //     self.change_state()
        // }

        // Global attributes
        // TODO class should be able to specify multiple times
        forr! { $attr:ty in [
            // TODO ARIA: https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes
            class, accesskey<char>, autocapitalize/*off/none, on/sentence, words, characters*/, autofocus<bool>, contenteditable/*true, false, plaintext-only*/, dir/*ltr,rtl,auto*/, draggable/*true,false*/, enterkeyhint,hidden<FlagOrValue<String>>/*hidden|until-found*/, id, inert<bool>, inputmode/*none,text,decimal,numeric,tel,search,email,url*/, is, itemid, itemprop, itemref, itemscope, itemtype, lang, nonce, part, popover, rolle, slot, spellcheck<FlagOrValue<String>>/*true,false*/, tabindex, title, translate/*yes,no*/, virtualkeyboardpolicy/*auto,manual*/] $*
            attribute!(global|$attr);
        }
        // Event handlers
        forr! { $attr:ty in [
            onabort, onautocomplete, onautocompleteerror, onblur, oncancel, oncanplay, oncanplaythrough, onchange, onclick, onclose, oncontextmenu, oncuechange, ondblclick, ondrag, ondragend, ondragenter, ondragleave, ondragover, ondragstart, ondrop, ondurationchange, onemptied, onended, onerror, onfocus, oninput, oninvalid, onkeydown, onkeypress, onkeyup, onload, onloadeddata, onloadedmetadata, onloadstart, onmousedown, onmouseenter, onmouseleave, onmousemove, onmouseout, onmouseover, onmouseup, onmousewheel, onpause, onplay, onplaying, onprogress, onratechange, onreset, onresize, onscroll, onseeked, onseeking, onselect, onshow, onsort, onstalled, onsubmit, onsuspend, ontimeupdate, ontoggle, onvolumechange, onwaiting
        ] $*
            attribute!(event|$attr);
        }
    }

    iff! {!equals_any($type)[(area), (base), (br), (col), (embeded), (hr), (input), (link), (meta), (source), (track), (wbr)] $:

        impl <Attr: ElementState> $type<'_, Attr> {
            iff! {equals($type)(script) $:
                pub fn body(mut self, body: impl IntoScript) -> impl IntoHtml {
                    Attr::close_tag(&mut self.html);
                    body.into_script(&mut self.html);
                    self.html.write_close_tag_unchecked(stringify!($type));
                    Fragment::EMPTY
                }
            }

            iff! {equals($type)(style) $:
                pub fn body(mut self, body: impl IntoStyle) -> impl IntoHtml {
                    Attr::close_tag(&mut self.html);
                    body.into_style(&mut self.html);
                    self.html.write_close_tag_unchecked(stringify!($type));
                    Fragment::EMPTY
                }
            }

            iff! {!equals_any($type)[(style), (script)] $:
                pub fn body(mut self, body: impl IntoHtml) -> impl IntoHtml {
                    Attr::close_tag(&mut self.html);
                    body.into_html(&mut self.html);
                    self.html.write_close_tag_unchecked(stringify!($type));
                    Fragment::EMPTY
                }
            }
        }

        impl <Attr: ElementState> $type<'_, Attr> {
            pub fn close(self) -> impl IntoHtml {
                self.body(::htmx::Fragment::EMPTY)
            }
        }
    }

    iff! {equals_any($type)[(area), (base), (br), (col), (embeded), (hr), (input), (link), (meta), (source), (track), (wbr)] $:
        impl <Attr: ElementState> $type<'_, Attr> {
            pub fn close(mut self) -> impl IntoHtml {
                Attr::close_tag(&mut self.html);
                Fragment::EMPTY
            }
        }
    }


    // TODO
    // forr! {$Attr:ty in [CustomAttr, ClassesAttr, StyleAttr] $*
    //     impl $type<$Attr> {
    //         pub fn add(mut self, value: impl Display) -> Self {
    //             write!(self.html, "; {value}");
    //             self
    //         }

    //         pub fn close_attr(mut self) -> $type<T, Tag> {
    //             self.html.write_quote();
    //             self.change_state()
    //         }
    //     }
    // }
}
