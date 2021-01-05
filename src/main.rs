mod atom;
/**
* This is a collaborative code editing application based on `https://hal.inria.fr/inria-00336191v3/document`.
*/
mod config;
mod document;
mod id;
mod node;
mod position;
mod range;

use {
    config::{Client, Config},
    node::Node,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse()?;
    let addr = Client::new("localhost".to_string(), 2000);
    let client = Client::new("localhost".to_string(), 2001);
    let mut node = Node::init(addr, client).await;

    node.run().await?;

    Ok(())
}
