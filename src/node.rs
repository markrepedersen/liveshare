use std::{collections::HashMap, net::SocketAddr};

use {
    crate::document::{Char, Document},
    bincode::{deserialize, serialize},
    serde::{Deserialize, Serialize},
    std::io,
    tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
    },
    tracing::{error, info, instrument},
};

#[derive(Debug)]
pub struct Peer {
    id: u64,
    addr: SocketAddr,
    conn: TcpStream,
}

impl Peer {
    pub fn new(id: u64, addr: SocketAddr, conn: TcpStream) -> Self {
        Self { id, addr, conn }
    }

    /// Send the event to the peer.
    pub async fn send(&mut self, event: &Event) -> io::Result<()> {
        let buf = serialize(event).unwrap();
        self.conn.write_all(&buf).await
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    RemoteInsert { val: Char },
    RemoteDelete { val: Char },
    Insert { c: char, line: usize, column: usize },
    Delete { line: usize, column: usize },
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
    document: Document,
    peers: HashMap<i64, Peer>,
}

impl Node {
    /// Creates the node.
    /// Note: no work is done until the `run` function is called.
    /// `host`: the hostname of this node
    /// `port`: the port of this node
    /// `clients`: the set of nodes that changes will be progagated to
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            id: -1,
            document: Document::new(-1),
            peers: HashMap::new(),
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
        let host = &self.host;
        let port = self.port;
        let socket = TcpListener::bind((host.clone(), port)).await?;

        info!("Started TCP listener on {}:{}.", host.clone(), port);

        loop {
            let (ref mut stream, ref addr) = socket.accept().await?;
            let mut buf = Vec::new();

            stream.read_to_end(&mut buf).await?;

            match deserialize::<Event>(&buf) {
                Ok(event) => match event {
                    Event::Insert { c, line, column } => {
                        if let Some(val) = self.document.insert_by_index(c, line) {
                            self.propagate(Event::RemoteInsert { val }).await;
                        }
                    }
                    Event::Delete { line, column } => {
                        if let Some(val) = self.document.delete_by_index(line) {
                            self.propagate(Event::RemoteDelete { val }).await;
                        }
                    }
                    Event::RemoteInsert { ref val } => {
                        // TODO: render remote change to gui
                        self.add_peer(id, addr, conn);
                        self.document.insert_by_val(val);
                    }
                    Event::RemoteDelete { ref val } => {
                        // TODO: render remote change to gui
                        self.add_peer(id, addr, conn);
                        self.document.delete_by_val(val);
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
    fn add_peer(&mut self, id: i64, addr: SocketAddr, conn: TcpStream) {
        if id == -1 || !self.peers.contains_key(&id) {
            self.peers.insert(id, Peer::new(id, addr, conn));
        }
    }

    /// Configures this node's `unique` site ID.
    /// Each node must have a globally unique site ID, so when a new node is introduced to the network, it will
    /// ping each other node to determine the highest ID so far. The new node's ID will be that ID incremented by one.
    #[instrument(level = "info")]
    fn init_guid(peers: &Vec<Peer>) -> u64 {
        unimplemented!("Add GUID for each node");
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
    use super::Node;

    #[tokio::test]
    async fn test_add_node() -> Result<(), Box<dyn std::error::Error>> {
        let host = "localhost".to_string();
        let mut n1 = Node::new(host, 2001);
        // let mut n2 = Node::new(host, 2002);

        n1.run().await?;

        Ok(())
    }
}
