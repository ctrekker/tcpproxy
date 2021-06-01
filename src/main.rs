extern crate clap;
use clap::{Arg, App};

use std::thread;
use std::io;
use std::sync::mpsc;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::Command;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    let matches = App::new(NAME.to_string())
                    .version(VERSION)
                    .author(AUTHOR)
                    .about(DESCRIPTION)
                    .arg(Arg::with_name("host")
                        .short("h")
                        .long("host")
                        .value_name("HOST")
                        .help("Hostname to bind proxy to")
                        .takes_value(true)
                    )
                    .arg(Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .value_name("PORT")
                        .help("Port to bind proxy to")
                        .takes_value(true)
                    )
                    .arg(Arg::with_name("proxy-host")
                        .short("H")
                        .long("proxy-host")
                        .value_name("HOST")
                        .help("Hostname to connect proxy to")
                        .takes_value(true)
                    )
                    .arg(Arg::with_name("proxy-port")
                        .short("P")
                        .long("proxy-port")
                        .value_name("PORT")
                        .help("Port to connect proxy to")
                        .takes_value(true)
                    )
                    .arg(Arg::with_name("before-connect")
                        .long("before-connect")
                        .value_name("COMMAND")
                        .help("Script to execute before client connects"))
                    .get_matches();

    let default_port: u16 = 80;
    let default_port_str: &str = &default_port.to_string();
    let default_proxy_port: u16 = 8080;
    let default_proxy_port_str: &str = &default_proxy_port.to_string();

    let host = matches.value_of("host").unwrap_or("127.0.0.1");
    let port = matches.value_of("port").unwrap_or(default_port_str);
    let proxy_host = matches.value_of("proxy-host").unwrap_or("127.0.0.1");
    let proxy_port = matches.value_of("proxy-port").unwrap_or(default_proxy_port_str);

    listen(
        host.to_string(),
        port.parse::<u16>().unwrap_or(default_port),
        proxy_host.to_string(),
        proxy_port.parse::<u16>().unwrap_or(default_proxy_port),
        matches.value_of("before-connect")
    );
}

fn listen(host: String, port: u16, proxy_host: String, proxy_port: u16, before_connect: Option<&str>) {
    let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
    println!("{}:{} => {}:{}", host, port, proxy_host, proxy_port);

    let mut command = String::new();
    let mut has_command = false;
    if let Some(cmd) = before_connect {
        has_command = true;
        command = cmd.to_string();
    }

    for stream in listener.incoming() {
        let proxy_host_copy = proxy_host.clone();
        let command_copy = command.clone();

        thread::spawn(move || {
            if has_command {
                let output = if cfg!(target_os = "windows") {
                    Command::new("cmd")
                        .arg("/C")
                        .arg(command_copy)
                        .output()
                } else {
                    Command::new("bash")
                        .arg("-c")
                        .arg(command_copy)
                        .output()
                };
                let result = output.expect("Pre-connect execution failed. Proceeding anyways");
                println!("Executed pre-connect command and recieved the following output:\n{}", std::str::from_utf8(&result.stdout).unwrap());
            }

            let stream = stream.unwrap();

            println!("Connection established");
            handle_connection(stream, proxy_host_copy, proxy_port);
        });
    }
}

fn handle_connection(mut client_stream: TcpStream, proxy_host: String, proxy_port: u16) {
    match TcpStream::connect(format!("{}:{}", proxy_host, proxy_port)) {
        Ok(mut server_stream) => {
            println!("Successfully connected");

            let (tx_server, rx_server) = mpsc::channel::<Vec<u8>>();
            let (tx_client, rx_client) = mpsc::channel::<Vec<u8>>();

            client_stream.set_nonblocking(true).unwrap();
            server_stream.set_nonblocking(true).unwrap();

            thread::spawn(move || {
                loop {
                    let mut buffer = vec![];
                    match client_stream.read_to_end(&mut buffer) {
                        Ok(_) => {
                            
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // Do nothing during connection phase
                        }
                        Err(e) => println!("Encountered IO error: {}", e),
                    };
                    if buffer.len() > 0 {
                        tx_server.send(buffer).unwrap();
                    }
                    match rx_client.try_recv() {
                        Ok(msg) => {
                            client_stream.write(&msg[..]).unwrap();
                        },
                        Err(_e) => {
                            // Happens when there are no messages
                        }
                    };
                }
            });
            thread::spawn(move || {
                loop {
                    let mut buffer = vec![];
                    match server_stream.read_to_end(&mut buffer) {
                        Ok(_) => {

                        },
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // Do nothing during connection phase
                        }
                        Err(e) => println!("encountered IO error: {}", e),
                    };
                    if buffer.len() > 0 {
                        tx_client.send(buffer).unwrap();
                    }
                    match rx_server.try_recv() {
                        Ok(msg) => {
                            server_stream.write(&msg[..]).unwrap();
                        },
                        Err(_e) => {
                            // Happens when there are no messages
                        }
                    };
                }
            });
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
