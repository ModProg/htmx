use axum_core::response::IntoResponse;

use crate::{Css, Fragment, Html, HtmxSrc};

impl IntoResponse for Html {
    fn into_response(self) -> axum_core::response::Response {
        (
            [("Content-Type", "text/html; charset=utf-8")],
            self.to_string(),
        )
            .into_response()
    }
}

impl<F: FnOnce(&mut Html)> IntoResponse for Fragment<F> {
    fn into_response(self) -> axum_core::response::Response {
        (
            [("Content-Type", "text/html; charset=utf-8")],
            Html::from(self).to_string(),
        )
            .into_response()
    }
}

impl IntoResponse for Css<'static> {
    fn into_response(self) -> axum_core::response::Response {
        ([("Content-Type", "text/css; charset=utf-8")], self.0).into_response()
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
