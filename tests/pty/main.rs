//! Tests for [`rustix::pty`].

#![cfg(feature = "pty")]

#[cfg(any(apple, linux_like, target_os = "freebsd", target_os = "fuchsia"))]
mod openpty;
