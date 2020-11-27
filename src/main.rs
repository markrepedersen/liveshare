/**
* This is a collaborative code editing application based on `https://hal.inria.fr/inria-00336191v3/document`.
*/
mod config;
mod document;
mod node;

use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use {config::Config, node::Node};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let config = Config::parse()?;
    // // Trick to tell the compiler that it will live for entirety of program's life.
    // let node: &'static mut Node = Box::leak(Box::new(Node::new(
    //     config.addr.host,
    //     config.addr.port,
    //     config.clients,
    // )?));

    // node.run().await?;

    // TODO: make frontend separate from Node backend. This is better, since frontends can then be interchangeable.

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, r#"{}{}ctrl + q to exit, ctrl + h to print "Hello world!", alt + t to print "termion is cool""#, termion::cursor::Goto(1, 1), termion::clear::All)
            .unwrap();
    stdout.flush().unwrap();

    for key in stdin.keys() {
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::All
        )
        .expect("Clearing screen failed.");

        match key.unwrap() {
            Key::Ctrl('h') => println!("Hello world!"),
            Key::Ctrl('q') => break,
            Key::Alt('t') => println!("termion is cool"),
            _ => (),
        }

        stdout.flush().unwrap();
    }

    Ok(())
}
