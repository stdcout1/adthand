use log::{debug, error, info, trace, warn};
use tokio::signal;
use tokio::sync::mpsc;
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


#[tokio::main]
async fn main() -> io::Result<()> {
    init();

    // for now we will write the socket here:

    let listener = UnixListener::bind("/tmp/adthand").unwrap();
    let mut prayer = Prayers::new_async(String::from("Toronto"), String::from("Canada"), chrono::Local::now().date_naive() )
        .await
        .unwrap();

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let stx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen to ctrl_c");
        stx_clone.send(()).await.unwrap();
    });

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                break;
            }
            result = listener.accept() => {
                match result {
                    Ok((socket, _addr)) => {
                        let clone = shutdown_tx.clone();
                        tokio::spawn(async move {
                            handle_client(socket, clone).await
                        });
                    }
                    Err(e) => {error!("{:?}",e);}
                }
            },
            result = prayer.get_next_prayer_async() => {
                info!("{:?}", prayer);
                match result {
                    Ok(current) => {
                        notify(&current);
                    }
                    Err(e) => {error!("{:?}",e);}
                }
            }
        }
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

async fn handle_client(mut stream: UnixStream, shutdown_tx: mpsc::Sender<()>) {
    info!("Got a connection");
    let mut reader = BufReader::new(stream);
    const SIZE: usize = std::mem::size_of::<Request>();
    let mut buf: [u8; SIZE] = [0u8; SIZE]; //we know how big the request will be
                                           // TODO: Handle errors
    reader.read_exact(&mut buf).await; // error here that should be handeled
    let cmd: Request = bitcode::decode(&buf).unwrap();
    match cmd {
        Request::Ping => info!("Pinged!"),
        Request::Kill => shutdown_tx.send(()).await.unwrap(),
    }
    info!("Size of payload: {}", buf.len());
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
