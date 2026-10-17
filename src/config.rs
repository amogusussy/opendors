use std::fs::{File, metadata};
use std::io::Read;
use nix::unistd;
use std::os::linux::fs::MetadataExt;
use toml::{self, Table, Value};
use crate::ids;

#[derive(Debug, Clone)]
pub struct EnvPair {
    pub name: String,
    pub value: String
}

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub is_group: bool,
    pub permitted_as: Vec<String>,
    pub pass: bool,
    pub log: bool,
    pub persist: bool,
    pub persist_len: u32,
    pub keep_env: Vec<String>,
    pub custom_env: Vec<EnvPair>
}

pub struct Config {
    entries: Vec<Entry>,
    path: String,
}

fn get_owner(file_path: &str) -> Result<u32, &str> {
    match metadata(file_path) {
        Ok(meta) => Ok(meta.st_gid()),
        Err(_) => Err("Could not determine file owner")
    }
}

fn get_string_array(value: Option<&Value>) -> Vec<String> {
    let mut result = Vec::new();
    let inner = match value {
        Some(n) => n,
        None => return result
    };
    let arr = match inner.as_array() {
        Some(arr) => arr,
        _ => {return result}
    };
    for val in arr {
        if val.is_str() {
            let s = match val.as_str() {
                None => continue,
                Some(s) => s
            };
            result.push(String::from(s));
        }
    }

    result
}

fn read_entry(name: &str, entry: &Value) -> Entry {
    let permitted_as = get_string_array(entry.get("permitted_as"));
    let not_permitted_as = get_string_array(entry.get("not_permitted_as"));
    let keep_env = get_string_array(entry.get("keep_env"));

    let log = entry.get("log").and_then(Value::as_bool).unwrap_or(false);
    let is_group = entry.get("is_group").and_then(Value::as_bool).unwrap_or(false);
    let pass = entry.get("pass").and_then(Value::as_bool).unwrap_or(false);
    let persist = entry.get("persist").and_then(Value::as_bool).unwrap_or(false);

    let persist_len = match entry.get("persist").and_then(Value::as_integer) {
        Some(n) => {
            if persist { n as u32 } else { 0 }
        },
        _ => 0
    };

    let custom_env: Vec<EnvPair> = match entry
        .get("custom_env")
        .and_then(Value::as_array) {
            Some(t) => {
                let mut v = Vec::new();
                for var in t {
                    if !var.is_table() {
                        continue
                    }
                    let q = match var.as_table() {
                        Some(q) => q,
                        _ => continue
                    };
                    let name = match q.get("name") {
                        Some(name) => match name.as_str() {
                            Some(s) => s,
                            _ => continue
                        },
                        _ => continue
                    };
                    let value = match q.get("value") {
                        Some(value) => match value.as_str() {
                            Some(s) => s,
                            _ => continue
                        },
                        _ => continue
                    };

                    v.push(EnvPair {
                        name: String::from(name),
                        value: String::from(value)
                    });
                };
                v
            },
            None => Vec::new()
    };
    Entry {
        name: String::from(name),
        is_group: is_group,
        permitted_as: permitted_as,
        pass: pass,
        log: log,
        persist: persist,
        persist_len: persist_len,
        keep_env: keep_env,
        custom_env: custom_env
    }
}

pub fn read_config(config_file: &str) -> Result<Vec<Entry>, String> {
    let mut fp = match File::open(config_file) {
        Ok(f) => f,
        Err(_) => return Err(String::from("Could not open the file."))
    };
    let owner = match get_owner(config_file) {
        Ok(owner) => owner,
        Err(_) => {
            return Err(String::from("Could not determine owner of config file"));
        }
    };
    if owner != 0 {
        // not root
        return Err(String::from("File doesn't belong to root"))
    };
    let mut content = String::new();
    match fp.read_to_string(&mut content) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("Could not read config file."))
        }
    };
    let config = match content.parse::<Table>() {
        Ok(config) => config,
        Err(e) => {
            let msg = format!("Could not parse the config file\n{e}");
            return Err(msg);
        }
    };
    let mut output = Vec::new();
    for (name, value) in config.iter() {
        if value.is_table() {
            let entry = read_entry(name, &value);
            output.push(entry);
        }
    };

    Ok(output)
}
