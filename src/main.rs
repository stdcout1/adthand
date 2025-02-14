
mod parser;
mod waybar;
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
        Adthand::Init => {} // do we really need this?
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
        Adthand::Next { relative } => {
            stream
                .write_all(bitcode::encode(&Request::Next).as_slice())
                .unwrap();
            let mut buf: Vec<u8> = Vec::new();
            stream.read_to_end(&mut buf).unwrap();
            let cmd: Answer = bitcode::decode(&buf).unwrap();
            println!("Recived command of: {:?}", cmd);
            if let Answer::Next(name, time, relative_time) = cmd {
                if relative {
                    println!("{} {}", name, relative_time)
                } else {
                    println!("{} at {}", name, time)
                }
            } else {
                eprint!("Recived incorrect response")
            }
        }
        Adthand::Waybar => {
            stream
                .write_all(bitcode::encode(&Request::Waybar).as_slice())
                .unwrap();
            let mut buf: Vec<u8> = Vec::new();
            stream.read_to_end(&mut buf).unwrap();
            let cmd: Answer = bitcode::decode(&buf).unwrap();
            if let Some(result) = waybar::Waybar::new(cmd) {
                println!("{}", result.to_string())
            } else {
                // this should never happen and is truly unreconverable
                panic!("incorrect response")
            }
        }
    }
}
