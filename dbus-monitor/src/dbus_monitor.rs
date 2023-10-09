// Mostly taken from https://github.com/diwic/dbus-rs/blob/366a6dca3d20745f5dcfa006b1b1311c376d420e/dbus/examples/monitor.rs

// This programs implements the equivalent of running the "dbus-monitor" tool
// modified to only search for messages in the interface specificied in config.json,
// and then run arbitary inputmodule-rs commands to react to them

use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;

use dbus::Message;
use dbus::MessageType;

use std::time::Duration;

use clap::Parser;
use inputmodule_control::commands::ClapCli;
use inputmodule_control::inputmodule::serial_commands;

use log::debug;

use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Read;

use lazy_static::lazy_static;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    dbus_interface: String,
    dbus_member: String,
    scan_args_for: String,
    run_inputmodule_commands: Vec<String>,
}

fn read_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut config_data = String::new();
    file.read_to_string(&mut config_data)?;

    let config: Config = serde_json::from_str(&config_data)?;

    Ok(config)
}

lazy_static! {
    pub static ref CONFIG: Config = {
        // Read and deserialize the JSON configuration
        let config_file = "dbus-monitor/src/config.json";
        let config = read_config(config_file).expect("Failed to read config");
        config
    };
}

fn handle_message(msg: &Message) {
    debug!("Got message from DBus: {:?}", msg);

    let mut iter = msg.iter_init();
    while let Some(arg) = iter.get_refarg() {
        if let Some(string_ref) = arg.as_str() {
            let string_value: String = string_ref.to_string();
            debug!("String value: {}", string_value);

            if string_value.contains(&CONFIG.scan_args_for) {
                for command in &CONFIG.run_inputmodule_commands {
                    let command_vec: Vec<&str> = command.split_whitespace().collect();
                    run_inputmodule_command(command_vec);
                }
            }
        }
        iter.next();
    }

    debug!("DBus Message handled");
}

pub fn run_inputmodule_command(args: Vec<&str>) {
    let bin_placeholder = vec!["bin-placeholder"];
    let full_args = [&bin_placeholder[..], &args[..]].concat();
    let args = ClapCli::parse_from(full_args);

    serial_commands(&args);
}

pub fn run_dbus_monitor() {
    // First open up a connection to the desired bus.
    let conn = Connection::new_session().expect("D-Bus connection failed");
    debug!("Connection to DBus session monitor opened");

    // Second create a rule to match messages we want to receive
    let rule = MatchRule::new()
        .with_type(MessageType::MethodCall)
        .with_interface(&CONFIG.dbus_interface)
        .with_member(&CONFIG.dbus_member);

    // Try matching using new scheme
    let proxy = conn.with_proxy(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        Duration::from_millis(5000),
    );
    let result: Result<(), dbus::Error> = proxy.method_call(
        "org.freedesktop.DBus.Monitoring",
        "BecomeMonitor",
        (vec![rule.match_str()], 0u32),
    );
    debug!("Monitoring DBus channel...");

    match result {
        // BecomeMonitor was successful, start listening for messages
        Ok(_) => {
            conn.start_receive(
                rule,
                Box::new(|msg, _| {
                    debug!("Start listening");
                    handle_message(&msg);
                    true
                }),
            );
        }
        // BecomeMonitor failed, fallback to using the old scheme
        Err(e) => {
            debug!(
                "Failed to BecomeMonitor: '{}', falling back to eavesdrop",
                e
            );

            // First, we'll try "eavesdrop", which as the name implies lets us receive
            // *all* messages, not just ours.
            let rule_with_eavesdrop = {
                let mut rule = rule.clone();
                rule.eavesdrop = true;
                rule
            };

            let result = conn.add_match(rule_with_eavesdrop, |_: (), _, msg| {
                handle_message(&msg);
                true
            });

            match result {
                Ok(_) => {
                    // success, we're now listening
                }
                // This can sometimes fail, for example when listening to the system bus as a non-root user.
                // So, just like `dbus-monitor`, we attempt to fallback without `eavesdrop=true`:
                Err(e) => {
                    debug!("Failed to eavesdrop: '{}', trying without it", e);
                    conn.add_match(rule, |_: (), _, msg| {
                        handle_message(&msg);
                        true
                    })
                    .expect("add_match failed");
                }
            }
        }
    }

    // Loop and print out all messages received (using handle_message()) as they come.
    // Some can be quite large, e.g. if they contain embedded images..
    loop {
        conn.process(Duration::from_millis(1000)).unwrap();
    }
}
