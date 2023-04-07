use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use tracing::{info, instrument};
use zbus::blocking::Connection;
use zbus::dbus_proxy;

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

#[derive(Debug)]
pub(crate) struct SessionInterface<'a> {
    proxy: SessionProxyBlocking<'a>,
}

impl<'a> SessionInterface<'a> {
    #[instrument(
        skip(connection),
        fields(
            connection.guid = connection.inner().server_guid(),
            connection.unique_name = connection
                .inner()
                .unique_name()
                .map(|un| un.as_str())
                .unwrap_or_else(|| "No unique name")
        )
    )]
    pub fn new(connection: &'a Connection, session_id: SessionId) -> eyre::Result<Self> {
        let session_path = format!("/org/freedesktop/login1/session/{}", session_id);
        let session: SessionProxyBlocking = SessionProxyBlocking::builder(&connection)
            .path(session_path)?
            .build()?;
        Ok(SessionInterface { proxy: session })
    }

    pub fn system_bus_connection() -> eyre::Result<Connection> {
        Connection::system().wrap_err_with(|| "Failed to connect to system bus")
    }

    #[instrument(
        skip(self),
        fields(
            session.destination = self.proxy.destination().as_str(),
            session.path = self.proxy.path().as_str(),
            session.interface = self.proxy.interface().as_str())
    )]
    pub fn blocking_subscribe_to_locked_hint(self) -> eyre::Result<()> {
        // The first event is returned immediately with whatever the current state is,
        // so ignore it
        let _ = self.proxy.receive_locked_hint_changed().next();

        info!("Subscribing to LockedHint changes");

        while let Some(locked) = self.proxy.receive_locked_hint_changed().next() {
            if self.proxy.active()? {
                info!(locked_hint = locked.get()?)
            }
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SessionId {
    session_id: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SessionIdParseError {
    NonEmptyString,
}

impl Display for SessionIdParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Session Id must be a non-empty string")
    }
}

impl Error for SessionIdParseError {}

impl FromStr for SessionId {
    type Err = SessionIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err(SessionIdParseError::NonEmptyString)
        } else {
            Ok(SessionId {
                session_id: s.to_owned(),
            })
        }
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.session_id)
    }
}
