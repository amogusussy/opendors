use pam;
use rpassword;

pub fn get_password() -> String {
    rpassword::prompt_password("Your password: ").unwrap()
}

pub fn verify_pass(username: &String, password: &String) -> Result<bool, ()> {
    let mut auth = match pam::Authenticator::with_password("dors") {
        Ok(auth) => auth,
        Err(_) => {
            return Err(());
        }
    };

    auth.get_handler().set_credentials(username, password);
    Ok(auth.authenticate().is_ok())
}
