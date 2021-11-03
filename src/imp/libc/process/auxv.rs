use super::super::c;
#[cfg(any(target_os = "android", target_os = "linux"))]
use std::ffi::CStr;

#[inline]
pub(crate) fn page_size() -> usize {
    unsafe { c::sysconf(c::_SC_PAGESIZE) as usize }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub(crate) fn linux_hwcap() -> (usize, usize) {
    unsafe {
        let hwcap = c::getauxval(c::AT_HWCAP) as usize;
        let hwcap2 = c::getauxval(c::AT_HWCAP2) as usize;
        (hwcap, hwcap2)
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub(crate) fn linux_execfn() -> &'static CStr {
    unsafe { CStr::from_ptr(c::getauxval(c::AT_EXECFN) as *const c::c_char) }
}
