/**
* This is a collaborative code editing application based on `https://hal.inria.fr/inria-00336191v3/document`.
*/
mod config;
mod document;

use config::Config;
use document::Char;
use document::Document;
use document::Id;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let config = Config::parse()?;
    // let listener = TcpListener::bind((config.addr.host, config.addr.port))?;

    Ok(())
}
