use std::{error::Error, io::IsTerminal};

use clap::Args;
use color_eyre::Section;
use eyre::Context;
use tracing_error::ErrorLayer;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Args)]
pub struct Instrumentation {
    /// Enable debug logs, may be specified multiple times for increased verbosity (max: 2)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

impl Instrumentation {
    pub fn setup(&self) -> eyre::Result<()> {
        let filter_layer = self.filter_layer()?;

        let registry = tracing_subscriber::registry()
            .with(filter_layer)
            .with(ErrorLayer::default());

        match self.formatter() {
            Formatter::Compact => {
                let fmt_layer = self.fmt_layer_compact();
                registry.with(fmt_layer).init();
            }
            Formatter::Full => {
                let fmt_layer = self.fmt_layer_full();
                registry.with(fmt_layer).init();
            }
        }

        Ok(())
    }

    fn fmt_layer_compact<S>(&self) -> impl tracing_subscriber::Layer<S>
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    {
        tracing_subscriber::fmt::Layer::new()
            .with_ansi(std::io::stderr().is_terminal())
            .with_writer(std::io::stderr)
            .compact()
            .without_time()
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
    }

    fn fmt_layer_full<S>(&self) -> impl tracing_subscriber::Layer<S>
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    {
        tracing_subscriber::fmt::Layer::new()
            .with_ansi(std::io::stderr().is_terminal())
            .with_writer(std::io::stderr)
    }

    fn filter_layer(&self) -> eyre::Result<EnvFilter> {
        EnvFilter::try_from_default_env().or_else(|err| {
            if let Some(source) = err.source() {
                match source.downcast_ref::<std::env::VarError>() {
                    Some(std::env::VarError::NotPresent) => (),
                    _ => {
                        return Err(err)
                            .wrap_err_with(|| {
                                format!("failed to parse directives in {}", EnvFilter::DEFAULT_ENV)
                            })
                            .suggestion("See https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives");
                    }
                }
            }

            Ok(EnvFilter::try_new(format!(
                "{}={}",
                env!("CARGO_PKG_NAME").replace('-', "_"),
                self.log_level().as_str()
            ))?)
        })
    }

    fn log_level(&self) -> tracing::Level {
        match self.verbose {
            0 => tracing::Level::INFO,
            1 => tracing::Level::DEBUG,
            2.. => tracing::Level::TRACE,
        }
    }

    fn formatter(&self) -> Formatter {
        match self.verbose {
            0..=1 => Formatter::Compact,
            2.. => Formatter::Full,
        }
    }
}

enum Formatter {
    Compact,
    Full,
}
