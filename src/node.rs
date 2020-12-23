use {
    crate::{
        config::Client,
        document::{Char, Document},
    },
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
    host: String,
    port: u16,
    conn: TcpStream,
}

impl Peer {
    /// Connect to the given peer and return its connection details.
    pub async fn connect(host: &String, port: u16) -> io::Result<Self> {
        Ok(Self {
            host: host.clone(),
            port,
            conn: TcpStream::connect((host.clone(), port)).await?,
        })
    }

    /// Send the event to the peer.
    pub async fn send(&mut self, event: &Event) -> io::Result<()> {
        let buf = serialize(event).unwrap();
        self.conn.write_all(&buf).await
    }
}

/// An event can come from one of two sources: The messaging service (editor frontend) or from connected peers (foreign replicated documents).
/// *Messaging Service*
/// Message from RabbitMQ -> Update local document state -> Propagate change(s) to connected peers
/// *Peers*
/// Message from peer -> Send character operation to messaging service -> Renders the new document state
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
    document: Document,
    peers: Vec<Peer>,
}

impl Node {
    /// Creates the node.
    /// Note: no work is done until the `run` function is called.
    /// `host`: the hostname of this node
    /// `port`: the port of this node
    /// `clients`: the set of nodes that changes will be progagated to
    pub async fn new(host: String, port: u16, clients: Vec<Client>) -> io::Result<Self> {
        let mut peers = Vec::new();

        for client in &clients {
            peers.push(Peer::connect(&client.host, client.port).await?);
        }

        let guid = Self::init_guid(&peers);
        let document = Document::new(guid);

        Ok(Self {
            host,
            port,
            document,
            peers,
        })
    }

    /// Runs the event loop
    #[instrument(level = "info")]
    pub async fn run(&mut self) -> io::Result<()> {
        let host = &self.host;
        let port = self.port;
        let socket = TcpListener::bind((host.clone(), port)).await?;

        info!("Started TCP listener on {}:{}.", host.clone(), port);

        loop {
            let (ref mut stream, _) = socket.accept().await?;
            let mut buf = Vec::new();

            stream.read_to_end(&mut buf).await?;

            match deserialize::<Event>(&buf) {
                Ok(event) => self.handle_event(event).await,
                Err(e) => error!("Error parsing message from peer: {}", e),
            };
        }
    }

    /// Configures this node's `unique` site ID.
    /// Each node must have a globally unique site ID, so when a new node is introduced to the network, it will
    /// ping each other node to determine the highest ID so far. The new node's ID will be that ID incremented by one.
    #[instrument(level = "info")]
    fn init_guid(peers: &Vec<Peer>) -> u64 {
        unimplemented!("Add GUID for each node");
    }

    #[instrument(level = "info")]
    async fn handle_event(&mut self, event: Event) {
        match event {
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
                self.document.insert_by_val(val);
            }
            Event::RemoteDelete { ref val } => {
                // TODO: render remote change to gui
                self.document.delete_by_val(val);
            }
        }
    }

    /// Send the change to each client's respective thread.
    #[instrument(level = "info")]
    async fn propagate(&mut self, event: Event) {
        let tasks: Vec<_> = self
            .peers
            .iter_mut()
            .map(|peer| peer.send(&event))
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
        let mut n1 = Node::new(String::from("localhost"), 2001, Vec::new()).await?;
        // let mut n2 = Node::new(String::from("localhost"), 2002, Vec::new()).await?;

        n1.run().await?;

        Ok(())
    }
}
