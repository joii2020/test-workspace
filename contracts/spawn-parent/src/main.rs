#![cfg_attr(not(feature = "native-simulator"), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "native-simulator", test))]
extern crate alloc;

#[cfg(not(any(feature = "native-simulator", test)))]
use ckb_std::default_alloc;
#[cfg(not(any(feature = "native-simulator", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "native-simulator", test)))]
default_alloc!();

use ckb_std::ckb_constants::Source;
use ckb_std::syscalls;
use core::ffi::CStr;

pub fn program_entry() -> i8 {
    ckb_std::debug!("This is a sample contract spawn-parent!");

    let mut std_fds: [u64; 2] = [0, 0];
    let mut son_fds: [u64; 3] = [0, 0, 0];
    let (r0, w0) = syscalls::pipe().expect("pipe 0");
    std_fds[0] = r0;
    son_fds[1] = w0;
    let (r1, w1) = syscalls::pipe().expect("pipe 1");
    std_fds[1] = w1;
    son_fds[0] = r1;
    let mut pid: u64 = 0;

    let argc: u64 = 2;
    let argv = [
        CStr::from_bytes_with_nul(b"hello\0").unwrap().as_ptr(),
        CStr::from_bytes_with_nul(b"world\0").unwrap().as_ptr(),
    ];
    let mut spgs: syscalls::SpawnArgs = syscalls::SpawnArgs {
        argc: argc,
        argv: argv.as_ptr(),
        process_id: &mut pid as *mut u64,
        inherited_fds: son_fds.as_ptr(),
    };
    syscalls::spawn(1, Source::CellDep, 0, 0, &mut spgs).expect("spawn 1");
    let mut buf: [u8; 256] = [0; 256];
    let len = syscalls::read(std_fds[0], &mut buf).expect("read 1");
    assert_eq!(len, 10);
    buf[len] = 0;
    assert_eq!(
        CStr::from_bytes_until_nul(&buf).unwrap().to_str().unwrap(),
        "helloworld"
    );
    0
}
