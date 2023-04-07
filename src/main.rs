mod session;

use crate::session::SessionInterface;
use clap::builder::NonEmptyStringValueParser;
use clap::Parser;
use color_eyre::eyre;
use tracing::metadata::LevelFilter;
use tracing::{debug, debug_span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};

fn main() -> eyre::Result<()> {
    let tracing_subscriber = initialize_tracing_subscriber();
    tracing::subscriber::set_global_default(tracing_subscriber)?;

    let _main = debug_span!("main").entered();
    color_eyre::install()?;

    let options = Options::parse();
    debug!(options = ?options);

    let connection = SessionInterface::system_bus_connection()?;
    let session = SessionInterface::new(&connection, options.session_id.parse()?)?;

    session.blocking_subscribe_to_locked_hint()
}

fn initialize_tracing_subscriber() -> impl tracing::Subscriber {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let filtered_fmt = tracing_subscriber::fmt::layer().with_filter(env_filter);
    let journald = tracing_journald::layer()
        .ok()
        .with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry()
        .with(filtered_fmt)
        .with(journald)
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    /// ID of session to watch
    #[arg(required = true, env = "XDG_SESSION_ID", value_parser = NonEmptyStringValueParser::new())]
    session_id: String,
}
