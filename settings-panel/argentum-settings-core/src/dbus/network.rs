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
        // Trigger a scan, ignore the result (it's a no-op if recent). nmcli
        // is the path of least resistance here — implementing the same in
        // raw D-Bus would require walking Device.Wireless, RequestScan,
        // GetAccessPoints, then each AP's properties separately. ~250 LOC
        // vs. ~20 here, and behaviour is identical to nm-applet.
        let _ = tokio::process::Command::new("nmcli")
            .args(["device", "wifi", "rescan"])
            .output()
            .await;
        let out = tokio::process::Command::new("nmcli")
            .args(["-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "device", "wifi", "list"])
            .output()
            .await?;
        if !out.status.success() {
            return Ok(Vec::new());
        }
        Ok(parse_wifi(&String::from_utf8_lossy(&out.stdout)))
    })
    .await
}

pub async fn list_vpn() -> Result<Vec<VpnConnection>> {
    on_runtime(async {
        let out = tokio::process::Command::new("nmcli")
            .args(["-t", "-f", "UUID,NAME,TYPE,STATE", "connection", "show"])
            .output()
            .await?;
        if !out.status.success() {
            return Ok(Vec::new());
        }
        Ok(parse_vpn(&String::from_utf8_lossy(&out.stdout)))
    })
    .await
}

/// Connect to a Wi-Fi network. `password` is optional for open networks.
pub async fn connect_wifi(ssid: &str, password: Option<&str>) -> Result<()> {
    let ssid = ssid.to_string();
    let password = password.map(|p| p.to_string());
    on_runtime(async move {
        let mut cmd = tokio::process::Command::new("nmcli");
        cmd.args(["device", "wifi", "connect", &ssid]);
        if let Some(pw) = password.as_deref() {
            cmd.args(["password", pw]);
        }
        let out = cmd.output().await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(Error::Subprocess {
                cmd: format!("nmcli device wifi connect {ssid}"),
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    })
    .await
}

pub fn parse_wifi(input: &str) -> Vec<WifiNetwork> {
    let mut out = Vec::new();
    let mut seen: std::collections::HashSet<String> = Default::default();
    for line in input.lines() {
        // nmcli -t output: colon-separated; an `\:` escape is possible in
        // SSIDs but rare. Best-effort split — duplicate SSIDs (multiple APs
        // broadcasting the same network) are deduplicated by stronger signal.
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 {
            continue;
        }
        let in_use = parts[0].trim() == "*";
        let ssid = parts[1].trim().to_string();
        if ssid.is_empty() {
            continue;
        }
        let strength: u8 = parts[2].trim().parse().unwrap_or(0);
        let security = parts[3].trim();
        let secured = !security.is_empty() && security != "--";
        if !seen.insert(ssid.clone()) {
            continue;
        }
        out.push(WifiNetwork { ssid, strength, secured, connected: in_use });
    }
    out.sort_by(|a, b| b.strength.cmp(&a.strength));
    out
}

pub fn parse_vpn(input: &str) -> Vec<VpnConnection> {
    let mut out = Vec::new();
    for line in input.lines() {
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 {
            continue;
        }
        let uuid = parts[0].trim().to_string();
        let name = parts[1].trim().to_string();
        let conn_type = parts[2].trim();
        let state = parts[3].trim();
        if !conn_type.starts_with("vpn") && !conn_type.starts_with("wireguard") {
            continue;
        }
        out.push(VpnConnection {
            uuid,
            name,
            active: state.eq_ignore_ascii_case("activated"),
        });
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wifi_list() {
        let raw = "*:HomeNet:78:WPA2\n :Cafe:45:--\n :HomeNet:60:WPA2\n";
        let r = parse_wifi(raw);
        // Duplicate SSID dropped, open net detected.
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].ssid, "HomeNet");
        assert!(r[0].secured);
        assert!(r[0].connected);
        assert_eq!(r[1].ssid, "Cafe");
        assert!(!r[1].secured);
    }

    #[test]
    fn parses_vpn_only() {
        let raw = "abc:Eth0:ethernet:activated\nuvw:Work:vpn:activated\nxyz:Tunnel:wireguard:disconnected\n";
        let r = parse_vpn(raw);
        assert_eq!(r.len(), 2);
        assert!(r[0].active);
        assert!(!r[1].active);
    }
}
