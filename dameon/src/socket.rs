// we need to implment ipc to talk to the client
//

use std::{fs, os::unix::net::UnixListener, path::PathBuf};
use log::{debug, error, info, trace, warn};
pub struct SocketWrapper(pub UnixListener);

impl SocketWrapper {
    pub fn new() -> Result<Self, String> {
        let socket_addr = PathBuf::from("/tmp/adthand");
        if socket_addr.exists() {
            //TODO: handle this case
        }
        let listener = UnixListener::bind(socket_addr).unwrap();
        Ok(SocketWrapper(listener))
    }
}

impl Drop for SocketWrapper {
    // FOR NOW THIS DOES NOT EXECUTE
    fn drop(&mut self) {
        let socket_addr = PathBuf::from("/tmp/adthand");
        if let Err(e) = fs::remove_file(&socket_addr) {
            error!("Failed to remove socket at {socket_addr:?}: {e}");
        }
        info!{"Removed socket at {:?}", socket_addr};
    }

}
