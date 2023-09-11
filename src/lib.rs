#![warn(clippy::pedantic/* , missing_docs */)]
#![allow(clippy::wildcard_imports)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod attributes;
mod htmx_utils;
pub mod native;
use std::fmt::Write;
use std::iter;

use derive_more::Display;
pub use htmx_macros::*;
pub use htmx_utils::*;

const DOCTYPE: &str = "<!DOCTYPE html>";

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
#[must_use]
pub struct Html(String);

impl Html {
    pub fn new() -> Self {
        Self(DOCTYPE.into())
    }
}

impl ToHtml for Html {
    fn write_to_html(&self, html: &mut Html) {
        html.0.write_str(&self.0[DOCTYPE.len()..]).unwrap();
    }

    fn to_html(&self) -> Html {
        self.clone()
    }

    fn into_html(self) -> Html
    where
        Self: Sized,
    {
        self
    }
}

pub trait ToHtml {
    fn write_to_html(&self, html: &mut Html);

    fn to_html(&self) -> Html {
        let mut html = Html::default();
        self.write_to_html(&mut html);
        html
    }

    fn into_html(self) -> Html
    where
        Self: Sized,
    {
        self.to_html()
    }
}

pub trait IntoHtmlElements {
    type Element: ToHtml;
    type Elements: IntoIterator<Item = Self::Element>;

    fn into_elements(self) -> Self::Elements;
}

impl<T: IntoHtmlElements> IntoHtmlElements for Vec<T> {
    type Element = T::Element;
    type Elements = Vec<Self::Element>;

    fn into_elements(self) -> Self::Elements {
        self.into_iter()
            .flat_map(IntoHtmlElements::into_elements)
            .collect()
    }
}

impl<T: IntoHtmlElements> IntoHtmlElements for Option<T> {
    type Element = T::Element;
    type Elements = Vec<Self::Element>;

    fn into_elements(self) -> Self::Elements {
        self.into_iter()
            .flat_map(IntoHtmlElements::into_elements)
            .collect()
    }
}

impl<T: ToHtml> IntoHtmlElements for T {
    type Element = Self;
    type Elements = iter::Once<Self::Element>;

    fn into_elements(self) -> Self::Elements {
        iter::once(self)
    }
}

#[cfg(feature = "actix-web")]
mod actix {
    use std::mem;
    use std::task::Poll;

    use actix_web::body::{BoxBody, MessageBody};
    use actix_web::http::header::ContentType;
    use actix_web::web::Bytes;
    use actix_web::{HttpResponse, Responder};

    use crate::Html;

    impl Responder for Html {
        type Body = BoxBody;

        fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
            HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(self)
        }
    }

    impl MessageBody for Html {
        type Error = <String as MessageBody>::Error;

        fn size(&self) -> actix_web::body::BodySize {
            self.0.size()
        }

        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Result<actix_web::web::Bytes, Self::Error>>> {
            if self.0.is_empty() {
                Poll::Ready(None)
            } else {
                let string = mem::take(&mut self.0);
                Poll::Ready(Some(Ok(Bytes::from(string))))
            }
        }

        fn try_into_bytes(self) -> Result<Bytes, Self>
        where
            Self: Sized,
        {
            Ok(Bytes::from(self.0))
        }
    }
}
#[cfg(feature = "actix-web")]
pub use actix::*;

#[cfg(feature = "axum")]
mod axum {
    use axum_core::response::IntoResponse;

    use crate::Html;

    impl IntoResponse for Html {
        fn into_response(self) -> axum_core::response::Response {
            (
                [("Content-Type", "text/html; charset=utf-8")],
                self.to_string(),
            )
                .into_response()
        }
    }
}
#[cfg(feature = "axum")]
pub use axum::*;
