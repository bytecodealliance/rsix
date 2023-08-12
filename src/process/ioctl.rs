use crate::{backend, io, ioctl};
use backend::{c, fd::AsFd};

/// `ioctl(fd, TIOCSCTTY, 0)`—Sets the controlling terminal for the processs.
///
/// # References
///  - [Linux]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=tty&sektion=4
/// [NetBSD]: https://man.netbsd.org/tty.4
/// [OpenBSD]: https://man.openbsd.org/tty.4
#[cfg(not(any(windows, target_os = "aix", target_os = "redox", target_os = "wasi")))]
#[inline]
#[doc(alias = "TIOCSCTTY")]
pub fn ioctl_tiocsctty<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    ioctl::ioctl(fd, Tiocsctty)
}

#[cfg(not(any(windows, target_os = "aix", target_os = "redox", target_os = "wasi")))]
struct Tiocsctty;

#[allow(unsafe_code)]
#[cfg(not(any(windows, target_os = "aix", target_os = "redox", target_os = "wasi")))]
unsafe impl ioctl::Ioctl for Tiocsctty {
    type Output = ();
    const OPCODE: ioctl::Opcode = ioctl::Opcode::Bad(c::TIOCSCTTY as ioctl::RawOpcode);
    const IS_MUTATING: bool = false;

    fn as_ptr(&mut self) -> *mut c::c_void {
        (&0u32) as *const u32 as *mut c::c_void
    }

    unsafe fn output_from_ptr(
        _: ioctl::IoctlOutput,
        _: *mut c::c_void,
    ) -> io::Result<Self::Output> {
        Ok(())
    }
}
