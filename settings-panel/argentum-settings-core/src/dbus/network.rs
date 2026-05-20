//! NetworkManager backend via async zbus.

use crate::{Error, Result, on_runtime};
use zbus::{Connection, proxy};

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    fn get_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    fn activate_connection(
        &self,
        connection: &zbus::zvariant::ObjectPath<'_>,
        device: &zbus::zvariant::ObjectPath<'_>,
        specific_object: &zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn deactivate_connection(&self, active: &zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;
    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub strength: u8,
    pub secured: bool,
    pub connected: bool,
}

#[derive(Debug, Clone)]
pub struct VpnConnection {
    pub uuid: String,
    pub name: String,
    pub active: bool,
}

pub async fn list_wifi() -> Result<Vec<WifiNetwork>> {
    on_runtime(async {
        let conn = Connection::system().await?;
        let _nm = NetworkManagerProxy::new(&conn).await?;
        // TODO: implement full AP enumeration via Device.Wireless + AccessPoint proxies.
        Ok(Vec::new())
    })
    .await
}

pub async fn list_vpn() -> Result<Vec<VpnConnection>> {
    on_runtime(async {
        // TODO: walk Settings.ListConnections, filter type=vpn, cross-ref ActiveConnections.
        Ok(Vec::new())
    })
    .await
}

pub async fn activate(connection_path: &str) -> Result<()> {
    let connection_path = connection_path.to_string();
    on_runtime(async move {
        let conn = Connection::system().await?;
        let nm = NetworkManagerProxy::new(&conn).await?;
        let conn_op = zbus::zvariant::ObjectPath::try_from(connection_path.as_str())
            .map_err(|e| Error::DBus(e.to_string()))?;
        let empty = zbus::zvariant::ObjectPath::try_from("/")
            .map_err(|e| Error::DBus(e.to_string()))?;
        nm.activate_connection(&conn_op, &empty, &empty).await?;
        Ok(())
    })
    .await
}

pub async fn deactivate(active_path: &str) -> Result<()> {
    let active_path = active_path.to_string();
    on_runtime(async move {
        let conn = Connection::system().await?;
        let nm = NetworkManagerProxy::new(&conn).await?;
        let active_op = zbus::zvariant::ObjectPath::try_from(active_path.as_str())
            .map_err(|e| Error::DBus(e.to_string()))?;
        nm.deactivate_connection(&active_op).await?;
        Ok(())
    })
    .await
}

// TODO: WiFi password modal flow.
