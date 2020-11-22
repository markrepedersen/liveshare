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
    let mut map = BTreeMap::new();

    map.insert(1, Char::new(Id::new(1, 0), 0, 'e'));
    map.insert(2, Char::new(Id::new(2, 0), 0, 'e'));
    map.insert(3, Char::new(Id::new(3, 1), 0, 'e'));

    for (key, val) in map.range(..2) {
        println!("Key: {:?}, Value: {:?}", key, val);
    }

    Ok(())
}
