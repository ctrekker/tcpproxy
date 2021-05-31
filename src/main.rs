use std::thread;
use std::io;
use std::sync::mpsc;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:25565").unwrap();
    for stream in listener.incoming() {
        thread::spawn(move || {
            let stream = stream.unwrap();

            println!("Connection established!");
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut client_stream: TcpStream) {
    match TcpStream::connect("192.168.1.18:25565") {
        Ok(mut server_stream) => {
            println!("Successfully connected to server in port 25565");

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
                        Err(e) => println!("encountered IO error: {}", e),
                    };
                    if buffer.len() > 0 {
                        tx_server.send(buffer).unwrap();
                    }
                    match rx_client.try_recv() {
                        Ok(msg) => {
                            client_stream.write(&msg[..]).unwrap();
                        },
                        Err(_e) => {
                            
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
                            
                        }
                    };
                }
            });
            // thread::spawn(|| {

            // })
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
