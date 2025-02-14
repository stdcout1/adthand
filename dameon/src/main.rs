use chrono::NaiveTime;
use log::{debug, error, info, trace, warn};
use std::sync::Arc;
use std::time::SystemTime;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{UnixListener, UnixStream};
use tokio::signal;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::sleep;
use utils::prayer::{format_time_difference, Prayer};
use utils::Answer;

mod socket;
use utils::{self, prayer::Prayers, Request};

struct State {
    next: Prayer,
    upcoming: Vec<Prayer>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    init();

    // for now we will write the socket here:

    let listener = UnixListener::bind("/tmp/adthand").unwrap();
    let mut prayer = Prayers::new_async(
        String::from("Toronto"),
        String::from("Canada"),
        chrono::Local::now().date_naive(),
    )
    .await
    .unwrap();

    let prayer = Arc::new(RwLock::new(prayer));

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let stx_clone = shutdown_tx.clone();
    let prayer_ptr = prayer.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen to ctrl_c");
        stx_clone.send(()).await.unwrap();
    });

    tokio::spawn(async move {
        loop {
            let sleep_dur = prayer_ptr.write().await.get_next_prayer_duration().await;
            match sleep_dur {
                Ok(time) => {
                    sleep(time).await;
                    notify(prayer_ptr.read().await.next.as_ref().unwrap().name.as_ref());
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    });

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
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
