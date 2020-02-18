use std::env;
use std::{thread,time};

use url::Url;

use futures::{Future, Sink, Stream};
use async_tungstenite::tokio::connect_async;
use async_tungstenite::tungstenite::Message;

use tokio::sync::mpsc;

pub struct TwitchClient {
    sender: mpsc::Sender<Message>,
    rec: mpsc::Receiver<Message>,
    user: String,
}

impl TwitchClient {

    pub async fn send(&self, msg: &str) -> Result<(),()> {
        let ws_msg = Message::Text(format!("PRIVMSG #{} :{}", self.user, msg).into());
        println!("private message sent: {}", format!("PRIVMSG #{} {}", self.user, msg));
        return self.send_raw(ws_msg).await;
    }

    pub async fn send_raw(&self, msg: Message) -> Result<(),()> {
        let sender = self.sender.clone();

        match sender.send(msg).await {
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

    pub async fn new(url: &str, auth: &str, usr: &str) -> TwitchClient {

        let connect_addr = env::args()
            .nth(1)
            .unwrap_or_else(|| panic!("this program requires at least one argument"));

        let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();

        let (ws_stream, _) = connect_async(url)
            .await
            .expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");

        let (write, read) = ws_stream.split();

        let stdin_to_ws = stdin_rx.map(Ok).forward(write);
        let ws_to_stdout = {
            read.for_each(|message| async {
                let data = message.unwrap().into_data();
                async_std::io::stdout().write_all(&data).await.unwrap();
            })
        };

        pin_mut!(stdin_to_ws, ws_to_stdout);
        future::select(stdin_to_ws, ws_to_stdout).await;



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
