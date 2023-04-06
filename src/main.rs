use clap::builder::NonEmptyStringValueParser;
use clap::Parser;
use color_eyre::eyre;
use tracing::{debug, debug_span, info, instrument};
use tracing_subscriber::EnvFilter;
use zbus::blocking::Connection;
use zbus::dbus_proxy;

fn main() -> eyre::Result<()> {
    let tracing_subscriber = initialize_tracing_subscriber();
    tracing::subscriber::set_global_default(tracing_subscriber)?;

    let _main = debug_span!("main").entered();
    color_eyre::install()?;

    let options = Options::parse();
    debug!(options = ?options);

    let session_path = format!("/org/freedesktop/login1/session/{}", options.session_id);

    let connection = system_bus_connection()?;
    let session = session(&connection, &session_path)?;

    subscribe_to_locked_hint_blocking(session)
}

fn initialize_tracing_subscriber() -> impl tracing::Subscriber {
    tracing_subscriber::fmt()
        // .pretty()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
}

fn system_bus_connection() -> eyre::Result<Connection> {
    let connection = Connection::system()?;

    info!(
        connection.guid = connection.inner().server_guid(),
        connection.unique_name = connection
            .inner()
            .unique_name()
            .map(|un| un.as_str())
            .unwrap_or_else(|| "No unique name")
    );
    Ok(connection)
}

fn session<'a>(
    connection: &'a Connection,
    path: &'a str,
) -> eyre::Result<SessionProxyBlocking<'a>> {
    let session: SessionProxyBlocking = SessionProxyBlocking::builder(connection)
        .path(path)?
        .build()?;
    info!(
        session.destination = session.destination().as_str(),
        session.path = session.path().as_str(),
        session.interface = session.interface().as_str()
    );
    Ok(session)
}

#[instrument(skip(session))]
fn subscribe_to_locked_hint_blocking(session: SessionProxyBlocking) -> eyre::Result<()> {
    // The first event is returned immediately with whatever the current state is,
    // so ignore it
    let _ = session.receive_locked_hint_changed().next();

    info!("Subscribing to LockedHint changes");

    while let Some(locked) = session.receive_locked_hint_changed().next() {
        if session.active()? {
            info!(locked_hint = locked.get()?)
        }
    }
    Ok(())
}

#[dbus_proxy(
    interface = "org.freedesktop.login1.Session",
    default_service = "org.freedesktop.login1"
)]
trait Session {
    /// Active property
    #[dbus_proxy(property)]
    fn active(&self) -> zbus::Result<bool>;

    /// LockedHint property
    #[dbus_proxy(property)]
    fn locked_hint(&self) -> zbus::Result<bool>;
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    /// ID of session to watch
    #[arg(required = true, env = "XDG_SESSION_ID", value_parser = NonEmptyStringValueParser::new())]
    session_id: String,
}
