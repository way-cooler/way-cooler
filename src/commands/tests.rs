//! Tests on the command API
use std::sync::{Arc, Mutex, Condvar};

use commands::{self, CommandFn};

lazy_static! {
    static ref ACCESS_PAIR: Arc<(Mutex<bool>, Condvar)>
        = Arc::new((Mutex::new(false), Condvar::new()));
}

// Commands for the commands tests

#[test]
pub fn command() {
    println!("command being run!");
}

pub fn panic_command() {
    panic!("panic_command panic")
}

/// Wait for the registry to initialize
pub fn wait_for_commands() {
    // First star for lazy static, second for Arc
    let &(ref lock, ref cond) = &**ACCESS_PAIR;
    let mut started = lock.lock().expect("Unable to lock ACCESS_PAIR lock!");
    while !*started {
        started = cond.wait(started)
            .expect("Oh boy, I can't wait for the commands to start!");
    }
}

#[test]
fn add_commands() {
    // Command
    assert!(commands::set("command".to_string(), Arc::new(command))
        .is_none(), "Unable to initialize command in registry");

    // Panicking command
    assert!(commands::set("panic_command".to_string(), Arc::new(panic_command))
        .is_none(), "Unable to initialize command in registry");

   // Allow waiting threads to continue
    let &(ref lock, ref cond) = &**ACCESS_PAIR;
    let mut started = lock.lock().expect("Couldn't unlock threads");
    *started = true;
    cond.notify_all();
}

#[test]
fn get_command() {
    wait_for_commands();
    assert!(commands::get("command").is_some(), "Could not get command!");
}

#[test]
fn run_command() {
    wait_for_commands();
    let command = commands::get("command")
        .expect("Command not found");
    command();
}

#[test]
#[should_panic(expected = "command panic")]
fn run_panic_command() {
    wait_for_commands();
    let command = commands::get("panic_command")
        .expect("Command not found");
    command();
}
