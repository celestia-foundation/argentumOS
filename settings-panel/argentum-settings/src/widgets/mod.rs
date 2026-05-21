//! Reusable input widgets. Modal text/password prompts are implemented via
//! `zenity` because GPUI at the pinned commit lacks a working inline text
//! input primitive. The interface is async-Future shaped so a future inline
//! widget can drop in without changing call sites.

pub mod prompt;
