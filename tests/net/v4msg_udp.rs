//! Test a simple IPv4 socket server and client. The client send a
//! message and the server sends one back, uses `sendmsg` and `recvmsg`.

#![cfg(not(any(target_os = "redox", target_os = "wasi")))]

use rustix::net::{
    bind_v4, connect_v4, getsockname, recvmsg_v4, sendmsg_v4, socket, AddressFamily, Ipv4Addr,
    Protocol, RecvFlags, SendFlags, SocketAddrAny, SocketAddrV4, SocketType,
};
use std::io::{IoSlice, IoSliceMut};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

const BUFFER_SIZE: usize = 20;

fn server(ready: Arc<(Mutex<u16>, Condvar)>) {
    let connection_socket = socket(AddressFamily::INET, SocketType::DGRAM, Protocol::UDP).unwrap();

    let name = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0);
    bind_v4(&connection_socket, &name).unwrap();

    let who = match getsockname(&connection_socket).unwrap() {
        SocketAddrAny::V4(addr) => addr,
        _ => panic!(),
    };

    {
        let (lock, cvar) = &*ready;
        let mut port = lock.lock().unwrap();
        *port = who.port();
        cvar.notify_all();
    }

    // no accept for UDP
    let data_socket = connection_socket;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    let res = recvmsg_v4(
        &data_socket,
        &mut [IoSliceMut::new(&mut buffer)],
        RecvFlags::empty(),
    )
    .unwrap();
    assert!(res.addr.is_some());
    assert_eq!(
        String::from_utf8_lossy(&buffer[..res.bytes]),
        "hello, world"
    );

    sendmsg_v4(
        &data_socket,
        &[IoSlice::new(b"goodnight, moon")],
        res.addr.as_ref(),
        SendFlags::empty(),
    )
    .unwrap();
}

fn client(ready: Arc<(Mutex<u16>, Condvar)>) {
    let port = {
        let (lock, cvar) = &*ready;
        let mut port = lock.lock().unwrap();
        while *port == 0 {
            port = cvar.wait(port).unwrap();
        }
        *port
    };

    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);

    let data_socket = socket(AddressFamily::INET, SocketType::DGRAM, Protocol::UDP).unwrap();
    connect_v4(&data_socket, &addr).unwrap();

    sendmsg_v4(
        &data_socket,
        &[IoSlice::new(b"hello, world")],
        None,
        SendFlags::empty(),
    )
    .unwrap();

    let mut buffer = vec![0u8; BUFFER_SIZE];
    let res = recvmsg_v4(
        &data_socket,
        &mut [IoSliceMut::new(&mut buffer)],
        RecvFlags::empty(),
    )
    .unwrap();
    assert!(res.addr.is_some());
    assert_eq!(
        String::from_utf8_lossy(&buffer[..res.bytes]),
        "goodnight, moon"
    );
}

#[test]
fn test_v4_msg_udp() {
    #[cfg(windows)]
    rustix::net::wsa_startup().unwrap();

    let ready = Arc::new((Mutex::new(0_u16), Condvar::new()));
    let ready_clone = Arc::clone(&ready);

    let server = thread::Builder::new()
        .name("server".to_string())
        .spawn(move || {
            server(ready);
        })
        .unwrap();
    let client = thread::Builder::new()
        .name("client".to_string())
        .spawn(move || {
            client(ready_clone);
        })
        .unwrap();
    client.join().unwrap();
    server.join().unwrap();

    #[cfg(windows)]
    rustix::net::wsa_cleanup().unwrap();
}

// Verify `Ipv4PacketInfo` for `sendmsg`.
// This creates a (udp) socket bound to localhost, then sends a message to
// itself but uses Ipv4PacketInfo to force the source address to be localhost.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "netbsd"))]
#[test]
pub fn test_v4_msg_ipv4packetinfo() {
    use cfg_if::cfg_if;
    use rustix::{
        cmsg_buffer,
        net::{sendmsg_v4_with_ancillary, Ipv4PacketInfo, SendSocketAncillaryV4},
    };

    let connection_socket =
        socket(AddressFamily::INET, SocketType::DGRAM, Protocol::default()).expect("socket failed");

    let name = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 4000);
    bind_v4(&connection_socket, &name).expect("bind failed");

    let slice = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let iovs = [IoSlice::new(&slice)];

    cfg_if! {
        if #[cfg(target_os = "netbsd")] {
            let pi = Ipv4PacketInfo::default();
        } else {
            let mut pi = Ipv4PacketInfo::default();
            pi.set_local_addr(&name);
        }
    }

    let mut cmsg_buffer = cmsg_buffer!(Ipv4PacketInfo);
    let mut cmsg = SendSocketAncillaryV4::new(&mut cmsg_buffer);
    cmsg.add_packet_info(&pi);
    sendmsg_v4_with_ancillary(
        &connection_socket,
        &iovs,
        Some(&name),
        &mut cmsg,
        SendFlags::empty(),
    )
    .expect("sendmsg");
}

// #[cfg(any(
//     target_os = "android",
//     target_os = "ios",
//     target_os = "linux",
//     target_os = "macos",
//     target_os = "netbsd",
// ))]
// #[test]
// pub fn test_recv_ipv4pktinfo() {
//     let lo_ifaddr = loopback_address(AddressFamily::INET);
//     let (lo_name, lo) = match lo_ifaddr {
//         Some(ifaddr) => (
//             ifaddr.interface_name,
//             ifaddr.address.expect("Expect IPv4 address on interface"),
//         ),
//         None => return,
//     };
//     let receive = socket(AddressFamily::INET, SocketType::DGRAM, Protocol::default())
//         .expect("receive socket failed");
//     bind_v4(receive, &lo).expect("bind failed");
//     let sa = getsockname(receive).expect("getsockname failed");
//     setsockopt(receive, Ipv4PacketInfo, &true).expect("setsockopt failed");

//     {
//         let slice = [1u8, 2, 3, 4, 5, 6, 7, 8];
//         let iov = [IoVec::from_slice(&slice)];

//         let send = socket(
//             AddressFamily::Inet,
//             SockType::Datagram,
//             SockFlag::empty(),
//             None,
//         )
//         .expect("send socket failed");
//         sendmsg(send, &iov, &[], MsgFlags::empty(), Some(&sa)).expect("sendmsg failed");
//     }

//     {
//         let mut buf = [0u8; 8];
//         let iovec = [IoVec::from_mut_slice(&mut buf)];
//         let mut space = cmsg_space!(libc::in_pktinfo);
//         let msg =
//             recvmsg(receive, &iovec, Some(&mut space), MsgFlags::empty()).expect("recvmsg failed");
//         assert!(!msg
//             .flags
//             .intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC));

//         let mut cmsgs = msg.cmsgs();
//         if let Some(ControlMessageOwned::Ipv4PacketInfo(pktinfo)) = cmsgs.next() {
//             let i = if_nametoindex(lo_name.as_bytes()).expect("if_nametoindex");
//             assert_eq!(
//                 pktinfo.ipi_ifindex as libc::c_uint, i,
//                 "unexpected ifindex (expected {}, got {})",
//                 i, pktinfo.ipi_ifindex
//             );
//         }
//         assert!(cmsgs.next().is_none(), "unexpected additional control msg");
//         assert_eq!(msg.bytes, 8);
//         assert_eq!(iovec[0].as_slice(), [1u8, 2, 3, 4, 5, 6, 7, 8]);
//     }
// }

// #[cfg(any(
//     target_os = "android",
//     target_os = "freebsd",
//     target_os = "ios",
//     target_os = "linux",
//     target_os = "macos",
//     target_os = "netbsd",
//     target_os = "openbsd",
// ))]
// fn loopback_address(family: AddressFamily) -> Option<nix::ifaddrs::InterfaceAddress> {
//     use std::io;
//     use std::io::Write;

//     let addrs = match getifaddrs() {
//         Ok(iter) => iter,
//         Err(e) => {
//             let stdioerr = io::stderr();
//             let mut handle = stdioerr.lock();
//             writeln!(handle, "getifaddrs: {:?}", e).unwrap();
//             return None;
//         }
//     };
//     // return first address matching family
//     for ifaddr in addrs {
//         if ifaddr.flags.contains(InterfaceFlags::IFF_LOOPBACK) {
//             match ifaddr.address {
//                 Some(SockAddr::Inet(InetAddr::V4(..))) => match family {
//                     AddressFamily::Inet => return Some(ifaddr),
//                     _ => continue,
//                 },
//                 Some(SockAddr::Inet(InetAddr::V6(..))) => match family {
//                     AddressFamily::Inet6 => return Some(ifaddr),
//                     _ => continue,
//                 },
//                 _ => continue,
//             }
//         }
//     }
//     None
// }
