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

use ckb_std::syscalls;
use alloc::vec;

pub fn program_entry() -> i8 {
    ckb_std::debug!("This is a sample contract spawn-child!");

    let argv = ckb_std::env::argv();
    let mut std_fds: [u64; 2] = [0; 2];
    syscalls::inherited_file_descriptors(&mut std_fds);
    let mut out = vec![];
    for arg in argv {
        out.extend_from_slice(arg.to_bytes());
    }
    let len = syscalls::write(std_fds[1], &out).expect("child write");
    assert_eq!(len, 10);
    0
}
