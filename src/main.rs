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
use std::{io::{Read, Write}, os::unix::net::UnixStream};

use clap::Parser;
use parser::Adthand;
use utils::{self, Request};

fn main() {
    let adthand = Adthand::parse();
    if let Adthand::Init = adthand {
        //connect to socket 
        let mut stream = UnixStream::connect("/tmp/adthand").unwrap();
        stream.write_all(bitcode::encode(&Request::Ping).as_slice()).unwrap();
        println!("flushed");
    }

    if let Adthand::Kill = adthand {
        println!("Killing...");
        let mut stream = UnixStream::connect("/tmp/adthand").unwrap();
        stream.write_all(bitcode::encode(&Request::Kill).as_slice()).unwrap();
        println!("flushed");
        
    }

}
