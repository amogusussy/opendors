use users::{get_user_by_uid, get_current_uid };
use users::os::unix::UserExt;
use std::path::Path;
use pam::{Authenticator, PamError};
use rpassword;


pub fn get_current_username() -> Result<String, String> {
    let user = get_user_by_uid(get_current_uid()).unwrap();
    let username = user.name().to_str();
    match username {
        Some(name) => return Ok(String::from(name)),
        _ => return Err(String::from("Err"))
    }
}

#[cfg(target_os = "linux")]
pub fn get_shell() -> Result<String, String> {
    let user = get_user_by_uid(get_current_uid()).unwrap();
    let shell_path = match user.shell().to_str() {
        Some(shell) => shell,
        None => {return Err(String::from("No shell found"))}
    };
    Ok(String::from(shell_path))
}

pub fn get_password(prompt: &str) -> String {
    rpassword::prompt_password(prompt).unwrap()
}

pub fn verify_pass(username: &String, password: &String) -> Result<(), PamError> {
    let mut auth = match Authenticator::with_password("dors") {
        Ok(auth) => auth,
        Err(s) => {
            return Err(s);
        }
    };

    auth.get_handler().set_credentials(username, password);
    let t = auth.authenticate();
    t
}
