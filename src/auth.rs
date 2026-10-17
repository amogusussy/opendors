use pam::{Authenticator, PamError};
use rpassword;

pub fn get_password() -> String {
    rpassword::prompt_password("Your password: ").unwrap()
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
