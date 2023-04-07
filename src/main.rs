mod session;

use crate::session::SessionInterface;
use clap::builder::NonEmptyStringValueParser;
use clap::Parser;
use color_eyre::eyre;
use tracing::{debug, debug_span};
use tracing_subscriber::EnvFilter;

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
    tracing_subscriber::fmt()
        // .pretty()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    /// ID of session to watch
    #[arg(required = true, env = "XDG_SESSION_ID", value_parser = NonEmptyStringValueParser::new())]
    session_id: String,
}
