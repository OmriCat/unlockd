use color_eyre::eyre::{self, eyre};
use tracing::instrument;
use zbus::blocking::Connection;
use zbus::dbus_proxy;
use zbus::zvariant::OwnedObjectPath;

use crate::session_id::SessionId;

#[dbus_proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait Manager {
    fn list_sessions(&self) -> zbus::Result<Vec<(String, u32, String, String, OwnedObjectPath)>>;
}

#[instrument]
pub fn session_path_from_id(
    connection: &Connection,
    session_id: SessionId,
) -> eyre::Result<OwnedObjectPath> {
    let manager = ManagerProxyBlocking::builder(&connection).build()?;

    let sessions = manager.list_sessions()?;

    sessions
        .into_iter()
        .find_map(|(sess_id, _uid, _uname, _seat, obj_path)| {
            (session_id == sess_id.parse().ok()?).then_some(obj_path)
        })
        .ok_or_else(|| eyre!("Can't find session object path for session {session_id}"))
}
