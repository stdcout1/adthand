use core::time;
use log::{debug, error, info, trace, warn};
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::time::SystemTime;
use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    thread::{self, spawn},
    time::Duration,
};

mod socket;
use utils::prayer;
use utils::{self, prayer::Prayers, Request};

static EXIT: AtomicBool = AtomicBool::new(false);
fn main() {
    init();
    // we need to do ensure the thread gets dropped so that everything inside in dropped
    let command_thread = spawn(|| {
        let listener: &UnixListener = &socket::SocketWrapper::new().unwrap().0;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_client(stream); // we dont want theaded connections!
                }
                Err(err) => {
                    panic!("Error: with types");
                }
            }
        }
    });

    let prayer_thread = spawn(|| {
        let prayers = Prayers::new("Toronto", "Canada").unwrap();
        loop {
            let name = prayers.get_next_prayer().unwrap();
            info!("prayer time!");
            //implement notify system here
            notify(&name);
            thread::sleep(Duration::from_secs(1)); //dont really need but just to be safe!
        }
    });

    ctrlc::set_handler(|| EXIT.store(true, Ordering::SeqCst)).unwrap();
    while !should_exit() {
        check();
        thread::sleep(time::Duration::from_secs(1));
    }

    cleanup();
}

fn init() {
    setup_logger().unwrap();
    // connect to the socket and start listening
    info!("Started");
}

fn notify(name: &str) {
    let message = format!("It is {name} time");
    notify_rust::Notification::new()
        .summary("Adthan")
        .body(&message)
        .timeout(notify_rust::Timeout::Milliseconds(6000)) //milliseconds
        .show()
        .unwrap();
}

fn handle_client(mut stream: UnixStream) {
    info!("Got a connection");
    let mut reader = BufReader::new(stream);
    const SIZE: usize = std::mem::size_of::<Request>();
    let mut buf: [u8; SIZE] = [0u8; SIZE]; //we know how big the request will be
                                           // TODO: Handle errors
    reader.read_exact(&mut buf).unwrap();
    let cmd: Request = bitcode::decode(&buf).unwrap();
    match cmd {
        Request::Ping => info!("Pinged!"),
        Request::Kill => EXIT.store(true, Ordering::Relaxed),
    }
    info!("Size of payload: {}", buf.len());
}

fn check() {
    info!("Checking...");
}

fn cleanup() {
    //delete the socket --
    let socket_addr = PathBuf::from("/tmp/adthand");
    if let Err(e) = fs::remove_file(&socket_addr) {
        error!("Failed to remove socket at {socket_addr:?}: {e}");
    }
    info! {"Removed socket at {:?}", socket_addr};
    info!("Cleaning up...")
}

fn should_exit() -> bool {
    EXIT.load(Ordering::Acquire)
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
