<div align="center">
  <h1><code>rsix</code></h1>

  <p>
    <strong>Safe Rust ("rs") bindings to POSIX-like/Unix-like/Linux ("ix") syscalls</strong>
  </p>

  <strong>A <a href="https://bytecodealliance.org/">Bytecode Alliance</a> project</strong>

  <p>
    <a href="https://github.com/bytecodealliance/rsix/actions?query=workflow%3ACI"><img src="https://github.com/bytecodealliance/rsix/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/217126-wasmtime"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/rsix"><img src="https://img.shields.io/crates/v/rsix.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/rsix"><img src="https://docs.rs/rsix/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

`rsix` (formerly known as `posish`) provides efficient memory-safe and
[I/O-safe] wrappers to POSIX-like, Unix-like, and Linux syscall APIs, with
configurable backends. It uses Rust references, slices, and return values
instead of raw pointers, and [`io-lifetimes`] instead of raw file descriptors,
providing memory safety and [I/O safety]. It uses `Result`s for reporting
errors, [`bitflags`] instead of bare integer flags, an [`Arg`] trait with
optimizations to efficiently accept any Rust string type, and several other
efficient conveniences.

`rsix` is low-level and does not support Windows; for higher-level and more
portable APIs built on this functionality, see the [`system-interface`],
[`cap-std`], and [`fs-set-times`] crates, for example.

`rsix` currently has two backends available: `linux_raw` and `libc`.

The `linux_raw` backend is enabled by default on Linux on x86-64, x86, aarch64,
and riscv64gc, and uses raw Linux system calls and vDSO calls. It supports
stable as well as nightly Rust.
 - By being implemented entirely in Rust, avoiding `libc`, `errno`, and pthread
   cancellation, and employing some specialized optimizations, most functions
   compile down to very efficient code. On nightly Rust, they can often be
   fully inlined into user code.
 - Most functions in `linux_raw` preserve memory and I/O safety all the way
   down to the syscalls.
 - `linux_raw` uses a 64-bit `time_t` type on all platforms, avoiding the
   [y2038 bug].

The `libc` backend is enabled by default on all other platforms, and can be
set explicitly for any target by setting `RUSTFLAGS` to `--cfg rsix_use_libc`.
It uses the [`libc`] crate which provides bindings to native `libc` libraries
and is portable to many OS's.

## Similar crates

`rsix` is similar to [`nix`], [`simple_libc`], [`unix`], and [`nc`]. `rsix` is
a relatively new project with less overall coverage, architected for
[I/O safety] with most APIs using [`OwnedFd`] and [`AsFd`] to manipulate file
descriptors rather than `File` or even `c_int`, and supporting multiple
backends so that it can use direct syscalls while still being usable on all
platforms `libc` supports. Like `nix`, `rsix` has an optimized and flexible
filename argument mechanism that allows users to use a variety of string types,
including non-UTF-8 string types.

[`relibc`] is a similar project which aims to be a full "libc", including
C-compatible interfaces and higher-level C/POSIX standard-library
functionality; `rsix` just aims to provide safe and idiomatic Rust interfaces
to low-level syscalls. `relibc` also doesn't tend to support features not
supported on Redox, such as `*at` functions like `openat`, which are
important features for `rsix`.

`rsix` has its own code for making direct syscalls, similar to the [`sc`]
and [`scall`] crates, though `rsix` currently only supports direct syscalls on
Linux on x86\_64, x86, aarch64, and riscv64. `rsix` can use either the unstable
Rust `asm!` macro or out-of-line `.s` files so it supports both Stable and
Nightly Rust. `rsix`'s syscalls report errors using an optimized `Error` type,
and `rsix` supports Linux's vDSO mechanism to optimize Linux `clock_gettime` on
all architectures, and all Linux system calls on x86.

`rsix`'s `*at` functions are similar to the [`openat`] crate, but `rsix`
provides them as free functions rather than associated functions of a `Dir`
type. `rsix`'s `cwd()` function exposes the special `AT_FDCWD` value in a safe
way, so users don't need to open `.` to get a current-directory handle.

`rsix`'s `openat2` function is similar to the [`openat2`] crate, but uses
I/O safety types rather than `RawFd`. `rsix` does not provide dynamic feature
detection, so users must handle `NOSYS` themselves.

[`nix`]: https://crates.io/crates/nix
[`unix`]: https://crates.io/crates/unix
[`nc`]: https://crates.io/crates/nc
[`simple_libc`]: https://crates.io/crates/simple_libc
[`relibc`]: https://github.com/redox-os/relibc
[`syscall`]: https://crates.io/crates/syscall
[`sc`]: https://crates.io/crates/sc
[`scall`]: https://crates.io/crates/scall
[`system-interface`]: https://crates.io/crates/system-interface
[`openat`]: https://crates.io/crates/openat
[`openat2`]: https://crates.io/crates/openat2
[`fs-set-times`]: https://crates.io/crates/fs-set-times
[`io-lifetimes`]: https://crates.io/crates/io-lifetimes
[`libc`]: https://crates.io/crates/libc
[`cap-std`]: https://crates.io/crates/cap-std
[`bitflags`]: https://crates.io/crates/bitflags
[`Arg`]: https://docs.rs/rsix/latest/rsix/path/trait.Arg.html
[I/O-safe]: https://github.com/rust-lang/rfcs/pull/3128
[I/O safety]: https://github.com/rust-lang/rfcs/pull/3128
[y2038 bug]: https://en.wikipedia.org/wiki/Year_2038_problem
[`OwnedFd`]: https://docs.rs/io-lifetimes/latest/io_lifetimes/struct.OwnedFd.html
[`AsFd`]: https://docs.rs/io-lifetimes/latest/io_lifetimes/trait.AsFd.html
