pub mod server {

    use uuid::Uuid;
    use tokio::net::{TcpStream, TcpListener};
    use std::sync::{Arc, Mutex};
    use std::error::Error;
    use std::str::from_utf8;
    use tokio::io::AsyncReadExt;
    use tokio::time::{sleep, Duration};
    use tracing::{debug, info};
    use futures::future::join_all;

    const SNOOZE: u64 = 1000;

    pub struct Server {
        pub id: Uuid,
        ip: Arc<str>,
        buffer: Arc<Mutex<String>>,
        peer_addresses: Vec<String>,
        peer_streams: Option<Vec<TcpStream>>,
    }

    fn write_to_buffer(mut stream: TcpStream, buffer: Arc<Mutex<String>>) -> Arc<Mutex<String>> {
        let mut loc_buffer: [u8; 2048] = [0; 2048];
        let _ = stream.read(&mut loc_buffer);
        let new_string = from_utf8(&loc_buffer).unwrap();
        *buffer.lock().unwrap() = new_string.to_owned();
        buffer
    }

    impl Server {
        pub async fn new(ip: impl Into<String>, peer_addresses: Vec<String>, buffer: Arc<Mutex<String>>) -> Result<Self, Box<dyn Error>> {
            let this_ip: Arc<str> = ip.into().into();
            debug!("Listening over ip: {}", this_ip.clone());

            Ok(Server {
                id: Uuid::new_v4(),
                ip: this_ip,
                buffer,
                peer_addresses,
                peer_streams: None,
            })
        }


        pub async fn init(&mut self) -> Result<(), Box<dyn Error>> {
            let mut buffer_clone = self.buffer.clone();
            debug!("binding to ip: {}", self.ip.as_ref());
            let sock_listener = TcpListener::bind(self.ip.as_ref()).await?;
            debug!("Listening to peers...");
            tokio::spawn(async move {
                loop {
                    let (listener, _) = sock_listener.accept().await.unwrap();
                    buffer_clone =  write_to_buffer(listener, buffer_clone);
                }
            });
            debug!("Connecting to peers...");
            let peer_streams: Vec<TcpStream> = join_all(self.peer_addresses.iter().map(|peer_ip| async { 
                let ref_peer: &str = peer_ip.as_ref();
                debug!("Attempting connection to {}", ref_peer);
                let mut opt_stream = None;
                while opt_stream.is_none() {
                    tokio::time::sleep(Duration::from_millis(SNOOZE)).await;
                    opt_stream = TcpStream::connect(ref_peer).await.ok();
                    debug!("Sleeping...");
                }
                opt_stream.unwrap()
            })).await;
            self.peer_streams = Some(peer_streams);
            Ok(())
        }
    }
}
