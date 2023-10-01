use htmx_macros::{component, htmx};

use crate as htmx;
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

#[component]
pub fn HtmlPage(
    /// Sets `<meta name="viewport">` to specify page supports mobile
    /// form factor.
    mobile: bool,
    #[component(default)] title: String,
    #[component(default)] style_sheets: Vec<String>,
    #[component(default)] scripts: Vec<String>,
    lang: Option<String>,
    children: Children,
) -> Html {
    htmx!(
        <html lang=lang>
            <meta charset="utf-8"/>
            <title>{title}</title>
            <meta name="viewport" content="width=device-width, initial-scale=1"/>
            // <for style_sheet in style_sheets + 2 >
            //     <link href = style_sheet rel = "stylesheet" />
            // </for>
        </html>
    )
}
