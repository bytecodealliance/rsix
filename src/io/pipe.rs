use crate::backend;
use crate::fd::OwnedFd;
use crate::io;
use backend::fd::AsFd;

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::io::types::PipeFlags;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub use backend::io::types::SpliceFlags;

/// `PIPE_BUF`—The maximum length at which writes to a pipe are atomic.
///
/// # References
///  - [Linux]
///  - [POSIX]
///
/// [Linux]: https://man7.org/linux/man-pages/man7/pipe.7.html
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/write.html
#[cfg(not(any(
    windows,
    target_os = "illumos",
    target_os = "redox",
    target_os = "solaris",
    target_os = "wasi",
)))]
pub const PIPE_BUF: usize = backend::io::types::PIPE_BUF;

/// `pipe()`—Creates a pipe.
///
/// This function creates a pipe and returns two file descriptors, for the
/// reading and writing ends of the pipe, respectively.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/pipe.html
/// [Linux]: https://man7.org/linux/man-pages/man2/pipe.2.html
#[inline]
pub fn pipe() -> io::Result<(OwnedFd, OwnedFd)> {
    backend::io::syscalls::pipe()
}

/// `pipe2(flags)`—Creates a pipe, with flags.
///
/// This function creates a pipe and returns two file descriptors, for the
/// reading and writing ends of the pipe, respectively.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/pipe2.2.html
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[inline]
#[doc(alias = "pipe2")]
pub fn pipe_with(flags: PipeFlags) -> io::Result<(OwnedFd, OwnedFd)> {
    backend::io::syscalls::pipe_with(flags)
}

/// splice
#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub fn splice<FdIn: AsFd, FdOut: AsFd>(
    fd_in: FdIn,
    off_in: Option<u64>,
    fd_out: FdOut,
    off_out: Option<u64>,
    len: usize,
    flags: SpliceFlags,
) -> io::Result<usize> {
    backend::io::syscalls::splice(fd_in.as_fd(), off_in, fd_out.as_fd(), off_out, len, flags)
}
