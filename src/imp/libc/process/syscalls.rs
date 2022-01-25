//! libc syscalls supporting `rustix::process`.

use super::super::c;
#[cfg(not(any(target_os = "wasi", target_os = "fuchsia")))]
use super::super::conv::borrowed_fd;
use super::super::conv::{c_str, ret, ret_c_int, ret_discarded_char_ptr};
#[cfg(any(target_os = "android", target_os = "linux"))]
use super::super::conv::{syscall_ret, syscall_ret_u32};
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "fuchsia",
    target_os = "dragonfly"
))]
use super::RawCpuSet;
#[cfg(not(any(target_os = "wasi", target_os = "fuchsia")))]
use crate::fd::BorrowedFd;
use crate::ffi::ZStr;
use crate::io;
use core::mem::MaybeUninit;
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
use {
    super::super::conv::ret_infallible,
    super::super::offset::{libc_getrlimit, libc_rlimit, libc_setrlimit, LIBC_RLIM_INFINITY},
    crate::as_ptr,
    crate::process::{Resource, Rlimit},
    core::convert::TryInto,
};
#[cfg(any(target_os = "android", target_os = "linux"))]
use {
    super::super::offset::libc_prlimit,
    crate::process::{Cpuid, MembarrierCommand, MembarrierQuery},
};
#[cfg(not(target_os = "wasi"))]
use {
    super::RawUname,
    crate::process::{Gid, Pid, RawNonZeroPid, RawPid, Signal, Uid, WaitOptions, WaitStatus},
};

#[cfg(not(target_os = "wasi"))]
pub(crate) fn chdir(path: &ZStr) -> io::Result<()> {
    unsafe { ret(c::chdir(c_str(path))) }
}

#[cfg(not(any(target_os = "wasi", target_os = "fuchsia")))]
pub(crate) fn fchdir(dirfd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(c::fchdir(borrowed_fd(dirfd))) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn getcwd(buf: &mut [u8]) -> io::Result<()> {
    unsafe { ret_discarded_char_ptr(c::getcwd(buf.as_mut_ptr().cast(), buf.len())) }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub(crate) fn membarrier_query() -> MembarrierQuery {
    const MEMBARRIER_CMD_QUERY: u32 = 0;
    unsafe {
        match syscall_ret_u32(c::syscall(c::SYS_membarrier, MEMBARRIER_CMD_QUERY, 0)) {
            Ok(query) => MembarrierQuery::from_bits_unchecked(query),
            Err(_) => MembarrierQuery::empty(),
        }
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub(crate) fn membarrier(cmd: MembarrierCommand) -> io::Result<()> {
    unsafe { syscall_ret(c::syscall(c::SYS_membarrier, cmd as u32, 0)) }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub(crate) fn membarrier_cpu(cmd: MembarrierCommand, cpu: Cpuid) -> io::Result<()> {
    const MEMBARRIER_CMD_FLAG_CPU: u32 = 1;
    unsafe {
        syscall_ret(c::syscall(
            c::SYS_membarrier,
            cmd as u32,
            MEMBARRIER_CMD_FLAG_CPU,
            cpu.as_raw(),
        ))
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getuid() -> Uid {
    unsafe {
        let uid = c::getuid();
        Uid::from_raw(uid)
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn geteuid() -> Uid {
    unsafe {
        let uid = c::geteuid();
        Uid::from_raw(uid)
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getgid() -> Gid {
    unsafe {
        let gid = c::getgid();
        Gid::from_raw(gid)
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getegid() -> Gid {
    unsafe {
        let gid = c::getegid();
        Gid::from_raw(gid)
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getpid() -> Pid {
    unsafe {
        let pid = c::getpid();
        debug_assert_ne!(pid, 0);
        Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pid))
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getppid() -> Option<Pid> {
    unsafe {
        let pid: i32 = c::getppid();
        Pid::from_raw(pid)
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "fuchsia",
    target_os = "dragonfly"
))]
#[inline]
pub(crate) fn sched_getaffinity(pid: Option<Pid>, cpuset: &mut RawCpuSet) -> io::Result<()> {
    unsafe {
        ret(c::sched_getaffinity(
            Pid::as_raw(pid) as _,
            core::mem::size_of::<RawCpuSet>(),
            cpuset,
        ))
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "fuchsia",
    target_os = "dragonfly"
))]
#[inline]
pub(crate) fn sched_setaffinity(pid: Option<Pid>, cpuset: &RawCpuSet) -> io::Result<()> {
    unsafe {
        ret(c::sched_setaffinity(
            Pid::as_raw(pid) as _,
            core::mem::size_of::<RawCpuSet>(),
            cpuset,
        ))
    }
}

#[inline]
pub(crate) fn sched_yield() {
    unsafe {
        let _ = c::sched_yield();
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn uname() -> RawUname {
    let mut uname = MaybeUninit::<RawUname>::uninit();
    unsafe {
        ret(c::uname(uname.as_mut_ptr())).unwrap();
        uname.assume_init()
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[inline]
pub(crate) fn nice(inc: i32) -> io::Result<i32> {
    errno::set_errno(errno::Errno(0));
    let r = unsafe { c::nice(inc) };
    if errno::errno().0 != 0 {
        ret_c_int(r)
    } else {
        Ok(r)
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn getpriority_user(uid: Uid) -> io::Result<i32> {
    errno::set_errno(errno::Errno(0));
    let r = unsafe { c::getpriority(c::PRIO_USER, uid.as_raw() as _) };
    if errno::errno().0 != 0 {
        ret_c_int(r)
    } else {
        Ok(r)
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn getpriority_pgrp(pgid: Option<Pid>) -> io::Result<i32> {
    errno::set_errno(errno::Errno(0));
    let r = unsafe { c::getpriority(c::PRIO_PGRP, Pid::as_raw(pgid) as _) };
    if errno::errno().0 != 0 {
        ret_c_int(r)
    } else {
        Ok(r)
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn getpriority_process(pid: Option<Pid>) -> io::Result<i32> {
    errno::set_errno(errno::Errno(0));
    let r = unsafe { c::getpriority(c::PRIO_PROCESS, Pid::as_raw(pid) as _) };
    if errno::errno().0 != 0 {
        ret_c_int(r)
    } else {
        Ok(r)
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn setpriority_user(uid: Uid, priority: i32) -> io::Result<()> {
    unsafe { ret(c::setpriority(c::PRIO_USER, uid.as_raw() as _, priority)) }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn setpriority_pgrp(pgid: Option<Pid>, priority: i32) -> io::Result<()> {
    unsafe {
        ret(c::setpriority(
            c::PRIO_PGRP,
            Pid::as_raw(pgid) as _,
            priority,
        ))
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn setpriority_process(pid: Option<Pid>, priority: i32) -> io::Result<()> {
    unsafe {
        ret(c::setpriority(
            c::PRIO_PROCESS,
            Pid::as_raw(pid) as _,
            priority,
        ))
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn getrlimit(limit: Resource) -> Rlimit {
    let mut result = MaybeUninit::<libc_rlimit>::uninit();
    unsafe {
        ret_infallible(libc_getrlimit(limit as _, result.as_mut_ptr()));
        let result = result.assume_init();
        let current = if result.rlim_cur == LIBC_RLIM_INFINITY {
            None
        } else {
            result.rlim_cur.try_into().ok()
        };
        let maximum = if result.rlim_max == LIBC_RLIM_INFINITY {
            None
        } else {
            result.rlim_max.try_into().ok()
        };
        Rlimit { current, maximum }
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn setrlimit(limit: Resource, new: Rlimit) -> io::Result<()> {
    let rlim_cur = match new.current {
        Some(r) => r.try_into().map_err(|_| io::Error::INVAL)?,
        None => LIBC_RLIM_INFINITY as _,
    };
    let rlim_max = match new.maximum {
        Some(r) => r.try_into().map_err(|_| io::Error::INVAL)?,
        None => LIBC_RLIM_INFINITY as _,
    };
    let lim = libc_rlimit { rlim_cur, rlim_max };
    unsafe { ret(libc_setrlimit(limit as _, as_ptr(&lim))) }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub(crate) fn prlimit(pid: Option<Pid>, limit: Resource, new: Rlimit) -> io::Result<Rlimit> {
    let lim = libc_rlimit {
        rlim_cur: new.current.unwrap_or(LIBC_RLIM_INFINITY),
        rlim_max: new.maximum.unwrap_or(LIBC_RLIM_INFINITY),
    };
    let mut result = MaybeUninit::<libc_rlimit>::uninit();
    unsafe {
        ret_infallible(libc_prlimit(
            Pid::as_raw(pid),
            limit as _,
            as_ptr(&lim),
            result.as_mut_ptr(),
        ));
        let result = result.assume_init();
        let current = if result.rlim_cur == LIBC_RLIM_INFINITY {
            None
        } else {
            result.rlim_cur.try_into().ok()
        };
        let maximum = if result.rlim_max == LIBC_RLIM_INFINITY {
            None
        } else {
            result.rlim_max.try_into().ok()
        };
        Ok(Rlimit { current, maximum })
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn wait(waitopts: WaitOptions) -> io::Result<Option<(Pid, WaitStatus)>> {
    _waitpid(!0, waitopts)
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn waitpid(
    pid: Option<Pid>,
    waitopts: WaitOptions,
) -> io::Result<Option<(Pid, WaitStatus)>> {
    _waitpid(Pid::as_raw(pid), waitopts)
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn _waitpid(
    pid: RawPid,
    waitopts: WaitOptions,
) -> io::Result<Option<(Pid, WaitStatus)>> {
    unsafe {
        let mut status: c::c_int = 0;
        let pid = ret_c_int(c::waitpid(pid as _, &mut status, waitopts.bits() as _))?;
        Ok(RawNonZeroPid::new(pid).map(|non_zero| {
            (
                Pid::from_raw_nonzero(non_zero),
                WaitStatus::new(status as _),
            )
        }))
    }
}

#[inline]
pub(crate) fn exit_group(code: c::c_int) -> ! {
    // `_exit` and `_Exit` are the same; it's just a matter of which ones
    // the libc bindings expose.
    #[cfg(any(target_os = "wasi", target_os = "solid"))]
    unsafe {
        libc::_Exit(code)
    }
    #[cfg(unix)]
    unsafe {
        libc::_exit(code)
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn setsid() -> io::Result<Pid> {
    unsafe {
        let pid = ret_c_int(libc::setsid())?;
        debug_assert_ne!(pid, 0);
        Ok(Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pid)))
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn kill_process(pid: Pid, sig: Signal) -> io::Result<()> {
    unsafe { ret(libc::kill(pid.as_raw_nonzero().get(), sig as i32)) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn kill_process_group(pid: Pid, sig: Signal) -> io::Result<()> {
    unsafe {
        ret(libc::kill(
            pid.as_raw_nonzero().get().wrapping_neg(),
            sig as i32,
        ))
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn kill_current_process_group(sig: Signal) -> io::Result<()> {
    unsafe { ret(libc::kill(0, sig as i32)) }
}
