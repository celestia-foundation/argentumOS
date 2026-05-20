//! Flatpak CLI wrappers. Every operation defaults to `--user` scope — see
//! `app-store/README.md` for why.

pub mod catalog;
pub mod info;
pub mod install;
pub mod installed;
pub mod permissions;
pub mod remotes;
pub mod runtimes;

/// `--user` scope flag, applied to every transactional `flatpak` invocation.
pub(crate) const USER: &str = "--user";
