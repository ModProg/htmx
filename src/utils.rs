use crate::{IntoHtml, ToHtml, ToScript, WriteHtml};

/// Embed [HTMX script](https://htmx.org/).
///
/// Can either be embedded into [`Html`] as component or be returned from
/// endpoints.
///
/// [`v1.9.5`](https://github.com/bigskysoftware/htmx/releases/tag/v1.9.5)
#[must_use]
pub struct HtmxSrc;

impl HtmxSrc {
    // TODO add preescaped variant
    /// HTMX source.
    pub const HTMX_SRC: &'static str = include_str!("htmx.min.js");

    #[allow(clippy::new_ret_no_self)]
    pub fn new<H: WriteHtml>(html: H) -> ExprHtml<H> {
        ExprHtml::new(Self, html)
    }
}

impl ToHtml for HtmxSrc {
    fn to_html(&self, mut html: impl crate::WriteHtml) {
        crate::html! {html =>
            <script>{self}</script>
        };
    }
}

impl ToScript for HtmxSrc {
    fn to_script(&self, out: impl WriteHtml) {
        Self::HTMX_SRC.to_script(out);
    }
}

pub struct ExprHtml<H>(H);

impl<H: WriteHtml> ExprHtml<H> {
    pub fn new(to_html: impl ToHtml, mut html: H) -> Self {
        to_html.to_html(&mut html);
        Self(html)
    }

    pub fn close(self) -> H {
        self.0
    }
}

/// Implements `From<impl IntoIterator<Item = impl Into<T>>>`.
/// This means attributes can accept more values.
#[derive(Default)]
pub struct AttrVec<T>(pub Vec<T>);

impl<T, F: Into<T>, I: IntoIterator<Item = F>> From<I> for AttrVec<T> {
    fn from(value: I) -> Self {
        Self(value.into_iter().map(Into::into).collect())
    }
}

// /// HTML boilerplate
// #[component]
// pub fn HtmlPage(
//     /// Sets `<meta name="viewport">` to specify page supports mobile
//     /// form factor.
//     mobile: bool,
//     /// `<title>{}</title>`
//     #[default]
//     title: String,
//     /// `<link href="{}" rel="stylesheet">`
//     #[default]
//     AttrVec(style_sheets): AttrVec<String>,
//     /// `<script src="{}">`
//     #[default]
//     AttrVec(scripts): AttrVec<String>,
//     #[default]
//     /// `<html lang="{lang}">`
//     #[builder(setter(strip_option))]
//     lang: Option<String>,
//     children: Children,
// ) -> Html {
//     html!(
//         <html lang=lang>
//             <head>
//                 <meta charset="utf-8"/>
//                 <title>{title}</title>
//                 if mobile {
//                     <meta name="viewport" content="width=device-width,
// initial-scale=1"/>                 }
//                 for style_sheet in style_sheets {
//                     <link href=style_sheet rel="stylesheet"/>
//                 }
//                 for script in scripts {
//                     <script src=script/>
//                 }
//             </head>
//             <body>
//                 {children}
//             </body>
//         </html>
//     )
// }

// Components need to follow the following contract for <Html: WriteHtml>:
// - new(html: Html) -> Self;
// - close(self); Closes element without body
// - body(self, impl FnOnce(Html)); Closes element with body, only required when
//   accepting children

pub struct HtmlPage<Html: ::htmx::WriteHtml, Mobile, Title, StyleSheets, Scripts, Lang> {
    html: Html,
    mobile: Mobile,
    title: Title,
    style_sheets: StyleSheets,
    scripts: Scripts,
    lang: Lang,
}

const _: () = {
    use ::htmx::WriteHtml as _;

    struct Unset;
    struct Set<T>(T);

    impl<Html: ::htmx::WriteHtml> HtmlPage<Html, Unset, Unset, Unset, Unset, Unset> {
        pub fn new(html: Html) -> Self {
            Self {
                html,
                mobile: Unset,
                title: Unset,
                style_sheets: Unset,
                scripts: Unset,
                lang: Unset,
            }
        }
    }

    impl<Html: ::htmx::WriteHtml, Title, StyleSheets, Scripts, Lang>
        HtmlPage<Html, Unset, Title, StyleSheets, Scripts, Lang>
    {
        pub fn mobile(
            self,
            mobile: bool,
        ) -> HtmlPage<Html, Set<bool>, Title, StyleSheets, Scripts, Lang> {
            let Self {
                html,
                mobile: _,
                title,
                style_sheets,
                scripts,
                lang,
            } = self;
            HtmlPage {
                html,
                mobile: Set(mobile),
                title,
                style_sheets,
                scripts,
                lang,
            }
        }
    }

    #[allow(non_camel_case_types)]
    struct mobile_was_already_set;

    impl<Html: ::htmx::WriteHtml, Mobile, Title, StyleSheets, Scripts, Lang>
        HtmlPage<Html, Set<Mobile>, Title, StyleSheets, Scripts, Lang>
    {
        #[deprecated = "mobile was already set"]
        pub fn mobile(
            self,
            mobile: bool,
            _: mobile_was_already_set,
        ) -> HtmlPage<Html, Set<bool>, Title, StyleSheets, Scripts, Lang> {
            let Self {
                html,
                mobile: _,
                title,
                style_sheets,
                scripts,
                lang,
            } = self;
            HtmlPage {
                html,
                mobile: Set(mobile),
                title,
                style_sheets,
                scripts,
                lang,
            }
        }
    }

    impl<Html: ::htmx::WriteHtml, Title, StyleSheets, Scripts, Lang>
        HtmlPage<Html, Set<bool>, Title, StyleSheets, Scripts, Lang>
    {
        pub fn body(self, body: impl ::htmx::IntoHtml<Html>) {
            let Self {
                html,
                mobile: Set(mobile),
                title,
                style_sheets,
                scripts,
                lang,
            } = self;
        }

        pub fn close(self) {
            self.body(::htmx::Fragment::EMPTY)
        }
    }
};
