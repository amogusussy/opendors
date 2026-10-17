use nix::libc::{self, getpwnam};
use nix::unistd::{self, Gid, Uid, User};
use std::process::exit;
use std::os::unix::fs::MetadataExt;
use std::ffi::{CStr, CString};

pub fn get_username() -> Result<String, String> {
    let uid = get_uid();
    match User::from_uid(unistd::Uid::from(uid)) {
        Ok(t) => match t {
            Some(x) => Ok(x.name),
            None => Err(String::from("Could not determine current user"))
        },
        Err(_) => Err(String::from("Could not determine current user"))
    }
}

pub fn get_uid() -> Uid {
    unistd::getuid()
}

pub fn get_ids_from_name(name: &str) -> (u32, u32) {
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
    let (uid, gid) = get_ids_from_name(&username);
    unsafe {
        libc::setresgid(gid, gid, gid);
        libc::setresuid(uid, uid, uid);
    }
}
