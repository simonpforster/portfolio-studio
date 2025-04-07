use axum::extract::Request;
use axum::http::{HeaderMap, HeaderName};
use axum::middleware::Next;
use axum::response::Response;
use opentelemetry::propagation::{Extractor, Injector};
use std::convert::Infallible;
use tracing::{instrument, Instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[instrument]
pub async fn extract_context(req: Request, next: Next) -> Result<Response, Infallible> {
    let parent_context: opentelemetry::Context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&AxumHeaderExtractor(req.headers()))
    });

    Span::current().set_parent(parent_context);

    Ok(next.run(req).in_current_span().await)
}

pub struct AxumHeaderExtractor<'a>(pub &'a HeaderMap);

impl<'a> Extractor for AxumHeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key)
            .and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys()
            .map(|k| k.as_str())
            .collect()
    }
}