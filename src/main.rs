mod module;
mod http_server;
mod keystore;
mod http_handler;
mod config;
mod schedule;

use config::Config;
use signal_hook::{consts::{SIGINT, SIGTERM}, iterator::Signals};

fn main() {
    let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
    let config = Config::from_args();
    
    let http_server = http_server::start(config.clone());
    let schedule = schedule::start(http_server.clone_mgmt_sender());

    for sig in signals.forever() {
        match sig {
            SIGINT | SIGTERM => {
                schedule.stop();
                http_server.stop();

                schedule.join();
                http_server.join();
                break;
            },
            _ => {}
        }
    }
}
