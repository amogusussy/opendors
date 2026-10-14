use nix::libc::{self, getpwnam};
use std::process::exit;
use std::os::unix::fs::MetadataExt;
use std::ffi::{CString};

pub fn get_uid() -> u32 {
    // This is only tested on a GNU/Linux system,
    // so might not work elsewhere.
    // taken from: https://stackoverflow.com/a/57952799
    match std::fs::metadata("/proc/self").map(|m| m.uid()) {
        Err(_) => {
            println!("Could not determine uid.");
            exit(1);
        },
        Ok(f) => f
    }
}

pub fn get_uid_from_name(name: &str) -> (u32, u32) {
    let s = CString::new(name).unwrap();
    let uid: u32;
    let gid: u32;
    unsafe {
        let p = getpwnam(s.as_ptr());
        if p == std::ptr::null_mut() {
            println!("Error: User {} doesn't exist", name);
            exit(1);
        }
        uid = (*p).pw_uid;
        gid = (*p).pw_gid;
    };
    (uid, gid)
}

pub fn get_euid() -> u32 {
    unsafe { libc::geteuid() }
}

pub fn set_ids(username: &String) {
    let (uid, gid) = get_uid_from_name(&username);
    unsafe {
        libc::setresgid(gid, gid, gid);
        libc::setresuid(uid, uid, uid);
    }
}

