use htmx_macros::htmx;

use crate::Html;

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
