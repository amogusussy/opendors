use nix::libc::{self, getpwnam};
use nix::unistd::{self, Uid, User};
use std::ffi::CString;

#[derive(Debug)]
pub enum IdError {
    NotDetermineCurUser,
    NotSetIds,
    NotGetIds
}

pub fn get_username() -> Result<String, IdError> {
    let uid = get_uid();
    match User::from_uid(unistd::Uid::from(uid)) {
        Ok(t) => match t {
            Some(x) => Ok(x.name),
            None => Err(IdError::NotDetermineCurUser)
        },
        Err(_) => Err(IdError::NotDetermineCurUser)
    }
}

pub fn get_uid() -> Uid {
    unistd::getuid()
}

pub fn get_ids_from_name(name: &str) -> Result<(u32, u32), IdError> {
    let s = CString::new(name).unwrap();
    let uid: u32;
    let gid: u32;
    unsafe {
        let p = getpwnam(s.as_ptr());
        if p == std::ptr::null_mut() {
            return Err(IdError::NotGetIds);
        }
        uid = (*p).pw_uid;
        gid = (*p).pw_gid;
    };
    Ok((uid, gid))
}

pub fn set_ids(username: &String) -> Result<(i32, i32), IdError> {
    let (uid, gid) = get_ids_from_name(&username)?;
    let r1;
    let r2;
    unsafe {
        r1 = libc::setresgid(gid, gid, gid);
        r2 = libc::setresuid(uid, uid, uid);
    };

    if (r1 != 0) || (r2 != 0) {
        println!("{}, {}, g: {}, u: {}", r1, r2, gid, uid);
        Err(IdError::NotSetIds)
    } else {
        Ok((r1, r2))
    }
}
