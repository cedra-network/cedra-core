// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

//! Global logger definition and functions

use crate::{counters::STRUCT_LOG_COUNT, Event, Metadata};

use once_cell::sync::OnceCell;
use std::sync::Arc;
use tracing_subscriber::prelude::*;

/// The global `Logger`
static LOGGER: OnceCell<Arc<dyn Logger>> = OnceCell::new();

/// A trait encapsulating the operations required of a logger.
pub trait Logger: Sync + Send + 'static {
    /// Determines if an event with the specified metadata would be logged
    fn enabled(&self, metadata: &Metadata) -> bool;

    /// Record an event
    fn record(&self, event: &Event);

    /// Flush any buffered events
    fn flush(&self);
}

/// Record a logging event to the global `Logger`
pub(crate) fn dispatch(event: &Event) {
    if let Some(logger) = LOGGER.get() {
        STRUCT_LOG_COUNT.inc();
        logger.record(event)
    }
}

/// Check if the global `Logger` is enabled
pub(crate) fn enabled(metadata: &Metadata) -> bool {
    LOGGER
        .get()
        .map(|logger| logger.enabled(metadata))
        .unwrap_or(false)
}

/// Sets the global `Logger` exactly once
pub fn set_global_logger(logger: Arc<dyn Logger>, console_port: Option<u16>) {
    if LOGGER.set(logger).is_err() {
        eprintln!("Global logger has already been set");
    }

    /*
     * if console_port is set all tracing::log are captured by the tokio-tracing infrastructure.
     * else aptos-logger intercepts all tracing::log events
     * In both scenarios *ALL* aptos-logger::log events are captured by aptos-logger as usual.
     */
    if let Some(port) = console_port {
        let console_layer = console_subscriber::ConsoleLayer::builder()
            .server_addr(([0, 0, 0, 0], port))
            .spawn();

        tracing_subscriber::registry().with(console_layer).init();
    } else {
        let _ = tracing::subscriber::set_global_default(
            crate::tracing_adapter::TracingToAptosDataLayer
                .with_subscriber(tracing_subscriber::Registry::default()),
        );
    }
}

/// Flush the global `Logger`
pub fn flush() {
    if let Some(logger) = LOGGER.get() {
        logger.flush();
    }
}
