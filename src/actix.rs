use std::mem;
use std::pin::Pin;
use std::task::Poll;

use actix_web::body::{BoxBody, MessageBody};
use actix_web::http::header::ContentType;
use actix_web::web::Bytes;
use actix_web::{HttpResponse, Responder};

use crate::{Css, Html, HtmxSrc, Fragment};

impl Responder for Html {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(self)
    }
}

impl<F: FnOnce(&mut Html)> Responder for Fragment<F> {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(Html::from(self))
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

impl Responder for HtmxSrc {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok()
            .content_type("text/javascript; charset=utf-8")
            .body(Self::HTMX_SRC)
    }
}

impl Responder for Css<'static> {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok()
            .content_type("text/css; charset=utf-8")
            .body(self.0)
    }
}

impl MessageBody for Css<'static> {
    type Error = <String as MessageBody>::Error;

    fn size(&self) -> actix_web::body::BodySize {
        self.0.size()
    }

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<actix_web::web::Bytes, Self::Error>>> {
        Pin::new(&mut self.0).poll_next(cx)
    }

    fn try_into_bytes(self) -> Result<Bytes, Self>
    where
        Self: Sized,
    {
        Ok(Bytes::from(self.0.to_string()))
    }
}
