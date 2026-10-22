use users::{get_user_by_uid, get_current_uid};
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
