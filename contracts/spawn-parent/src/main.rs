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

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::Unpack},
    error::SysError,
    syscalls,
};
use core::ffi::CStr;

fn run_sapwn(args: Vec<String>, fds: &[u64]) {
    let argc: u64 = args.len() as u64;

    let args: Vec<Vec<u8>> = args
        .iter()
        .map(|s| alloc::vec![s.as_bytes(), &[0u8]].concat())
        .collect();
    let argv: Vec<&CStr> = args
        .iter()
        .map(|s| CStr::from_bytes_until_nul(&s).unwrap())
        .collect();
    let argv2: Vec<*const i8> = argv.into_iter().map(|s| s.as_ptr()).collect();

    let mut pid: u64 = syscalls::process_id();
    let mut spgs: syscalls::SpawnArgs = syscalls::SpawnArgs {
        argc: argc,
        argv: argv2.as_ptr(),
        process_id: &mut pid as *mut u64,
        inherited_fds: fds.as_ptr(),
    };

    syscalls::spawn(0, Source::CellDep, 0, 0, &mut spgs).expect("spawn child");
}

fn spawn_base(_args: &[u8]) -> i8 {
    let spawn_args = alloc::vec!["0".to_string()];
    ckb_std::debug!("-A- VM Version: {}", syscalls::vm_version().unwrap());

    let mut std_fds: [u64; 2] = [0, 0];
    let mut son_fds: [u64; 3] = [0, 0, 0];
    let (r0, w0) = syscalls::pipe().expect("pipe 0");
    std_fds[0] = r0;
    son_fds[1] = w0;
    let (r1, w1) = syscalls::pipe().expect("pipe 1");
    std_fds[1] = w1;
    son_fds[0] = r1;

    run_sapwn(spawn_args, &son_fds);

    assert!(syscalls::close(std_fds[0]).is_ok());
    assert!(syscalls::close(std_fds[1]).is_ok());

    assert_eq!(syscalls::close(son_fds[0]), Err(SysError::InvalidFd));
    assert_eq!(syscalls::close(son_fds[1]), Err(SysError::InvalidFd));

    0
}

fn spawn_empty_pipe() -> i8 {
    let mut std_fds: [u64; 2] = [0, 0];
    let mut son_fds: [u64; 3] = [0, 0, 0];
    let (r0, w0) = syscalls::pipe().expect("pipe 0");
    std_fds[0] = r0;
    son_fds[1] = w0;
    let (r1, w1) = syscalls::pipe().expect("pipe 1");
    std_fds[1] = w1;
    son_fds[0] = r1;

    assert_eq!(std_fds[0], 2);
    assert_eq!(son_fds[1], 3);
    assert_eq!(son_fds[0], 4);
    assert_eq!(std_fds[1], 5);

    assert!(syscalls::close(std_fds[0]).is_ok());
    assert_eq!(syscalls::close(std_fds[0]), Err(SysError::InvalidFd));
    assert!(syscalls::close(std_fds[1]).is_ok());
    assert!(syscalls::close(son_fds[0]).is_ok());
    assert!(syscalls::close(son_fds[1]).is_ok());
    0
}

fn spawn_base_io(_args: &[u8]) -> i8 {
    let mut std_fds: [u64; 2] = [0, 0];
    let mut son_fds: [u64; 3] = [0, 0, 0];
    let (r0, w0) = syscalls::pipe().expect("pipe 0");
    std_fds[0] = r0;
    son_fds[1] = w0;
    let (r1, w1) = syscalls::pipe().expect("pipe 1");
    std_fds[1] = w1;
    son_fds[0] = r1;
    let mut pid: u64 = 0;

    ckb_std::debug!("-A- std fds: {}, {}", std_fds[0], std_fds[1]);
    ckb_std::debug!("-A- son fds: {}, {}", son_fds[0], son_fds[1]);

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
    ckb_std::debug!("-A- Spawn --");
    syscalls::spawn(0, Source::CellDep, 0, 0, &mut spgs).expect("spawn 1");
    ckb_std::debug!("-A- Spawn End, pid: {} --", pid);

    ckb_std::debug!("-A- Read --");
    let mut buf: [u8; 256] = [0; 256];
    let len = syscalls::read(std_fds[0], &mut buf).expect("read 1");
    ckb_std::debug!("-A- Read End --");

    // syscalls::close(pid).expect("close non code");

    let mut std_fds2: [u64; 2] = [0, 0];
    let mut son_fds2: [u64; 3] = [0, 0, 0];
    let (r0, w0) = syscalls::pipe().expect("pipe 0");
    std_fds2[0] = r0;
    son_fds2[1] = w0;
    let (r1, w1) = syscalls::pipe().expect("pipe 1");
    std_fds2[1] = w1;
    son_fds2[0] = r1;
    let mut pid2 = syscalls::process_id();
    let mut spgs: syscalls::SpawnArgs = syscalls::SpawnArgs {
        argc: argc,
        argv: argv.as_ptr(),
        process_id: &mut pid2 as *mut u64,
        inherited_fds: son_fds2.as_ptr(),
    };
    ckb_std::debug!("-A- Spawn --");
    syscalls::spawn(0, Source::CellDep, 0, 0, &mut spgs).expect("spawn 1");
    ckb_std::debug!("-A- Spawn End, pid: {} --", pid2);

    assert_eq!(len, 10);
    buf[len] = 0;
    assert_eq!(
        CStr::from_bytes_until_nul(&buf).unwrap().to_str().unwrap(),
        "helloworld"
    );

    ckb_std::debug!("-A- Spawn-Parent Exit");
    0
}

pub fn program_entry() -> i8 {
    ckb_std::debug!("-A- Spawn-Parent(pid:{}) Begin --", syscalls::process_id());

    let args = {
        let script = ckb_std::high_level::load_script().expect("Load script");
        let args: Bytes = script.args().unpack();
        args.to_vec()
    };
    assert!(args.len() >= 1, "args is empty");

    let cmd = args[0];
    let args = args[1..].to_vec();

   let rc = match cmd {
        0 => spawn_base(&args),
        1 => spawn_empty_pipe(),
        2 => spawn_base_io(&args),
        _ => {
            panic!("unknow command: {}", cmd);
        }
    };

    ckb_std::debug!("-A- Spawn-Parent(pid:{}) End --", syscalls::process_id());
    rc
}
