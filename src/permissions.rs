use crate::config::{Entry, EnvPair};
use nix::unistd;
use nix::unistd::{getuid, User, getgrouplist, Group};
use std::ffi::CString;
use crate::ids;

#[derive(Debug)]
pub struct Permissions {
    pub permitted_as: Vec<String>,
    pub pass: bool,
    pub log: bool,
    pub persist: bool,
    pub persist_len: u32,
    pub keep_env: Vec<String>,
    pub custom_env: Vec<EnvPair>
}

fn current_user_groups() -> Result<Vec<String>, String> {
    let user = match User::from_uid(getuid()) {
        Ok(user) => match user {
            Some(u) => u,
            _ => return Err(String::from("Couldn't get user groups."))
        },
        Err(_) => return Err(String::from("Couldn't get user groups."))
    };
    let name = match CString::new(user.name.as_str()) {
        Ok(name) => name,
        Err(_) => return Err(String::from("Couldn't get user groups."))
    };

    let gids = match getgrouplist(&name, user.gid) {
        Ok(gids) => gids,
        Err(_) => return Err(String::from("Couldn't get user groups."))
    };

    let mut names = Vec::with_capacity(gids.len());
    for gid in gids {
        let group = match Group::from_gid(gid) {
            Ok(group) => group,
            Err(_) => return Err(String::from("Couldn't get user groups."))
        };
        let group_name = match group {
            Some(g) => g.name,
            _ => continue
        };
        names.push(group_name);
    }
    Ok(names)
}

pub fn get_most_relevant_config(config: &Vec<Entry>) -> Result<Permissions, String> {
    // Get the most relevant config options for the user.
    // E.g. if user in wheel, and wheel.log = true, then
    // the config should inherit this.
    if config.len() == 0 {
        return Err(String::from("No config options"))
    }
    let username = ids::get_username()?;
    let groups = match current_user_groups() {
        Ok(g) => g,
        Err(_) => return Err(String::from("You're in no groups :("))
    };
    // 1 << 31 as fall back 
    let mut most_relevant_idx = 1 << 31;
    let seen_user = false;

    for (idx, config_entry) in config.iter().enumerate() {
        if config_entry.is_group && groups.contains(&config_entry.name) && !seen_user {
            most_relevant_idx = idx;
            continue
        }
        if !config_entry.is_group && (username == config_entry.name) {
            most_relevant_idx = idx;
            break;
        }
    };

    if most_relevant_idx == (1 << 31) {
        Err(String::from("No relevant config options."))
    } else {
        Ok(Permissions {
            permitted_as: config[most_relevant_idx].permitted_as.clone(),
            pass: config[most_relevant_idx].pass,
            log: config[most_relevant_idx].log,
            persist: config[most_relevant_idx].persist,
            persist_len: config[most_relevant_idx].persist_len,
            keep_env: config[most_relevant_idx].keep_env.clone(),
            custom_env: config[most_relevant_idx].custom_env.clone()
        })
    }
}

pub fn can_be_user(target_user: &str, perms: &Permissions) -> bool {
    let mut all_allowed = false;
    for i in &perms.permitted_as {
        if i == "*" {
            all_allowed = true;
            break;
        }
    }
    all_allowed || perms.permitted_as.contains(&String::from(target_user))
}
