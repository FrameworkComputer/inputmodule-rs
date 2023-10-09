mod dbus_monitor;

extern crate log;
use env_logger;

fn main() {
    env_logger::init();
    dbus_monitor::run_dbus_monitor();
}
