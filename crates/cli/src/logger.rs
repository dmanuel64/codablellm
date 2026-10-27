use std::{
    fmt::{self, Write},
    io,
};

use cliclack::log;
use tracing::{
    Event, Level, Subscriber,
    field::{Field, Visit},
    level_filters::LevelFilter,
};
use tracing_subscriber::{Layer, layer::Context, prelude::*};

use crate::cli::INTERACTIVE;

pub struct CliclackLayer;

#[derive(Default)]
struct Visitor {
    message: String,
    fields: String,
}

impl Visit for Visitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            let _ = write!(self.fields, " {}={value:?}", field.name());
        }
    }
}

impl<S: Subscriber> Layer<S> for CliclackLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut v = Visitor::default();
        event.record(&mut v);
        let text = format!("{}{}", v.message, v.fields);
        let _ = match *event.metadata().level() {
            Level::ERROR => log::error(text),
            Level::WARN => log::warning(text),
            Level::INFO => log::info(text),
            _ => log::remark(text),
        };
    }
}

pub(crate) fn init(filter: LevelFilter) {
    let interactive = !*INTERACTIVE;
    tracing_subscriber::registry()
        .with(filter)
        .with(interactive.then_some(CliclackLayer))
        .with((!interactive).then(|| tracing_subscriber::fmt::layer().with_writer(io::stderr)))
        .init();
}
