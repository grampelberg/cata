//! Integrate activity and error reporting with tracing
//!
//! This module provides a [`Telemetry`] layer for [`tracing`] that can capture
//! and report activity or errors to a backend. The backend is abstracted by the
//! [`Handler`] trait and already implemented for [`posthog`].
//!
//! Some things to note:
//! - By default, the layer ignores all events and spans. To opt-in to
//!   reporting, call `with_activity` or `with_errors` to enable.
//! - IDs are stable for a single machine and rely on [`machine_uid`]. These are
//!   hashed before being sent over the network.
//! - What is actually reported is up to the implementation of the `Handler`.
//!   Check the documentation to see what is reported.
//!
//! For a complete example, see [examples/telemetry].
//!
//! # Monitoring a function
//!
//! To see errors and usage for a function, you can do:
//!
//! ```
//! #[tracing::instrument(err, fields(activity = "my_function"))]
//! fn foo() -> eyre::Result<()> {
//!   Ok(())
//! }
//! ```
//!
//! Passing `err` to instrument causes the generated code to capture errors and
//! issue events to tracing. This ends up calling the `on_event` method of the
//! configured `Handler`. Note that the error event issued is a little different
//! than a normal event and will be missing the fields.
//!
//! By setting `activity = "my_function"`, the `on_span` method of the `Handler`
//! will be called. Any tracing activity that includes the `activity` field will
//! be reported, for example, it could be used with one of the macros:
//! `info!(activity = "my_function", "stuff happened")`.
//!
//! # Global Setup
//!
//! When adding the layer, make sure to be using per-layer filtering instead of
//! global. This will ensure that any user specific configurations don't get in
//! the way of reporting, such as log level. To do this with
//! `tracing_subscriber::fmt`, you can do:
//!
//! ```
//! use cata::telemetry::{posthog, Telemetry};
//! use tracing_subscriber::prelude::*;
//!
//! let format_layer = tracing_subscriber::fmt::layer()
//!   .with_filter(tracing_subscriber::EnvFilter::from_default_env());
//!
//! let telemetry = Telemetry::new(posthog::Posthog::new("api-key"))
//!   .with_activity()
//!   .with_errors();
//!
//! tracing_subscriber::registry()
//!   .with(format_layer)
//!   .with(telemetry)
//!   .init();
//! ```
//!
//! # Building
//!
//! It is recommended that you use `build.rs` to pull the API keys used for
//! handlers into the process without checking them into source. To do this with
//! Posthog, you can add it in `build.rs`:
//!
//! ```
//! static PH_VAR: &str = "POSTHOG_API_KEY";
//!
//! fn main() {
//!   if let Some(key) = std::env::var_os(PH_VAR) {
//!     println!("cargo:rustc-env={}={}", PH_VAR, key.to_string_lossy());
//!   }
//! }
//! ```
//!
//! This can be consumed wherever you're setting up the handler:
//!
//! ```
//! use cata::telemetry::posthog::Posthog;
//!
//! static PH_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");
//!
//! fn main() {
//!   let ph = Posthog::new(PH_KEY.unwrap_or("api-key"));
//! }
//! ```
//!
//! # Backends
//!
//! - [`posthog`]: A simple backend that sends events to Posthog.
//!
//! To implement your own backend, you need to implement the [`Handler`] trait.
//! It has two functions which construct events (`on_span` and `on_event`) and a
//! `capture` function to publish the event.
//!
//! Because tracing requires that layer handlers are synchronous, the `capture`
//! function is called via. tokio's `spawn_blocking`. This guarantees that any
//! telemetry completes before the program exits and does not block normal
//! program flow.
//!
//! [examples/telemetry]: https://github.com/grampelberg/cata/blob/main/examples/telemetry/src/main.rs
pub mod posthog;

use std::collections::HashMap;

use eyre::Result;
use tracing::{error, field::ValueSet, Subscriber};
use tracing_subscriber::{layer::Layer, registry::LookupSpan};

static NAME: &str = env!("CARGO_PKG_NAME");
static FIELD: &str = "activity";

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

/// A tracing layer that captures events and spans and sends them to a backend.
///
/// This layer is designed to be used with [`tracing_subscriber::registry`] and
/// allows pluggable backends via the [`Handler`] trait.
///
/// By default, it ignores all events and spans. To opt-in to reporting, call
/// `with_activity` or `with_errors` to enable.
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
    /// Create a new telemetry layer with the given handler.
    pub fn new(handler: H) -> Telemetry<H> {
        Self {
            user_id: uuid(),
            provider: handler,
            emit_activity: false,
            emit_errors: false,
        }
    }

    /// Enable capturing activity spans and events.
    #[must_use]
    pub fn with_activity(mut self) -> Self {
        self.emit_activity = true;
        self
    }

    /// Enable capturing error events.
    #[must_use]
    pub fn with_errors(mut self) -> Self {
        self.emit_errors = true;
        self
    }

    /// Check if the layer is interested in the metadata.
    ///
    /// Opt to test on a per-event basis instead of using the extensive
    /// `tracing_subscriber::filter` functionality. This is primarily because
    /// the `Filtered<>` type ends up being overly complex to use and doesn't
    /// support our use case of disabling everything by default.
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

        // Tracing layers must be synchronous. `spawn` allows for the capture event to
        // be sync or async depending on what it wants to do. This is `spawn_blocking`
        // specifically to ensure that the reporting happens before shutdown. Tokio does
        // not guarantee to `join()` handles thrown away which are async on shutdown. It
        // instead waits for a yield to do shutdown. By using blocking, the runtime will
        // continue running until the event is successfully reported.
        //
        // Note: it is possible to send events *after* the runtime has shutdown. If it
        // has, just spawn a new one and send the event.
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

/// An event constructed by the handler.
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

/// A handler for telemetry events.
///
/// This trait is used to capture and report telemetry events to a backend. It
/// allows for the difference in types between spans and events.
pub trait Handler: Clone + Send + Sync {
    /// Construct an event from a span.
    ///
    /// This is called `on_new_span`. It is only used for activity and filtered
    /// with the `activity` field.
    fn on_span(&self, user_id: String, meta: &tracing_core::Metadata, values: &ValueSet) -> Event;

    /// Construct a [`Event`] from a [`tracing::Event`].
    ///
    /// This is called `on_event`. It is either called for errors when
    /// `#[instrument(err)]` is used *or* when a macro such as `info!(activity =
    /// "my_function", "stuff happened")` is used. It needs to be able to
    /// support both use cases. Note that events must either contain the
    /// `activity` field or the `err` field to reach this call.
    fn on_event(&self, user_id: String, event: &tracing_core::Event) -> Event;

    /// Capture the event.
    fn capture(&self, event: Event) -> Result<()>;
}
