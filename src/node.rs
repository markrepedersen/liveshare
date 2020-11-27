use {
    crate::{
        config::Client,
        document::{Char, Document},
    },
    bincode::serialize_into,
    serde::{Deserialize, Serialize},
    std::net::TcpStream,
    std::{
        io::{self, BufWriter},
        net::TcpListener,
        sync::mpsc::{channel, Receiver, Sender},
    },
    tokio::task::{spawn, JoinHandle},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    CharacterKeyPress { val: char, i: usize },
    BackspaceKeyPress { i: usize },
    RemoteInsert { val: Char },
    RemoteDelete { val: Char },
}

/// A node will handle propagation of changes in its respective document.
/// Changes will be applied in a FIFO manner. Each local change will be accompanied by sending a request to each connected client to
/// apply the same change in order to keep each node's document consistent.
/// For efficiency, client connections are established at the start of the program so that connections can be re-used.
pub struct Node {
    host: String,
    port: u16,
    document: Document,
    handle: Option<JoinHandle<()>>,
    sender: Sender<Event>,
    event_handler: Receiver<Event>,
    clients: Vec<Client>,
    client_channels: Vec<Sender<Event>>,
}

impl Drop for Node {
    fn drop(&mut self) {
        if let Some(ref handle) = self.handle {
            // Cancel the thread instead of detaching it.
            handle.abort();
        }
    }
}

impl Node {
    /// Creates the node.
    /// Note: no work is done until the `run` function is called.
    /// `host`: the hostname of this node
    /// `port`: the port of this node
    /// `nodes`: the set of nodes that changes will be progagated to
    pub fn new(host: String, port: u16, clients: Vec<Client>) -> Result<Self, io::Error> {
        let (sender, receiver) = channel::<Event>();
        let document = Document::new();

        Ok(Self {
            host,
            port,
            handle: None,
            clients,
            client_channels: Vec::new(),
            document,
            sender,
            event_handler: receiver,
        })
    }

    /// Starts the node by processing events in any order, both locally and remotely.
    /// # Local Events
    /// Key presses will be registered as events and then sent as remote operations to connected clients.
    /// # Remote Events
    /// Foreign document changes caused by other clients will be sent over network to this node, which will then
    /// incorporate these changes into this node's document.
    /// Remote inserts and deletions will be handled transparently as if they were local changes.
    pub async fn run(&'static mut self) -> Result<(), io::Error> {
        self.init_clients();
        self.recv();
	
        loop {
            match self.event_handler.recv() {
                Ok(Event::CharacterKeyPress { val, i }) => {
                    if let Some(change) = self.document.insert_by_index(val, i) {
                        Self::propagate(&self.client_channels, &change);
                    }
                }
                Ok(Event::BackspaceKeyPress { i }) => {
                    if let Some(change) = self.document.delete_by_index(i) {
                        Self::propagate(&self.client_channels, &change);
                    }
                }
		// TODO: add binary search methods in document.rs so chars can be inserted/deleted.
                Ok(Event::RemoteInsert { val }) => {}
                Ok(Event::RemoteDelete { val }) => {}
                Err(e) => println!("Received error: {:#?}", e),
            }
        }
    }

    fn recv(&mut self) {
        let tx = self.sender.clone();
        let host = self.host.clone();
        let port = self.port;
        self.handle = Some(spawn(async move {
            let socket = TcpListener::bind((host, port)).unwrap();

            for stream in socket.incoming() {
                match stream {
                    Ok(ref stream) => {
                        let operation: Event = bincode::deserialize_from(stream).unwrap();
                        tx.send(operation)
                            .expect("Problem sending operation to local channel.");
                    }
                    Err(e) => println!("Error while receiving from client: {:#?}", e),
                }
            }
        }));
    }

    /// Send the change to each client's respective thread.
    fn propagate(channels: &Vec<Sender<Event>>, change: &Char) {
        for channel in channels {
            let event = Event::RemoteInsert {
                val: change.to_owned(),
            };
            channel.send(event).expect("Error sending to channel.");
        }
    }

    /// Establishes a TCP connection to each client in `clients`.
    /// Re-using each connection every time a request is made is more efficient, since requests may be done many times per second.
    fn init_clients(&mut self) {
        self.client_channels = self
            .clients
            .iter()
            .map(|client| {
                // TODO: Profile to see if https://docs.rs/flume/0.9.2/flume/ is faster than std::mpsc channels
                let (sender, receiver) = channel::<Event>();
                let (host, port) = (client.host.to_owned(), client.port);

                spawn(async move {
                    let host = host.clone();
                    let mut conn = BufWriter::new(TcpStream::connect((host, port)).unwrap());

                    loop {
                        match receiver.recv() {
                            Ok(op) => {
                                serialize_into(&mut conn, &op)
                                    .expect("Error serializing remote delete operation");
                            }
                            Err(e) => panic!("Sender hung up due to error: {:#?}.", e),
                        }
                    }
                });

                sender
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::Node;

    #[tokio::test]
    async fn test_add_node() {
        let node1 = Node::new(String::from("localhost"), 2001, Vec::new());
        let node2 = Node::new(String::from("localhost"), 2002, Vec::new());
    }
}
