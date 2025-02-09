use core::time;
use log::{debug, error, info, trace, warn};
use std::time::SystemTime;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::io::{self, AsyncReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

mod socket;
use utils::{self, prayer::Prayers, Request};

static EXIT: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() -> io::Result<()> {
    init();
    ctrlc::set_handler(|| EXIT.store(true, Ordering::SeqCst)).unwrap();
    // we need to do ensure the thread gets dropped so that everything inside in dropped

    // for now we will write the socket here:

    let listener = UnixListener::bind("/tmp/adthand").unwrap();
    let prayer = Prayers::new_async("Toronto", "Canada").await.unwrap();
    while !should_exit() {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((socket, _addr)) => {
                        tokio::spawn(async move { handle_client(socket).await });
                    }
                    Err(e) => {error!("{:?}",e);}
                }
            },
            result = prayer.get_next_prayer_async() => {
                info!("{:?}", prayer);
                match result {
                    Ok(current) => {
                        info!("new prayer: {}", current)
                    }
                    Err(e) => {error!("{:?}",e);}
                }
            }
        }
        let (socket, _addr) = listener.accept().await?;
    }

    Ok(cleanup())
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

async fn handle_client(mut stream: UnixStream) {
    info!("Got a connection");
    let mut reader = BufReader::new(stream);
    const SIZE: usize = std::mem::size_of::<Request>();
    let mut buf: [u8; SIZE] = [0u8; SIZE]; //we know how big the request will be
                                           // TODO: Handle errors
    reader.read_exact(&mut buf).await; // error here that should be handeled
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
