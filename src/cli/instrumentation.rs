use clap::Args;

#[derive(Debug, Args)]
pub struct Instrumentation {
    /// Enable debug logs, may be specified multiple times for increased verbosity (max: 2)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

impl Instrumentation {
    pub fn setup(&self) {
        tracing_subscriber::fmt()
            .with_max_level(self.log_level())
            .compact()
            .init();
    }

    fn log_level(&self) -> tracing::Level {
        match self.verbose {
            0 => tracing::Level::INFO,
            1 => tracing::Level::DEBUG,
            2.. => tracing::Level::TRACE,
        }
    }
}
