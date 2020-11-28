/**
* This is a collaborative code editing application based on `https://hal.inria.fr/inria-00336191v3/document`.
*/
mod config;
mod document;
mod node;

use {config::Config, node::Node};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse()?;
    // Trick to tell the compiler that it will live for entirety of program's life.
    let node: &'static mut Node = Box::leak(Box::new(Node::new(
        config.addr.host,
        config.addr.port,
        config.clients,
    )?));

    node.run().await?;

    Ok(())
}
