mod dbus_monitor;
mod utils;

extern crate log;
use log::{debug};
use env_logger;

fn main() {
    env_logger::init();
    dbus_monitor::run_dbus_monitor();
}
