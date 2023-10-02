use htmx_macros::{component, htmx};

use crate::{Children, Html};

/// Embed [HTMX script](https://htmx.org/).
///
/// Can either be embeded into [`Html`] as component or be returned from
/// endpoints.
///
/// [v1.9.5](https://github.com/bigskysoftware/htmx/releases/tag/v1.9.5)
#[must_use]
pub struct HtmxSrc;

impl HtmxSrc {
    /// HTMX source.
    pub const HTMX_SRC: &str = include_str!("htmx.min.js");

    // Only exist because I'm lazy
    #[doc(hidden)]
    pub fn builder() -> Self {
        Self
    }

    #[doc(hidden)]
    pub fn build(self) -> Html {
        htmx! {crate
            <script>{Self::HTMX_SRC}</script>
        }
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

/// Html boilderplate
#[component]
pub fn HtmlPage(
    /// Sets `<meta name="viewport">` to specify page supports mobile
    /// form factor.
    mobile: bool,
    /// `<title>{}</title>`
    #[component(default)]
    title: String,
    /// `<link href="{}" rel="stylesheet">`
    #[component(default)]
    AttrVec(style_sheets): AttrVec<String>,
    /// `<script src="{}">`
    #[component(default)]
    AttrVec(scripts): AttrVec<String>,
    #[component(default)]
    /// `<html lang="{lang}">`
    #[builder(setter(strip_option))]
    lang: Option<String>,
    children: Children,
) -> Html {
    htmx!(
        <html lang=lang>
            <head>
                <meta charset="utf-8"/>
                <title>{title}</title>
                if mobile {
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                }
                for style_sheet in style_sheets {
                    <link href=style_sheet rel="stylesheet"/>
                }
                for script in scripts {
                    <script src=script/>
                }
            </head>
            <body>
                {children}
            </body>
        </html>
    )
}
