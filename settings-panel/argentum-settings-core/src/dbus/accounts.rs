//! AccountsService backend (system D-Bus, `org.freedesktop.Accounts`).

use crate::{Error, Result, on_runtime};
use zbus::{Connection, proxy};

#[proxy(
    interface = "org.freedesktop.Accounts",
    default_service = "org.freedesktop.Accounts",
    default_path = "/org/freedesktop/Accounts"
)]
trait Accounts {
    fn find_user_by_id(&self, id: i64) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.Accounts.User",
    default_service = "org.freedesktop.Accounts"
)]
trait User {
    #[zbus(property)]
    fn user_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn real_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn icon_file(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn account_type(&self) -> zbus::Result<i32>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    Standard,
    Administrator,
}

#[derive(Debug, Clone)]
pub struct UserAccount {
    pub username: String,
    pub real_name: String,
    pub icon_file: String,
    pub account_type: AccountType,
}

pub async fn current() -> Result<UserAccount> {
    on_runtime(async {
        let uid = rustix::process::getuid().as_raw() as i64;
        let conn = Connection::system().await?;
        let accounts = AccountsProxy::new(&conn).await?;
        let path = accounts.find_user_by_id(uid).await?;
        let user = UserProxy::builder(&conn)
            .path(path)
            .map_err(|e| Error::DBus(e.to_string()))?
            .build()
            .await?;
        Ok(UserAccount {
            username: user.user_name().await.unwrap_or_default(),
            real_name: user.real_name().await.unwrap_or_default(),
            icon_file: user.icon_file().await.unwrap_or_default(),
            account_type: match user.account_type().await.unwrap_or(0) {
                1 => AccountType::Administrator,
                _ => AccountType::Standard,
            },
        })
    })
    .await
}

// TODO: change-password modal (passwd via pty).
