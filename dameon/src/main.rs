use chrono::Timelike;
use log::{error, info};
use notify_rust::error::ErrorKind;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{fs, path::PathBuf};
use tokio::fs::remove_file;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::signal;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, sleep, Interval};
use utils::prayer::format_time_difference;
use utils::Answer;

mod socket;
use utils::{self, prayer::Prayers, Request};

#[tokio::main]
async fn main() -> io::Result<()> {
    init();

    // for now we will write the socket here:
    const SOCKET_ADDR: &'static str = "/tmp/adthand";
    let listener;
    match UnixListener::bind(SOCKET_ADDR) {
        Ok(l) => {
            listener = l;
        }
        Err(error) => {
            if error.kind() == io::ErrorKind::AddrInUse {
                remove_file(SOCKET_ADDR).await.unwrap();
                listener = UnixListener::bind(SOCKET_ADDR).unwrap();
            } else {
                // un recoverable
                panic!("Socket error {error}")
            }
        }
    }
    let prayer = loop {
        match Prayers::new_async(
            String::from("Toronto"),
            String::from("Canada"),
            chrono::Local::now().date_naive(),
        )
        .await
        {
            Ok(p) => break p,
            Err(e) => {
                error!("Error creating Prayers object: {:?}. Retrying...", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    };

    let prayer = Arc::new(RwLock::new(prayer));
    let p1 = Arc::clone(&prayer);
    let p2 = Arc::clone(&prayer);

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let stx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen to ctrl_c");
        stx_clone.send(()).await.unwrap();
    });

    tokio::spawn(async move {
        loop {
            let sleep_dur = p1.write().await.get_next_prayer_duration().await;
            match sleep_dur {
                Ok(time) => {
                    sleep(time).await;
                    notify(p1.read().await.next.as_ref().unwrap().name.as_ref());
                }
                Err(e) => {
                    error!("sleep error: {:?}", e);
                }
            }
        }
    });

    // check for inter expiry
    // we can predict the expiry
    // tokio::spawn(async move {
    //     loop {
    //         let now = chrono::Local::now().time();
    //         let seconds_passed = now.num_seconds_from_midnight();
    //         let seconds_in_day = 24 * 60 * 60;
    //         let sleep_dur = Duration::from_secs((seconds_in_day - seconds_passed) as u64);
    //         info!("sleeping until midnight to check for expiry");
    //         sleep(sleep_dur).await;
    //         // now we know its expired. so we update the prayer
    //         prayer.write().await.get_next_prayer_duration().await
    //     }
    // });

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                break;
            },
            result = listener.accept() => {
                match result {
                    Ok((socket, _addr)) => {
                        let clone = shutdown_tx.clone();
                        let p_clone = prayer.clone();
                        tokio::spawn(async move {
                            handle_client(socket, clone, p_clone).await
                        });
                    }
                    Err(e) => {error!("{:?}",e);}
                }
            },
        }
    }

    Ok(cleanup())
}

fn init() {
    setup_logger().unwrap();
    // connect to the socket and start listening
    info!("Started");
    info!("Time is {}", chrono::Local::now())
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

async fn handle_client(
    mut stream: UnixStream,
    shutdown_tx: mpsc::Sender<()>,
    prayer: Arc<RwLock<Prayers>>,
) {
    info!("Got a connection");
    stream.readable().await;
    const SIZE: usize = std::mem::size_of::<Request>();
    let mut buf: [u8; SIZE] = [0u8; SIZE]; //we know how big the request will be
    stream.read_exact(&mut buf).await; // error here that should be handeled
    let cmd: Request = bitcode::decode(&buf).unwrap();
    info!("Got a {:?} command", cmd);
    let prayer = prayer.read().await;
    match cmd {
        Request::Ping => info!("Pinged!"),
        Request::Kill => shutdown_tx.send(()).await.unwrap(),
        Request::Next => {
            let next_name: &str = prayer.next.as_ref().unwrap().name.as_ref();
            let next_time: String = prayer
                .next
                .as_ref()
                .unwrap()
                .time
                .time()
                .format("%I:%M %P")
                .to_string();
            let next_difference: String =
                format_time_difference(prayer.next.as_ref().unwrap().time);
            send(
                &mut stream,
                Answer::Next(next_name, &next_time, &next_difference),
            )
            .await
        }
        Request::All => {
            send(
                &mut stream,
                Answer::All(
                    prayer
                        .prayer_que
                        .iter()
                        .map(|p| {
                            (
                                p.name.as_ref(),
                                p.time.time().format("%I:%M %P").to_string(),
                            )
                        })
                        .collect(),
                ),
            )
            .await
        }
        Request::Waybar => {
            let next_name: &str = prayer.next.as_ref().unwrap().name.as_ref();
            let next_time: String = prayer
                .next
                .as_ref()
                .unwrap()
                .time
                .time()
                .format("%I:%M %P")
                .to_string();
            let next_difference: String =
                format_time_difference(prayer.next.as_ref().unwrap().time);
            send(
                &mut stream,
                Answer::Waybar(
                    next_name,
                    &next_time,
                    &next_difference,
                    prayer
                        .prayer_que
                        .iter()
                        .map(|p| {
                            (
                                p.name.as_ref(),
                                p.time.time().format("%I:%M %P").to_string(),
                            )
                        })
                        .collect(),
                ),
            )
            .await;
        }
    }
    info!("Size of incomming payload: {}", buf.len());
}

async fn send<'a>(stream: &mut UnixStream, message: Answer<'a>) {
    let cmd: Vec<u8> = bitcode::encode(&message);
    info!("Encoded: {:?}", cmd);
    stream.writable().await;
    stream.write_all(&cmd).await.unwrap();
    info!("Sent back a payload of size: {}", cmd.len())
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
        // .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
