use crate::attributes::ToAttribute;
use crate::{html, Html, IntoHtml, ToHtml, ToScript};

/// Embed [HTMX script](https://htmx.org/).
///
/// Can either be embedded into [`Html`] as component or be returned from
/// endpoints.
///
/// [`v1.9.5`](https://github.com/bigskysoftware/htmx/releases/tag/v1.9.5)
#[must_use]
#[derive(Clone, Copy)]
pub struct HtmxSrc;

impl HtmxSrc {
    /// HTMX source.
    pub const HTMX_SRC: &'static str = include_str!("htmx.min.js");

    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &mut Html) -> ExprHtml<Self> {
        ExprHtml(Self)
    }
}

impl ToHtml for HtmxSrc {
    fn to_html(&self, html: &mut Html) {
        crate::html! {<script>{self}</script>}.into_html(html);
    }
}

impl ToScript for HtmxSrc {
    fn to_script(&self, out: &mut Html) {
        Self::HTMX_SRC.to_script(out);
    }
}

#[must_use]
pub struct ExprHtml<T>(T);

impl<T: IntoHtml> ExprHtml<T> {
    pub fn close(self) -> impl IntoHtml {
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

#[crate::component]
pub fn HtmlPage(
    /// Sets `<meta name="viewport">` to specify page supports mobile
    /// form factor.
    mobile: bool,
    /// `<title>{}</title>`
    title: Option<&'html str>,
    /// `<link href="{}" rel="stylesheet">`
    #[default_type(std::iter::Empty<&'html str>)]
    style_sheets: impl IntoIterator<Item = impl ToAttribute<String>> + 'html,
    /// `<script src="{}">`
    #[default_type(std::iter::Empty<&'html str>)]
    scripts: impl IntoIterator<Item = impl ToAttribute<String>> + 'html,
    /// `<html lang="{lang}">`
    lang: Option<&'html str>,
    body: impl ::htmx::IntoHtml + 'html,
) {
    html!(
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
                {body}
            </body>
        </html>
    )
}
