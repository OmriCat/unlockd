mod manager;
mod session;
mod session_id;

use crate::manager::session_path_from_id;
use crate::session::SessionInterface;
use crate::session_id::SessionId;
use clap::builder::NonEmptyStringValueParser;
use clap::Parser;
use color_eyre::eyre::{self, Context};
use duct::cmd;
use tracing::metadata::LevelFilter;
use tracing::{debug, debug_span, info};
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};
use zbus::blocking::Connection;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let tracing_subscriber = initialize_tracing_subscriber();
    tracing::subscriber::set_global_default(tracing_subscriber)
        .expect("Can't install the tracing subscriber");

    ctrlc::set_handler(|| {
        info!("Received SIGINT or SIGTERM, exiting");
        std::process::exit(0)
    })?;

    let _main = debug_span!("main").entered();

    let options = Options::parse();
    debug!(options = ?options);

    let session_id: SessionId = options.session_id.parse()?;

    let connection = Connection::system().wrap_err_with(|| "Failed to connect to system bus")?;

    let session_path = session_path_from_id(&connection, session_id)?;

    let session = SessionInterface::new(&connection, &session_path, cmd!("at-unlock"))?;

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
        .with(ErrorLayer::default())
}



#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    /// ID of session to watch
    #[arg(required = true, env = "XDG_SESSION_ID", value_parser = NonEmptyStringValueParser::new())]
    session_id: String,
}
