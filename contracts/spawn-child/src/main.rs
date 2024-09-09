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

use alloc::vec;
use ckb_std::env::Arg;
use ckb_std::syscalls;

fn spawn_base() -> i8 {
    let mut std_fds = [0u64; 2];
    syscalls::inherited_fds(&mut std_fds);
    assert_eq!(std_fds[0], 4);
    assert_eq!(std_fds[1], 3);

    let mut std_fds2 = [0u64; 3];
    syscalls::inherited_fds(&mut std_fds2);
    assert_eq!(std_fds2[0], 4);
    assert_eq!(std_fds2[1], 3);
    assert_eq!(std_fds2[2], 0);

    0
}

fn spawn_base_io(argv: &[Arg]) -> i8 {
    let mut std_fds: [u64; 2] = [0; 2];
    ckb_std::debug!("-B- InheritedFds --");
    syscalls::inherited_fds(&mut std_fds);
    ckb_std::debug!("-B- InheritedFds {} {} End --", std_fds[0], std_fds[1]);

    let mut out = vec![];
    for arg in argv {
        out.extend_from_slice(arg.to_bytes());
    }

    // ckb_std::debug!("-B- Read --");
    // let mut buf: [u8; 256] = [0; 256];
    // syscalls::read(std_fds[0], &mut buf).expect("child read");
    // ckb_std::debug!("-B- Read End --");

    ckb_std::debug!("-B- Write --");
    let len = syscalls::write(std_fds[1], &out).expect("child write");
    ckb_std::debug!("-B- Write End --");
    assert_eq!(len, 10);

    ckb_std::debug!("-B- Spawn-Child Exit");
    0
}

pub fn program_entry() -> i8 {
    ckb_std::debug!("-B- Spawn-Child(pid:{}) Begin --", syscalls::process_id());

    let argv = ckb_std::env::argv();
    assert!(argv.len() >= 1, "child args is failed: {}", argv.len());

    let cmd = u8::from_str_radix(argv[0].to_str().unwrap(), 10).expect("parse cmd");

    let rc = match cmd {
        0 => spawn_base(),
        1 => spawn_base_io(argv),
        _ => {
            panic!("unknow cmd: {}", cmd);
        }
    };

    ckb_std::debug!("-B- Spawn-Child(pid:{}) End --", syscalls::process_id());
    rc
}
