use core::mem::MaybeUninit;
use core::slice;
use linux_raw_sys::general::linux_dirent64;

use crate::backend::fs::syscalls::getdents_uninit;
use crate::fd::AsFd;
use crate::ffi::CStr;
use crate::fs::FileType;
use crate::io;

/// A directory iterator implemented with getdents.
///
/// Note: This implementation does not handle growing the buffer. If this functionality is
/// necessary, you'll need to drop the current iterator, resize the buffer, and then
/// re-create the iterator. The iterator is guaranteed to continue where it left off provided
/// the file descriptor isn't changed. See the example in [`RawDir::new`].
pub struct RawDir<'buf, Fd: AsFd> {
    fd: Fd,
    buf: &'buf mut [MaybeUninit<u8>],
    initialized: usize,
    offset: usize,
}

impl<'buf, Fd: AsFd> RawDir<'buf, Fd> {
    /// Create a new iterator from the given file descriptor and buffer.
    ///
    /// # Examples
    ///
    /// Using the heap:
    ///
    /// ```
    /// # use std::mem::MaybeUninit;
    /// # use rustix::fs::{cwd, Mode, OFlags, openat, RawDir};
    ///
    /// let fd = openat(cwd(), ".", OFlags::RDONLY | OFlags::DIRECTORY, Mode::empty()).unwrap();
    ///
    /// let mut buf = Vec::with_capacity(8192);
    /// for entry in RawDir::new(fd, buf.spare_capacity_mut()) {
    ///     let entry = entry.unwrap();
    ///     dbg!(&entry);
    /// }
    /// ```
    ///
    /// Using the stack:
    ///
    /// ```
    /// # use std::mem::MaybeUninit;
    /// # use rustix::fs::{cwd, Mode, OFlags, openat, RawDir};
    ///
    /// let fd = openat(cwd(), ".", OFlags::RDONLY | OFlags::DIRECTORY, Mode::empty()).unwrap();
    ///
    /// let mut buf = [MaybeUninit::uninit(); 2048];
    /// for entry in RawDir::new(fd, &mut buf) {
    ///     let entry = entry.unwrap();
    ///     dbg!(&entry);
    /// }
    /// ```
    ///
    /// When reading directory entries with arbitrarily large file names is required:
    ///
    /// ```
    /// # use std::mem::MaybeUninit;
    /// # use rustix::fs::{cwd, Mode, OFlags, openat, RawDir};
    /// # use rustix::io::Errno;
    ///
    /// let fd = openat(cwd(), ".", OFlags::RDONLY | OFlags::DIRECTORY, Mode::empty()).unwrap();
    ///
    /// let mut buf = Vec::with_capacity(8192);
    /// 'read: loop {
    ///     'resize: {
    ///         for entry in RawDir::new(&fd, buf.spare_capacity_mut()) {
    ///             let entry = match entry {
    ///                 Err(Errno::INVAL) => break 'resize,
    ///                 r => r.unwrap(),
    ///             };
    ///             dbg!(&entry);
    ///         }
    ///         break 'read;
    ///     }
    ///
    ///     let new_capacity = buf.capacity() * 2;
    ///     buf.reserve(new_capacity);
    /// }
    /// ```
    pub fn new(fd: Fd, buf: &'buf mut [MaybeUninit<u8>]) -> Self {
        Self {
            fd,
            buf,
            initialized: 0,
            offset: 0,
        }
    }
}

/// A raw directory entry, similar to `std::fs::DirEntry`.
///
/// Note that unlike the std version, this may represent the `.` or `..` entries.
#[derive(Debug)]
pub struct RawDirEntry<'a> {
    file_name: &'a CStr,
    file_type: u8,
    inode_number: u64,
    next_entry_cookie: i64,
}

impl<'a> RawDirEntry<'a> {
    /// Returns the file name of this directory entry.
    #[inline]
    pub fn file_name(&self) -> &CStr {
        self.file_name
    }

    /// Returns the type of this directory entry.
    #[inline]
    pub fn file_type(&self) -> FileType {
        FileType::from_dirent_d_type(self.file_type)
    }

    /// Returns the inode number of this directory entry.
    #[inline]
    #[doc(alias = "inode_number")]
    pub fn ino(&self) -> u64 {
        self.inode_number
    }

    /// Returns the seek cookie to the next directory entry.
    #[inline]
    #[doc(alias = "off")]
    pub fn next_entry_cookie(&self) -> i64 {
        self.next_entry_cookie
    }
}

impl<'buf, Fd: AsFd> Iterator for RawDir<'buf, Fd> {
    type Item = io::Result<RawDirEntry<'buf>>;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.offset < self.initialized {
                let dirent_ptr = self.buf[self.offset..].as_ptr();
                // SAFETY:
                // - This data is initialized by the check above.
                //   - Assumption: the kernel will not give us partial structs.
                // - Assumption: the kernel uses proper alignment.
                let dirent = unsafe { &*dirent_ptr.cast::<linux_dirent64>() };

                self.offset += usize::from(dirent.d_reclen);

                return Some(Ok(RawDirEntry {
                    file_type: dirent.d_type,
                    inode_number: dirent.d_ino,
                    next_entry_cookie: dirent.d_off,
                    file_name: {
                        let name_start = dirent.d_name.as_ptr().cast::<u8>();
                        let mut name_len = {
                            // On 32-bit platforms, the kernel continues to use 8 byte alignment,
                            // so cannot rely on size_of or align_of and must instead hardcode
                            // that knowledge in.
                            const ALIGNMENT: usize = 8;
                            const DIRENT64_PACKED_SIZE: usize = 19;

                            // Find the last non-padding byte of the file name so we can
                            // start searching for NUL bytes. If we started searching
                            // directly from the back, we would run into garbage left over
                            // from previous iterations.
                            usize::from(dirent.d_reclen)
                                .saturating_sub(DIRENT64_PACKED_SIZE + ALIGNMENT)
                        };

                        // SAFETY:
                        // - We start searching from within the dirent AND before the padding bytes.
                        // - The kernel guarantees a NUL terminated name.
                        while unsafe { *name_start.add(name_len) } != 0 {
                            name_len += 1;
                        }
                        // Add 1 for the NUL byte
                        name_len += 1;

                        // SAFETY: We found the NUL byte and length above.
                        unsafe {
                            let file_name = CStr::from_bytes_with_nul_unchecked(
                                slice::from_raw_parts(name_start, name_len),
                            );
                            debug_assert_eq!(file_name, CStr::from_ptr(name_start.cast()));
                            file_name
                        }
                    },
                }));
            }
            self.initialized = 0;
            self.offset = 0;

            match getdents_uninit(self.fd.as_fd(), self.buf) {
                Ok(bytes_read) if bytes_read == 0 => return None,
                Ok(bytes_read) => self.initialized = bytes_read,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
