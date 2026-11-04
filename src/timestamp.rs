use std::{
    fs::{self, File, create_dir, exists}, io::{Read, Write}, os::unix::fs::PermissionsExt, process, time::{SystemTime, UNIX_EPOCH}
};


const PROTECTED_FILE_PERMISSIONS: u32 = 0o644;


#[derive(Debug)]
pub enum TimestampErrors {
    TsNotExists,
    BrokenTsFile,
    CouldNotWrite,
    OSErr,
}

#[derive(Debug)]
pub struct TimeStampCookie {
    pub ts: u64, // prevent 32-bit timestamp bug in 10+ years
    pub file_name: String,
    pub pid: u32,
}

pub fn get_now() -> Result<u64, ()> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => Ok(n.as_secs_f64().round() as u64),
        Err(_) => return Err(())
    }
}


impl TimeStampCookie {
    // Scheme for writing/reading the cookie:
    // UNIX_TIMESTAMP,FILE_NAME,PID
    pub fn from_user(user_id: u32) -> Result<TimeStampCookie, TimestampErrors> {
        let file_path: String = format!("/run/dors/{}", user_id);
        let mut fp = match File::open(&file_path) {
            Ok(file) => file,
            Err(_) => return Err(TimestampErrors::TsNotExists)
        };
        // TODO: Make sure  file_path is owned by root

        let mut buf = String::new();
        match fp.read_to_string(&mut buf) {
            Ok(_) => {},
            Err(_) => return Err(TimestampErrors::BrokenTsFile)
        };

        let mut ts: u64 = 0;
        let mut file_path: String = String::new();
        let mut pid: u32 = 0;


        if let Some(first_comma) = buf.find(',') {
            if let Some(second_comma) = buf.rfind(',') {
                if first_comma >= second_comma {
                    return Err(TimestampErrors::BrokenTsFile)
                }
                ts = match buf[..first_comma].parse() {
                    Ok(ts) => ts,
                    Err(_) => {
                        return Err(TimestampErrors::BrokenTsFile)
                    }
                };
                file_path = String::from(&buf[first_comma + 1..second_comma]);
                pid = match buf[second_comma+1..].parse() {
                    Ok(pid) => pid,
                    Err(_) => {
                        return Err(TimestampErrors::BrokenTsFile)
                    }
                };
            }
        }

        Ok(TimeStampCookie {
            ts: ts,
            file_name: file_path,
            pid: pid,
        })
    }


    pub fn new(user_id: u32) -> Result<TimeStampCookie, ()> {
        let file_path: String = format!("/run/dors/{}", user_id);
        let now = get_now()?;

        let pid = process::id();

        Ok(TimeStampCookie {
            ts: now,
            file_name: file_path,
            pid: pid
        })
    }

    pub fn write(&self) -> Result<(), TimestampErrors> {
        if !exists(&self.file_name).unwrap() {
            match create_dir("/run/dors") {
                Ok(_) => {},
                Err(_) => {
                    return Err(TimestampErrors::OSErr);
                }
            };
            File::create(&self.file_name).unwrap();
        };

        let meta = match fs::metadata(&self.file_name) {
            Ok(meta) => meta,
            Err(_) => return Err(TimestampErrors::OSErr),
        };

        // if not root rw only, then make root rw only.
        let protected_permissions = fs::Permissions::from_mode(PROTECTED_FILE_PERMISSIONS);
        if !meta.permissions().eq(&protected_permissions) {
            match fs::set_permissions(&self.file_name, protected_permissions) {
                Ok(_) => {},
                Err(_) => return Err(TimestampErrors::OSErr)
            };
        }

        let mut fp = match File::create(&self.file_name) {
            Ok(fp) => fp,
            Err(_) => return Err(TimestampErrors::TsNotExists),
        };

        let line = String::from(format!("{},{},{}", self.ts, self.file_name, self.pid));
        match fp.write(&(line.as_bytes())) {
            Ok(_) => {},
            Err(_) => {
                return Err(TimestampErrors::CouldNotWrite);
            }
        };

        Ok(())

    }
}
