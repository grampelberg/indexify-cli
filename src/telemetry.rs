pub mod posthog;

use std::collections::HashMap;

use eyre::Result;
use tracing::{error, field::ValueSet, Subscriber};
use tracing_subscriber::{layer::Layer, registry::LookupSpan};

static NAME: &str = env!("CARGO_PKG_NAME");
pub static FIELD: &str = "activity";

fn uuid() -> String {
    let mid = machine_uid::get().unwrap_or_else(|_| "unknown".to_string());
    let tag = ring::hmac::sign(
        &ring::hmac::Key::new(ring::hmac::HMAC_SHA256, NAME.as_bytes()),
        mid.as_bytes(),
    );

    uuid::Builder::from_bytes(tag.as_ref()[..16].try_into().unwrap())
        .into_uuid()
        .hyphenated()
        .to_string()
}

#[derive(Clone, Debug)]
pub struct Telemetry<H>
where
    H: Handler + 'static,
    Self: 'static,
{
    provider: H,
    user_id: String,
    emit_activity: bool,
    emit_errors: bool,
}

impl<H> Telemetry<H>
where
    H: Handler,
{
    pub fn new(handler: H) -> Telemetry<H> {
        Self {
            user_id: uuid(),
            provider: handler,
            emit_activity: true,
            emit_errors: true,
        }
    }

    pub fn with_activity(mut self) -> Self {
        self.emit_activity = true;
        self
    }

    pub fn with_errors(mut self) -> Self {
        self.emit_errors = true;
        self
    }

    fn interested(&self, metadata: &tracing_core::Metadata<'_>) -> bool {
        (self.emit_activity && metadata.fields().field(FIELD).is_some())
            || (self.emit_errors && metadata.fields().field("error").is_some())
    }

    fn capture(&self, event: Event) {
        let provider = self.provider.clone();

        let handler = move || {
            if let Err(e) = provider.capture(event) {
                error!("Failed to capture: {:?}", e);
            }
        };

        if let Ok(current) = tokio::runtime::Handle::try_current() {
            current.spawn_blocking(handler);
        } else {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed building the Runtime")
                .spawn_blocking(handler);
        };
    }
}

impl<S, H> Layer<S> for Telemetry<H>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
    H: Handler + 'static,
{
    fn on_new_span(
        &self,
        attrs: &tracing_core::span::Attributes<'_>,
        _: &tracing_core::span::Id,
        _: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if !self.interested(attrs.metadata()) {
            return;
        }

        self.capture(
            self.provider
                .on_span(self.user_id.clone(), attrs.metadata(), attrs.values()),
        );
    }

    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        if !self.interested(event.metadata()) {
            return;
        }

        self.capture(self.provider.on_event(self.user_id.clone(), event));
    }
}

#[derive(Debug)]
pub struct Event {
    name: String,
    user_id: String,
    properties: HashMap<String, serde_json::Value>,
}

impl From<Event> for posthog_rs::Event {
    fn from(ev: Event) -> Self {
        let mut ph = posthog_rs::Event::new(ev.name, ev.user_id);
        for (k, v) in ev.properties {
            ph.insert_prop(k, v).expect("need to add prop");
        }

        ph
    }
}

pub trait Handler: Clone + Send + Sync {
    fn on_span(&self, user_id: String, meta: &tracing_core::Metadata, values: &ValueSet) -> Event;
    fn on_event(&self, user_id: String, event: &tracing_core::Event) -> Event;
    fn capture(&self, event: Event) -> Result<()>;
}
