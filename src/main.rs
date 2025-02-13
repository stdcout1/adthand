// use std::time::Duration;
// use std::thread;
//
// use rusty_adthan::{PrayerResults, Prayers};
//
// fn main() {
//     let mut prayers = Prayers::new("Toronto".to_string(), "Canada".to_string()).unwrap();
//
//     loop {
//         // this should catch unexpected wakeups on unix systems
//         match prayers.get_next_prayer_unix(3).unwrap() {
//             PrayerResults::Prayer(prayer) => {
//                 println!("it is {} time", prayer.name);
//             }
//             PrayerResults::CaughtUp => {
//                 println!("I am skipping")
//             }
//             PrayerResults::NotTimeYet(dur) => {
//                 println!("{}", dur);
//                 thread::sleep(Duration::from_secs(dur as u64))
//             }
//         }
//         thread::sleep(Duration::from_secs(1))
//     }
// }

mod parser;
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

use clap::Parser;
use parser::Adthand;
use utils::{self, Answer, Request};

fn main() {
    let adthand = Adthand::parse();

    let mut stream = UnixStream::connect("/tmp/adthand").unwrap();

    match adthand {
        Adthand::Init => {}// do we really need this?
        Adthand::Ping => {
            stream
                .write_all(bitcode::encode(&Request::Ping).as_slice())
                .unwrap();
            println!("flushed");
        }
        Adthand::Kill => {
            stream
                .write_all(bitcode::encode(&Request::Kill).as_slice())
                .unwrap();
            println!("flushed");
        }
        Adthand::All => {
            stream
                .write_all(bitcode::encode(&Request::All).as_slice())
                .unwrap();
            let mut buf: Vec<u8> = Vec::new();
            stream.read_to_end(&mut buf).unwrap();
            let cmd: Answer = bitcode::decode(&buf).unwrap();
            println!("Recived command of: {:?}", cmd)
        }
        Adthand::Next => {
            stream
                .write_all(bitcode::encode(&Request::Next).as_slice())
                .unwrap();
            let mut buf: Vec<u8> = Vec::new();
            stream.read_to_end(&mut buf).unwrap();
            let cmd: Answer = bitcode::decode(&buf).unwrap();
            println!("Recived command of: {:?}", cmd)
        }
    }

}
