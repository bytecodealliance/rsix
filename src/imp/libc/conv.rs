#![allow(dead_code)]

#[cfg(windows)]
use super::net::io_lifetimes;
#[cfg(not(windows))]
use super::offset::libc_off_t;
use crate::io;
use crate::io::{
    AsRawFd, BorrowedFd, FromFd, FromRawFd, IntoFd, IntoRawFd, LibcFd, OwnedFd, RawFd,
};
#[cfg(windows)]
use std::convert::TryInto;
use std::ffi::CStr;

#[inline]
pub(super) fn c_str(c: &CStr) -> *const libc::c_char {
    c.as_ptr().cast::<libc::c_char>()
}

#[cfg(not(windows))]
#[inline]
pub(super) fn no_fd() -> LibcFd {
    -1
}

#[inline]
pub(super) fn borrowed_fd(fd: BorrowedFd<'_>) -> LibcFd {
    fd.as_raw_fd() as LibcFd
}

#[inline]
pub(super) fn owned_fd(fd: OwnedFd) -> LibcFd {
    fd.into_fd().into_raw_fd() as LibcFd
}

#[inline]
pub(super) fn ret(raw: libc::c_int) -> io::Result<()> {
    if raw == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[inline]
pub(super) fn syscall_ret(raw: libc::c_long) -> io::Result<()> {
    if raw == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[inline]
pub(super) fn nonnegative_ret(raw: libc::c_int) -> io::Result<()> {
    if raw >= 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[inline]
pub(super) unsafe fn ret_infallible(raw: libc::c_int) {
    debug_assert_eq!(raw, 0);
}

#[inline]
pub(super) fn ret_c_int(raw: libc::c_int) -> io::Result<libc::c_int> {
    if raw == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(raw)
    }
}

#[inline]
pub(super) fn ret_u32(raw: libc::c_int) -> io::Result<u32> {
    if raw == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(raw as u32)
    }
}

#[inline]
pub(super) fn ret_ssize_t(raw: libc::ssize_t) -> io::Result<libc::ssize_t> {
    if raw == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(raw)
    }
}

#[inline]
pub(super) fn syscall_ret_ssize_t(raw: libc::c_long) -> io::Result<libc::ssize_t> {
    if raw == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(raw as libc::ssize_t)
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub(super) fn syscall_ret_u32(raw: libc::c_long) -> io::Result<u32> {
    if raw == -1 {
        Err(io::Error::last_os_error())
    } else {
        let r32 = raw as u32;

        // Converting `raw` to `u32` should be lossless.
        debug_assert_eq!(r32 as libc::c_long, raw);

        Ok(r32)
    }
}

#[cfg(not(windows))]
#[inline]
pub(super) fn ret_off_t(raw: libc_off_t) -> io::Result<libc_off_t> {
    if raw == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(raw)
    }
}

/// Convert a c_int returned from a libc function to an `OwnedFd`, if valid.
///
/// # Safety
///
/// The caller must ensure that this is the return value of a libc function
/// which returns an owned file descriptor.
#[inline]
pub(super) unsafe fn ret_owned_fd(raw: LibcFd) -> io::Result<OwnedFd> {
    if raw == !0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(OwnedFd::from_fd(io_lifetimes::OwnedFd::from_raw_fd(
            raw as RawFd,
        )))
    }
}

#[inline]
pub(super) fn ret_discarded_fd(raw: LibcFd) -> io::Result<()> {
    if raw == !0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[inline]
pub(super) fn ret_discarded_char_ptr(raw: *mut libc::c_char) -> io::Result<()> {
    if raw.is_null() {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Convert a c_long returned from `syscall` to an `OwnedFd`, if valid.
///
/// # Safety
///
/// The caller must ensure that this is the return value of a `syscall` call
/// which returns an owned file descriptor.
#[cfg(not(windows))]
#[inline]
pub(super) unsafe fn syscall_ret_owned_fd(raw: libc::c_long) -> io::Result<OwnedFd> {
    if raw == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(OwnedFd::from_fd(io_lifetimes::OwnedFd::from_raw_fd(
            raw as RawFd,
        )))
    }
}

/// Convert the buffer-length argument value of a `send` or `recv` call.
#[cfg(not(windows))]
#[inline]
pub(super) fn send_recv_len(len: usize) -> usize {
    len
}

/// Convert the buffer-length argument value of a `send` or `recv` call.
#[cfg(windows)]
#[inline]
pub(super) fn send_recv_len(len: usize) -> i32 {
    // On Windows, the length argument has type `i32`; saturate the length,
    // since `send` and `recv` are allowed to send and recv less data than
    // requested.
    len.try_into().unwrap_or(i32::MAX)
}

/// Convert the return value of a `send` or `recv` call.
#[cfg(not(windows))]
#[inline]
pub(super) fn ret_send_recv(len: isize) -> io::Result<libc::ssize_t> {
    ret_ssize_t(len)
}

/// Convert the return value of a `send` or `recv` call.
#[cfg(windows)]
#[inline]
pub(super) fn ret_send_recv(len: i32) -> io::Result<libc::ssize_t> {
    ret_ssize_t(len as isize)
}
