use color_eyre::eyre;
use color_eyre::eyre::{eyre, WrapErr};
use duct::{Expression, Handle};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use tracing::{debug, error, info, instrument, warn};
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
    unlock_cmd: Expression,
}

impl<'a> SessionInterface<'a> {
    #[instrument(skip(unlock_cmd))]
    pub fn new<T: Into<Expression>>(session_id: SessionId, unlock_cmd: T) -> eyre::Result<Self> {
        let session_path = format!("/org/freedesktop/login1/session/{}", session_id);
        let connection =
            Connection::system().wrap_err_with(|| "Failed to connect to system bus")?;
        debug!(
            connection.guid = connection.inner().server_guid(),
            connection.unique_name = connection
                .inner()
                .unique_name()
                .map(|un| un.as_str())
                .unwrap_or_else(|| "No unique name"),
        );
        let session: SessionProxyBlocking = SessionProxyBlocking::builder(&connection)
            .path(session_path)?
            .build()?;
        Ok(SessionInterface {
            proxy: session,
            unlock_cmd: unlock_cmd.into().unchecked(),
        })
    }

    #[instrument(
        skip(self),
        fields(
            connection.guid = self.proxy.connection().inner().server_guid(),
            connection.unique_name = self.proxy.connection()
                .inner()
                .unique_name()
                .map(|un| un.as_str())
                .unwrap_or_else(|| "No unique name"),
            session.destination = self.proxy.destination().as_str(),
            session.path = self.proxy.path().as_str(),
            session.interface = self.proxy.interface().as_str())
    )]
    pub fn blocking_subscribe_to_locked_hint(self) -> eyre::Result<()> {
        // The first event is returned immediately with whatever the current state is,
        // so ignore it
        let _ = self.proxy.receive_locked_hint_changed().next();

        info!("Subscribing to LockedHint changes");

        let mut handle: Option<Handle> = None;

        while let Some(locked) = self.proxy.receive_locked_hint_changed().next() {
            if self.proxy.active()? {
                let locked_hint = locked.get()?;
                info!(locked_hint = locked_hint);
                if !locked_hint {
                    handle = Self::run_cmd(&self.unlock_cmd)?.into();
                } else if let Some(h) = &handle {
                    Self::handle_prev_output(&self.unlock_cmd, h)
                }
            }
        }
        Ok(())
    }

    #[instrument]
    fn handle_prev_output(unlock_cmd: &Expression, h: &Handle) -> () {
        match h.try_wait() {
            Ok(None) => {
                warn!("Child process not completed, killing");
                h.kill().unwrap_or_else(
                    |e| error!(kill.error = ?e, "Error killing child process, ignoring"),
                )
            }
            Ok(Some(output)) => info!(child.output = ?output, "Output of previous run"),
            Err(e) => warn!(child.error = ?e),
        }
    }

    #[instrument]
    fn run_cmd(unlock_cmd: &Expression) -> eyre::Result<Handle> {
        unlock_cmd
            .start()
            .wrap_err_with(|| eyre!("Error starting command {:?}", unlock_cmd))
    }
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

#[derive(Debug, Eq, PartialEq)]
pub struct SessionId {
    session_id: String,
}

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
