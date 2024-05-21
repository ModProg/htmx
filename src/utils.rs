use crate::native::ToScript;
use crate::{ElementState, ToHtml, WriteHtml};

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

pub struct HtmlPage<Html: ::htmx::WriteHtml, S: ::htmx::ElementState> {
    html: ::std::mem::ManuallyDrop<Html>,
    state: ::std::marker::PhantomData<S>,
    // state
}

const _: () = {
    use ::htmx::WriteHtml;
    impl<Html: ::htmx::WriteHtml> HtmlPage<Html, ::htmx::Tag> {
        pub fn new(html: Html) -> Self {
            Self {
                html: ::std::mem::ManuallyDrop::new(html),
                state: ::std::marker::PhantomData,
            }
        }

        pub fn lang(&mut self, value: impl ::htmx::attributes::ToAttribute<String>) {
            self.html.write_str(" lang");
            value.write(&mut self.html);
        }
    }
};
