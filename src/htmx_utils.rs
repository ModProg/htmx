use htmx_macros::htmx;

use crate::Html;

#[must_use]
pub struct HtmxSrc;

impl HtmxSrc {
    pub fn builder() -> Self {
        Self
    }

    pub fn build(self) -> Html {
        htmx! {crate
            <script>{include_str!("htmx.min.js")}</script>
        }
    }
}
