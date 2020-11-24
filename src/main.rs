/**
* This is a collaborative code editing application based on `https://hal.inria.fr/inria-00336191v3/document`.
*/
mod config;
mod document;
mod node;

use {
    config::Config, document::Char, document::Document, document::Id, node::Node,
    std::collections::BTreeMap,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse()?;
    let mut node = Node::new(&config.addr.host, config.addr.port, config.clients)?;

    node.process_events().await?;

    Ok(())
}
