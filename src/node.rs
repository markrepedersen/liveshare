use std::{collections::HashMap, net::SocketAddr};

use {
    crate::{atom::Atom, config, document::Document, range::Range},
    bincode::{deserialize, serialize},
    serde::{Deserialize, Serialize},
    serde_json::ser::to_vec,
    std::io,
    tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
    },
    tracing::{error, info, instrument},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    RemoteInsert { id: i64, lines: Vec<Atom> },
    RemoteDelete { id: i64, lines: Vec<Atom> },
    Insert { lines: Vec<char>, range: Range },
    Delete { range: Range },
}

#[derive(Debug)]
pub struct Peer {
    id: i64,
    addr: SocketAddr,
    conn: TcpStream,
}

impl Peer {
    #[instrument(level = "info")]
    pub fn new(id: i64, addr: SocketAddr, conn: TcpStream) -> Self {
        Self { id, addr, conn }
    }

    /// Send the event to the peer.
    #[instrument(level = "info")]
    pub async fn send(&mut self, event: &Event) -> io::Result<()> {
        let buf = serialize(event).unwrap();
        self.conn.write_all(&buf).await
    }
}

#[derive(Debug)]
pub struct Client {
    addr: SocketAddr,
    conn: TcpStream,
}

impl Client {
    // Sends the event as a JSON payload to the frontend.
    #[instrument(level = "info")]
    pub fn send(&self, range: &Range, event: &Event) {
        let buf = to_vec(event).expect("Unable to serialize event.");
        self.conn.write_all(&buf);
    }

    #[instrument(level = "info")]
    pub async fn connect(config: config::Client) -> Self {
        if let Ok(conn) = TcpStream::connect((config.host, config.port)).await {
            Self {
                addr: conn.local_addr().unwrap(),
                conn,
            }
        } else {
            panic!("Unable to connect to client editor.");
        }
    }
}

/// A node will handle propagation of changes in its respective document.
/// Changes will be applied in a FIFO manner. Each local change will be accompanied by sending a request to each connected client to
/// apply the same change in order to keep each node's document consistent.
/// For efficiency, client connections are established at the start of the program so that connections can be re-used.
#[derive(Debug)]
pub struct Node {
    host: String,
    port: u16,
    id: i64,
    socket: TcpListener,
    client: Client,
    peers: HashMap<i64, Peer>,
    document: Document,
}

impl Node {
    /// Creates the node, creating client connections as necessary.
    /// Any errors connecting will immediately terminate the initalization process.
    #[instrument(level = "info")]
    pub async fn init(addr: config::Client, client_addr: config::Client) -> Self {
        match TcpListener::bind((addr.host.clone(), addr.port)).await {
            Ok(socket) => {
                info!(
                    "Started TCP listener on {}:{}.",
                    addr.host.clone(),
                    addr.port
                );

                Self {
                    host: addr.host,
                    port: addr.port,
                    id: -1,
                    socket,
                    client: Client::connect(client_addr).await,
                    peers: HashMap::new(),
                    document: Document::new(-1),
                }
            }
            Err(e) => panic!(format!(
                "Error connecting to local address: {}:{}",
                addr.host, addr.port
            )),
        }
    }

    /// An event can come from one of two sources:
    /// - The client (editor frontend); or
    /// - connected peers (foreign replicated documents)
    /// # Client
    /// Message from client -> Update local document state -> Propagate change(s) to connected peers
    /// # Peers
    /// Message from peer -> Send character operation to messaging service -> Renders the new document state
    #[instrument(level = "info")]
    pub async fn run(&mut self) -> io::Result<()> {
        info!("[{}:{}] Running node...", self.host, self.port);

        loop {
            let (mut conn, addr) = self.socket.accept().await?;
            let mut buf = Vec::new();

            conn.read_to_end(&mut buf).await?;

            match deserialize::<Event>(&buf) {
                Ok(event) => match event {
                    Event::Insert {
                        ref lines,
                        ref range,
                    } => {
                        if let Some(lines) = self.document.local_insert(range, lines) {
                            let event = Event::RemoteInsert { id: self.id, lines };
                            self.propagate(event).await;
                        }
                    }

                    Event::Delete { ref range } => {
                        if let Some(lines) = self.document.local_delete(range) {
                            let event = Event::RemoteDelete { id: self.id, lines };
                            self.propagate(event).await;
                        }
                    }

                    Event::RemoteInsert { id, ref lines } => {
                        self.add_peer(id, addr, conn);
                        if let Some(ref range) = self.document.remote_insert(lines) {
                            // send range and line contents
                            self.client.send(range, lines);
                        }
                    }

                    Event::RemoteDelete { id, ref lines } => {
                        self.add_peer(id, addr, conn);
                        if let Some(range) = self.document.remote_delete(lines) {
                            self.client.send(range, lines);
                        }
                    }
                },
                Err(e) => error!("Error parsing message from peer: {}", e),
            };
        }
    }

    /// Add a peer to the network.
    /// Peers are identified by their GUID.
    /// If a peer is unidentified (i.e. their GUID is either -1 (unitialized) or unknown), then it will be added to the network.
    /// Otherwise, it is ignored.
    #[instrument(level = "info")]
    fn add_peer(&mut self, id: i64, addr: SocketAddr, conn: TcpStream) {
        if !self.peers.contains_key(&id) {
            self.peers.insert(id, Peer::new(id, addr, conn));
        }
    }

    /// Send the change to each client's respective thread.
    #[instrument(level = "info")]
    async fn propagate(&mut self, event: Event) {
        let tasks: Vec<_> = self
            .peers
            .iter_mut()
            .map(|(_, peer)| peer.send(&event))
            .collect();

        for task in tasks {
            if let Err(e) = task.await {
                error!("Error sending change to peer: {}.", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::config::Client;
    use super::Node;

    #[tokio::test]
    async fn test_add_node() -> Result<(), Box<dyn std::error::Error>> {
        let addr = Client::new("localhost".to_string(), 2000);
        let client = Client::new("localhost".to_string(), 2001);
        let mut n1 = Node::init(addr, client).await;

        n1.run().await?;

        Ok(())
    }
}
