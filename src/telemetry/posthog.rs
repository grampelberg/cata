//! Posthog telemetry handler.
//!
//! This module provides a [`Handler`] implementation for sending events to
//! Posthog.
//!
//! For a detailed example, see [examples/telemetry].
//!
//! # Activity
//!
//! The name of the event in Posthog is `crate-name::activity`.
//!
//! When sending activity, the payload constructed is as follows:
//!
//! - `name`: The name of the function. This is provided by the `tracing`
//!   library.
//! - `activity`: The value of the `activity` field. It is recommended to use a
//!   common pattern here such as `parent_command::child_command`.
//! - `$screen_name`: The value of activity.
//! - `$lib`: The library name. This is always `telemetry/rust`.
//! - `level`: The level of the span/event.
//! - `module`: The module path of the span/event.
//! - `version`: The version of the crate.
//! - Any additional fields passed to tracing will be included in the payload.
//!   The Debug representation of the field is used for any of the non-standard
//!   types.
//!
//! # Error
//!
//! The name of the event in Posthog is `crate-name::error`.
//!
//! When sending errors, the only field included is `err`. This means that
//! `$screen_name` for example is not part of the payload. Instead, the
//! following are used:
//!
//! = `err`: The Debug representation of the error.
//! - `name`: This includes the full file path and line number of the error.
//! - `$lib`
//! - `level`
//! - `module`
//! - `version`
//!
//! [examples/telemetry]: ../../examples/telemetry/src/main.rs
use std::collections::HashMap;

use eyre::Result;
use tracing::field::{Field, ValueSet, Visit};

static ON_SPAN: &str = "activity";
static ON_EVENT: &str = "event";
static VERSION: &str = env!("CARGO_PKG_VERSION");
static NAME: &str = env!("CARGO_PKG_NAME");

use crate::telemetry::{Event, Handler, FIELD};

fn props(meta: &tracing_core::Metadata, visitor: &Visitor) -> HashMap<String, serde_json::Value> {
    let mut props = HashMap::new();

    props.insert(
        "name".to_string(),
        serde_json::Value::String(meta.name().into()),
    );
    props.insert(
        "$lib".to_string(),
        serde_json::Value::String("telemetry/rust".into()),
    );
    props.insert(
        "level".to_string(),
        serde_json::Value::String(meta.level().to_string().to_lowercase()),
    );
    props.insert(
        "module".to_string(),
        serde_json::Value::String(meta.module_path().unwrap().into()),
    );
    props.insert("version".to_string(), VERSION.into());

    if visitor.fields.contains_key(FIELD) {
        props.insert("$screen_name".into(), visitor.fields[FIELD].clone());
    }

    visitor.merge(&mut props);

    props
}

/// Posthog telemetry handler.
///
/// See the module documentation for usage.
#[derive(Clone, Debug)]
pub struct Posthog {
    api_key: String,
    on_span: String,
    on_event: String,
}

impl Posthog {
    /// Create a new Posthog handler using the given API key.
    pub fn new(api_key: impl AsRef<str>) -> Self {
        Self {
            api_key: api_key.as_ref().into(),
            on_span: format!("{NAME}::{ON_SPAN}"),
            on_event: format!("{NAME}::{ON_EVENT}"),
        }
    }

    /// Set the names of the events to use.
    ///
    /// By default, this is `crate-name::activity` and `crate-name::event`.
    #[must_use]
    pub fn with_names(self, on_span: impl AsRef<str>, on_event: impl AsRef<str>) -> Self {
        Self {
            on_span: on_span.as_ref().into(),
            on_event: on_event.as_ref().into(),
            ..self
        }
    }
}

impl Handler for Posthog {
    fn on_span(&self, user_id: String, meta: &tracing_core::Metadata, values: &ValueSet) -> Event {
        let mut visitor = Visitor::default();
        values.record(&mut visitor);

        Event {
            name: self.on_span.clone(),
            user_id,
            properties: props(meta, &visitor),
        }
    }

    fn on_event(&self, user_id: String, event: &tracing_core::Event) -> Event {
        let mut visitor = Visitor::default();
        event.record(&mut visitor);

        Event {
            name: self.on_event.clone(),
            user_id,
            properties: props(event.metadata(), &visitor),
        }
    }

    fn capture(&self, event: Event) -> Result<()> {
        let client = posthog_rs::client(self.api_key.as_str());

        Ok(client.capture(event.into())?)
    }
}

/// A visitor to extract all the fields on either a span or event.
#[derive(Default)]
struct Visitor {
    fields: HashMap<String, serde_json::Value>,
}

impl Visitor {
    pub fn merge(&self, props: &mut HashMap<String, serde_json::Value>) {
        props.extend(self.fields.clone());
    }
}

impl Visit for Visitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "self" {
            return;
        }

        self.fields
            .insert(field.name().into(), format!("{value:?}").into());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name().into(), value.into());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name().into(), value.into());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name().into(), value.into());
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name().into(), value.into());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.fields
            .insert(field.name().into(), value.to_string().into());
    }
}
