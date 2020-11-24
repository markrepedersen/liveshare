use {
    tokio::{task::{spawn, JoinHandle}},
    crate::{
        config::Client,
        document::{Char, Document},
    },
    serde::{Deserialize, Serialize},
    std::{
        io,
        net::TcpListener,
        sync::mpsc::{channel, Receiver, Sender},
    },
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Operation {
    LocalInsert { val: char, pos: usize },
    RemoteInsert { val: Char },
    LocalDelete { pos: usize },
    RemoteDelete { val: Char },
    Invalid,
}

pub struct Node {
    host: String,
    port: u16,
    document: Document,
    handle: Option<JoinHandle<()>>,
    sender: Sender<Operation>,
    receiver: Receiver<Operation>,
}

impl Node {
    /// Creates a new node that listens for incoming operations from remote clients.
    /// `host`: the hostname of this node
    /// `port`: the port of this node
    /// `nodes`: the set of nodes that changes will be progagated to
    pub fn new(host: &str, port: u16, nodes: Vec<Client>) -> Result<Self, io::Error> {
        let (sender, receiver) = channel::<Operation>();
        let mut document = Document::new();

        document.clients(nodes);
        Ok(Self {
            host: String::from(host),
            port,
            handle: None,
            document,
            sender,
            receiver,
        })
    }

    /// Process events in any order, both locally and remotely.
    /// This will spawn a child thread that will acts as a work scheduler for this node.
    /// Remote clients will send work to this node that will then be sent to the appropriate channel for processing.
    /// # Local Events
    /// Key presses will be registered as events and then sent as remote operations to connected clients.
    /// # Remote Events
    /// Foreign document changes caused by other clients will be sent over network to this node, which will then
    /// incorporate these changes into this node's document.
    /// Remote inserts and deletions will be handled transparently as if they were local changes.
    pub async fn process_events(&mut self) -> Result<(), io::Error> {
        let thread_sender = self.sender.clone();
	let host = self.host.clone();
	let port = self.port;
	
	self.handle = Some(spawn(async move {
	    let socket = TcpListener::bind((host, port)).unwrap();

	    for stream in socket.incoming() {
                match stream {
                    Ok(ref stream) => {
                        let operation: Operation = bincode::deserialize_from(stream).unwrap();
                        thread_sender
                            .send(operation)
                            .expect("Problem sending operation to local channel.");
                    }
                    Err(e) => println!("Error while receiving from client: {:#?}", e),
                }
            }
	}));

        loop {
            match self.receiver.recv() {
                Ok(Operation::LocalInsert { val, pos }) => self.document.local_insert(val, pos),
                Ok(Operation::RemoteInsert { val }) => self.document.remote_insert(val),
                Ok(Operation::LocalDelete { pos }) => self.document.local_delete(pos),
                Ok(Operation::RemoteDelete { val }) => self.document.remote_delete(val),
                Ok(Operation::Invalid) => println!("Received invalid operation."),
                Err(e) => println!("Received error: {:#?}", e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_add_node() {}
}
