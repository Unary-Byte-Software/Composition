// server.rs
// author: Garen Tyler
// description:
//   This module contains the server data.
//   World data and player information are stored here and saved here.

#[derive(Debug)]
pub struct Server {
    pub num_players: u16,
}
impl Server {
    pub fn new() -> Server {
        Server { num_players: 0 }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ServerEvent {
    PlayerConnected,
    PlayerDisconnected,
}
