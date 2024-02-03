use std::borrow::Borrow;
use std::env;
use std::io::Read;
use std::net::{Shutdown, TcpListener};
use std::process::Command;

/**
 * this function is the server which is listening for tcp streams to trigger the nginx to reload its configuration
 */

#[allow(non_snake_case)]
fn main() {
    // read the port as an env variable from the os
    let mut TRIGGER_PORT = env::var("TRIGGER_PORT").unwrap_or_default();

    if TRIGGER_PORT.is_empty() {
        TRIGGER_PORT = "3333".to_string();
    }
    let listener = TcpListener::bind(format!("0.0.0.0:{}", TRIGGER_PORT)).unwrap();
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let mut data = [0; 50];
                while match stream.borrow().read(&mut data) {
                    Ok(size) => {
                        let a = &data[0..size];
                        if a.eq(b"Hello!") {
                            Command::new("bash")
                                .arg("-c")
                                .arg("nginx -s reload")
                                .output()
                                .expect("can't reload nginx");
                            true
                        } else {
                            false
                        }
                    }
                    Err(_) => {
                        println!(
                            "An error occurred, terminating connection with {}",
                            stream.peer_addr().unwrap()
                        );
                        stream.shutdown(Shutdown::Both).unwrap();
                        false
                    }
                } {}
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}
