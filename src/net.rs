// net.rs
// authors: Garen Tyler, Danton Hou
// description:
//   The module with everything to do with networkng.

extern crate radix64;

use crate::mctypes::*;
use crate::protocol::*;
use crate::server::{Server, ServerEvent};
use crate::{config, log};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub fn start_listening(tx: Sender<ServerEvent>, slock: Arc<Mutex<Server>>) {
    let server_address: &str = &format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(server_address);
    if listener.is_err() {
        log.error("Could not start listener");
    } else {
        log.important(&format!("Started server on {}", server_address));
        for stream in listener.unwrap().incoming() {
            if stream.is_err() {
                log.error("Could not connect to client");
            } else {
                let c = tx.clone();
                let s = slock.clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(stream.unwrap(), s, c) {
                        log.error(&format!("Error when handling client: {}", e));
                    }
                });
            }
        }
    }
}
fn handle_client(
    t: TcpStream,
    slock: Arc<Mutex<Server>>,
    c: Sender<ServerEvent>,
) -> std::io::Result<()> {
    log.info("Got a client!");
    let mut gc = GameConnection {
        stream: t,
        state: GameState::Handshake,
        protocol_version: 0,
    };

    'main: loop {
        match gc.state {
            GameState::Handshake => {
                // Read the handshake packet.
                let (_packet_len, _packet_id) = read_packet_header(&mut gc.stream)?;
                let handshake = Handshake::read(&mut gc.stream)?;
                log.info(&format!("{:?}", handshake));
                gc.state = if handshake.protocol_version.value != config.protocol_version as i32
                    && handshake.next_state.value == 2
                {
                    GameState::Closed
                } else {
                    match handshake.next_state.value {
                        1 => GameState::Status,
                        2 => GameState::Login,
                        _ => GameState::Closed,
                    }
                };
                log.info(&format!("Next state: {:?}", gc.state));
                gc.protocol_version = handshake.protocol_version.value as u16;
            }
            GameState::Status => {
                // Read the request packet.
                let (_request_packet_len, _request_packet_id) = read_packet_header(&mut gc.stream)?;
                // Send the response packet.
                log.warn("Server favicon not working correctly. Fix this in issue #4");
                let mut base64_encoded_favicon = "".to_owned();
                let a = || -> std::io::Result<Vec<u8>> {
                    // Only call this if config.favicon is not None, or it'll panic.
                    use std::fs::File;
                    use std::io::prelude::*;
                    let mut file = File::open(config.favicon.as_ref().unwrap())?;
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                    Ok(buffer)
                };
                if config.favicon.is_some() {
                    let temp = a();
                    if let Ok(s) = temp {
                        base64_encoded_favicon = radix64::STD.encode(&s);
                    } else {
                        println!("{:?}", temp);
                    }
                }
                #[allow(unused_assignments)]
                let mut response = MCString::from(""); // Just a placeholder value.
                {
                    // Get the mutex lock.
                    // .unwrap() because the mutex would already be poisoned by a panic, so no worries about panicking here.
                    // Also no worries about blocking this thread, it's not important to be fast.
                    let server = slock.lock().unwrap();
                    response = MCString::from(
                        format!("{{\n\t\"version\": {{\n\t\t\"name\": \"Composition 1.15.2\",\n\t\t\"protocol\": {}\n\t}},\n\t\"players\": {{\n\t\t\"max\": {},\n\t\t\"online\": {},\n\t\t\"sample\": [\n\t\t\t{{\n\t\t\t\t\"name\": \"fumolover12\",\n\t\t\t\t\"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"\n\t\t\t}}\n\t\t]\n\t}},\n\t\"description\": {{\n\t\t\"text\": \"{}\"\n\t}},\n\t\"favicon\": \"data:image/png;base64,{}\"\n}}", config.protocol_version, config.max_players, server.num_players, config.motd, base64_encoded_favicon),
                    );
                    // Release the lock.
                }
                let packet_id = MCVarInt::from(0x00);
                let packet_len = MCVarInt::from(
                    packet_id.to_bytes().len() as i32 + response.to_bytes().len() as i32,
                );
                for b in packet_len.to_bytes() {
                    write_byte(&mut gc.stream, b)?;
                }
                for b in packet_id.to_bytes() {
                    write_byte(&mut gc.stream, b)?;
                }
                for b in response.to_bytes() {
                    write_byte(&mut gc.stream, b)?;
                }
                // Read the ping packet.
                let (_ping_packet_len, _ping_packet_id) = read_packet_header(&mut gc.stream)?;
                let num = MCLong::from_stream(&mut gc.stream)?;
                log.info(&format!("Ping number: {:?}", num));
                // Send the pong packet.
                let packet_id = MCVarInt::from(0x01);
                let packet_len = MCVarInt::from(packet_id.to_bytes().len() as i32 + 8i32);
                for b in packet_len.to_bytes() {
                    write_byte(&mut gc.stream, b)?;
                }
                for b in packet_id.to_bytes() {
                    write_byte(&mut gc.stream, b)?;
                }
                for b in num.to_bytes() {
                    write_byte(&mut gc.stream, b)?;
                }
                gc.state = GameState::Closed;
            }
            GameState::Login => {
                // Read the login start packet.
                let (_packet_len, _packet_id) = read_packet_header(&mut gc.stream)?;
                let login = LoginStart::read(&mut gc.stream)?;
                log.info(&format!("{:?}", login));
            }
            GameState::Play => {
                c.send(ServerEvent::PlayerConnected).unwrap();
                c.send(ServerEvent::PlayerDisconnected).unwrap();
            }
            GameState::Closed => {
                log.info(&format!(
                    "Client at {} closed connection",
                    gc.stream.peer_addr().unwrap()
                ));
                break 'main;
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum GameState {
    Handshake,
    Status,
    Login,
    Play,
    Closed,
}
#[allow(dead_code)]
pub struct GameConnection {
    pub stream: TcpStream,
    pub state: GameState,
    pub protocol_version: u16,
}
