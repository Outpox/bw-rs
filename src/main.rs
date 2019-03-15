#[macro_use]
extern crate clap;

use std::fs::File;
use std::io::prelude::*;
use std::io::{SeekFrom, ErrorKind};
use std::thread::sleep;
use std::time::Duration;
use std::u64;
use clap::{Arg, App};

const DEFAULT_INTERFACE: &str = "eth0";
const DEFAULT_INTERVAL: u64 = 1000;

fn main() {
    let matches = App::new("bw-rs-cli")
                    .author("Outpox, contact@guillaumemigeon.fr")
                    .version("1.0.0")
                    .arg(Arg::with_name("interface")
                        .short("i")
                        .long("interface")
                        .value_name("INTERFACE")
                        .help("Sets the interface you want to listen to.")
                        .required(true)
                        .takes_value(true))
                    .arg(Arg::with_name("direction")
                        .short("d")
                        .long("direction")
                        .value_name("DIRECTION")
                        .default_value("rx")
                        .possible_values(&["rx", "tx"])
                        .takes_value(true)
                        .help("Specify the data you want. Either received 'RX' or transmitted 'TX'."))
                    .arg(Arg::with_name("interval")
                        .short("r")
                        .long("interval")
                        .value_name("INTERVAL")
                        .default_value("1000")
                        .takes_value(true)
                        .help("Set the polling rate. Default 1000ms (1 output/s)."))
                    .get_matches();

    let interval = value_t!(matches, "interval", u64).unwrap_or(DEFAULT_INTERVAL);
    let interface = matches.value_of("interface").unwrap_or(DEFAULT_INTERFACE);
    let direction = match matches.value_of("direction").unwrap() {
        "rx" => Stats::RX,
        "tx" => Stats::TX,
        _ => unreachable!(),
    };

    dbg!(interval);
    dbg!(interface);
    dbg!(&direction);

    read_file(interface, direction, interval);
}

#[derive(Debug)]
enum Stats {
    RX,
    TX,
}

fn read_file(interface: &str, stat: Stats, interval: u64) {
    let mut buf = String::new();
    let mut p: String;

    let mut bufi: u64;
    let mut pi: u64;

    let path = get_path(interface, stat);
    // let mut file = File::open(path).expect("Error reading file");
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            dbg!(&e);
            if e.kind() == ErrorKind::NotFound {
                panic!("Interface not found".to_string());
            } else {
                panic!("{}", e);
            }
        }
    };

    loop {
        p = buf.clone();
        buf.clear();
        file.read_to_string(&mut buf).unwrap();
        // Set the cursor back to the beginning of the file
        file.seek(SeekFrom::Start(0)).unwrap();

        bufi = buf.trim().parse().unwrap();
        pi = p.trim().parse().unwrap_or(0);

        if pi != 0 {
            println!("{}", get_speed(bufi - pi));
        }

        sleep(Duration::from_millis(interval));
    }
}

fn get_path(interface_id: &str, stat: Stats) -> String {
    match stat {
        Stats::RX => format!("/sys/class/net/{}/statistics/rx_bytes", interface_id),
        Stats::TX => format!("/sys/class/net/{}/statistics/tx_bytes", interface_id),
    }
}

/// We need to calculate the speed depending on the interval.
/// Return a value in bytes per second.
fn get_speed(diff: u64) -> u64 {
    diff * DEFAULT_INTERVAL / 1000
}
