use std::cmp::Ordering;
use std::process::exit;
use std::env;

mod auth;
mod ids;

struct Arguments {
    config: String, // path to config file
    interactive: bool, // show a password prompt?
    user: String, // default root.
    command: Vec<String> // The command to be executed
}

fn usage(verbose: bool, code: i32) {
    if verbose {
        println!("usage: dors [-Lns] [-C config] [-u user]");
        println!("-C config -- The config file");
        println!("-n -- Non-interactive mode. Will fail if password required");
        println!("-u -- The user. Default is root");
    } else {
        println!("usage: dors [-n] [-C config] [-u user]");
    }
    exit(code);
}

fn strip_arg(arg: &str) -> Result<char, String> {
    // Gets arguments of the form -a, returning a.
    if arg.len() < 2 {
        Err(String::from("Argument (e.g. -C) must be at least 2 chars long"))
    } else if &arg[0..1] != "-" {
        Err(String::from("Argument didn't start with -"))
    } else {
        return Ok(arg.chars().nth(1).unwrap());
    }
}

fn read_arguments(args: &Vec<String>) -> Result<Arguments, ()> {
    let mut config_file = String::from("/etc/dors.conf");
    let mut interactive: bool = true;
    let mut user: String = String::from("root");

    let mut current_arg_count: usize = 1; // skip binary
    'arg_loop: while current_arg_count < args.len() {
        match strip_arg(&args[current_arg_count]) {
            Err(_) => {
                // The argument didn't start with a - so we must be at the
                // command part now
                break 'arg_loop
            }
            Ok('u') => {
                current_arg_count += 1;
                if current_arg_count >= args.len() {
                    usage(false, 1);
                }
                user = String::from(&args[current_arg_count]);
            },
            Ok('C') => {
                current_arg_count += 1;
                if current_arg_count >= args.len() {
                    usage(false, 1);
                }
                config_file = String::from(&args[current_arg_count]);
            },
            Ok('n') => {
                interactive = false;
            },
            Ok(_) => {
                usage(false, 1);
            }
        }
        current_arg_count += 1;
    }
    let command: Vec<String> = args[current_arg_count..].to_vec();

    Ok(Arguments {
        config: config_file,
        interactive: interactive,
        user: user,
        command: command,
    })
}


fn exec_string_vec(command: Vec<String>) -> Result<std::process::Command, ()> {
    match command.len().cmp(&(1 as usize)) {
        Ordering::Less => {
            println!("Insuffiecient arguments");
            exit(1);
        },
        Ordering::Equal => {
            Ok(std::process::Command::new(&command[0]))
        },
        Ordering::Greater => {
            exec_string_vec(command[1..].to_vec())
        }

    }
}

fn debug_args(args: &Arguments) {
    println!("Config: {}", args.config);
    println!("Interactive: {}", args.interactive);
    println!("User: {}", args.user);
    println!("Command: {}", args.command.join(" "));
}


fn main() {
    let argvec: Vec<String> = env::args().collect();
    let mut args: Arguments = match read_arguments(&argvec) {
        Ok(x) => x,
        Err(_) => {
            usage(false, 1);
            exit(1)
        }
    };
    debug_args(&args);

    let path = match env::var("PATH") {
        Ok(path) => path,
        Err(_) => String::from("/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin:/usr/local/sbin")
    };

    let password = auth::get_password();
    println!("{}", password);
    match auth::verify_pass(&args.user, &password) {
        Err(_) => {
            println!("Password incorrect.");
            exit(1);
        },
        Ok(f) => f
    };

    ids::set_ids(&args.user);

    args.command.reverse();
    let spawned = match exec_string_vec(args.command) {
        Ok(f) => f,
        Err(_) => {
            println!("Couldn't format the command");
            exit(1);
        }
    }.env("PATH", path).spawn();

    let mut cmd = match spawned {
        Ok(t) => t,
        Err(e) => {
            usage(false, 1);
            println!("{}", e);
            exit(1)
        }
    };

    let _ = cmd.wait();
}
