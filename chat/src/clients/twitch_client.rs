use std::env;
use std::{thread,time};
use std::io::{self, Read, Write};

use tungstenite::{connect, Message};
use url::Url;

use futures::sync::mpsc;
use futures::{Future, Sink, Stream};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::stream::PeerAddr;


pub struct TwitchClient {
    sender: mpsc::Sender<Message>,
    rec: mpsc::Receiver<Message>,
    user: String,
}

impl TwitchClient {

    pub fn send(&self, msg: &str) -> Result<(),()> {
        let ws_msg = Message::Text(format!("PRIVMSG #{} :{}", self.user, msg).into());
        println!("private message sent: {}", format!("PRIVMSG #{} {}", self.user, msg));
        return self.send_raw(ws_msg);
    }

    pub fn send_raw(&self, msg: Message) -> Result<(),()> {
        let sender = self.sender.clone();

        match sender.send(msg).wait() {
            Ok(_) => {
                println!("0");
            },
            Err(_) => {
                println!("channel closed");
                return Err(());
            },
        };
        return Ok(());
    }

    pub fn new(url: &str, auth: &str, usr: &str, read: &dyn Fn(String)) -> TwitchClient {
        let url = Url::parse(url).unwrap();
        let mut stdout = io::stdout();
        let (mut stdin_tx, stdin_rx) = mpsc::channel(0);
        let (mut msg_send, msg_r) = mpsc::channel(0);
        let stdin_rx = stdin_rx.map_err(|_| panic!());
        let client = connect_async(url)
            .and_then(move |(ws_stream, _)| {
                println!("WebSocket handshake has been successfully completed");

                let addr = ws_stream
                    .peer_addr()
                    .expect("connected streams should have a peer address");
                println!("Peer address: {}", addr);

                // `sink` is the stream of messages going out.
                // `stream` is the stream of incoming messages.
                let (sink, stream) = ws_stream.split();

                // We forward all messages, composed out of the data, entered to
                // the stdin, to the `sink`.
                let send_stdin = stdin_rx.forward(sink);
                let write_stdout = stream.for_each(move |message| {
                    let sender = msg_send.clone();
                    match sender.send(message).wait() {
                        Ok(_) => {
                            println!("0");
                        },
                        Err(_) => {
                            println!("channel closed");
                        },
                    };
                    Ok(())
                });

                // Wait for either of futures to complete.
                send_stdin
                    .map(|_| ())
                    .select(write_stdout.map(|_| ()))
                    .then(|_| Ok(()))
            })
            .map_err(|e| {
                println!("Error during the websocket handshake occurred: {}", e);
                io::Error::new(io::ErrorKind::Other, e)
            });

        // And now that we've got our client, we execute it in the event loop!
        thread::spawn(move || {
            tokio::runtime::run(client.map_err(|_e| ()));
        });
        let mut tc = TwitchClient{
            sender: stdin_tx.clone(),
            rec: msg_r,
            user: usr.to_string().clone(),
        };

        
        let twitch_auth = format!("PASS {}",auth);
        tc.send_raw(Message::Text(twitch_auth.into()));

        let twitch_nick = format!("NICK {}",usr);
        println!("sending the {}",twitch_nick);
        tc.send_raw(Message::Text(twitch_nick.into()));

        let twitch_join = format!("JOIN #{}",usr);
        println!("sending the {}", twitch_join);
        tc.send_raw(Message::Text(twitch_join.into()));

        return tc;
    }
}