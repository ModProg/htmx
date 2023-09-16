use axum_core::response::IntoResponse;

use crate::{Html, HtmxSrc};

impl IntoResponse for Html {
    fn into_response(self) -> axum_core::response::Response {
        (
            [("Content-Type", "text/html; charset=utf-8")],
            self.to_string(),
        )
            .into_response()
    }
}

impl IntoResponse for HtmxSrc {
    fn into_response(self) -> axum_core::response::Response {
        (
            [("Content-Type", "text/javascript; charset=utf-8")],
            Self::HTMX_SRC,
        )
            .into_response()
    }
}
