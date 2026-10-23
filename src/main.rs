use std::cmp::Ordering;
use std::process::{Command, exit};
use std::{env, os};

use nix::libc::getenv;

use crate::config::EnvPair;

mod auth;
mod ids;
mod config;
mod permissions;

#[derive(Debug)]
struct Arguments {
    config: String, // path to config file
    interactive: bool, // show a password prompt?
    shell: bool, // Whether to replace the command with $SHELL
    user: String, // default root.
    command: Vec<String>, // The command to be executed
}

fn usage(verbose: bool, code: i32) {
    if verbose {
        println!("usage: dors [-nus] [-C config] [-u user] [command]");
        println!("-C config -- The config file");
        println!("-n -- Non-interactive mode. Will fail if password required");
        println!("-u -- The user. Default is root");
        println!("-s -- Replace the command with $SHELL");
    } else {
        println!("usage: dors [-nus] [-C config] [-u user] [command]");
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
    let mut config_file = String::from("/etc/dors.toml");
    let mut interactive: bool = true;
    let mut user: String = String::from("root");
    let mut shell: bool = false;

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
            Ok('s') => {
                shell = true;
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
        shell: shell,
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

fn get_envvars(perms: &permissions::Permissions) -> Vec<EnvPair> {
    let mut vars = perms.custom_env.clone();
    for key in &perms.keep_env {
        let val = match std::env::var(key) {
            Ok(val) => val,
            Err(_) => continue
        };
        vars.push(EnvPair {
            name: key.into(),
            value: val
        })
    }
    vars
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
    //debug_args(&args);
    let config_opts = match config::read_config(&args.config) {
        Ok(opts) => opts,
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
    };

    let perms = match permissions::get_most_relevant_config(&config_opts) {
        Ok(p) => p,
        Err(_) => {
            println!("Could not determine permissions");
            exit(1);
        }
    };
    if !permissions::can_be_user(&args.user, &perms) {
        println!("You don't have permissions to be this user. Check {} for more info", args.config);
        exit(1);
    }

    let path = match env::var("PATH") {
        Ok(path) => path,
        Err(_) => String::from("/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin:/usr/local/sbin")
    };

    let cur_user = match auth::get_current_username() {
        Ok(user) => user,
        Err(_) => {
            println!("Could not get current user.");
            exit(1);
        }
    };
    ids::set_ids(&args.user);
    if perms.pass { 
        let password = auth::get_password(&format!("[dors] Password for {}: ", cur_user));
        match auth::verify_pass(&cur_user, &password) {
            Err(e) => {
                println!("Password incorrect. {}", e);
                exit(1);
            },
            Ok(f) => {
                f
            }
        };
    }

    if !args.interactive && perms.pass {
        println!("You must enter a password. Non-interactive mode wont work.");
        exit(1);
    }

    if args.shell {
        let shell = match auth::get_shell() {
            Ok(shell) => shell,
            Err(_) => env::var("SHELL").unwrap()
        };
        args.command = vec![shell];
    }


    args.command.reverse();

    let envvars = get_envvars(&perms);
    let spawned = match exec_string_vec(args.command) {
        Ok(f) => f,
        Err(_) => {
            println!("Couldn't format the command");
            exit(1);
        }
    }
    .env_clear()
    .env("PATH", path)
    .envs(envvars.iter().map(|e| (&e.name, &e.value)))
    .spawn();

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
