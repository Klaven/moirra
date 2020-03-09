use std::env;
use std::{thread,time};

use url::Url;

use async_tungstenite::tokio::connect_async;
use async_tungstenite::tungstenite::Message;
use futures::prelude::*;
use tokio::prelude::*;
use tokio::sync::mpsc;
use async_tungstenite::WebSocketStream;

pub struct TwitchClient {
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
    user: String,
    ws_done_join_handle: tokio::task::JoinHandle<()>,
}

impl TwitchClient {

    pub fn done(self) -> tokio::task::JoinHandle<()> {
        self.ws_done_join_handle
    }

    pub fn send(&self, msg: &str) -> Result<(),()> {
        let msg_fmt = format!("PRIVMSG #{} :{}\r\n", self.user, msg);
        let ws_msg = Message::Text(msg_fmt.clone().into());
        println!("private message sent: {}", msg_fmt);
        return self.send_raw(ws_msg);
    }

    pub fn send_raw(&self, msg: Message) -> Result<(),()> {
        let sender = self.sender.clone();

        match sender.send(msg) {
            Ok(_) => {
                println!("ws channel write ok");
            },
            Err(err) => {
                println!("channel closed: {:?}", err);
                return Err(());
            },
        };
        return Ok(());
    }

    pub async fn new(url: &str, auth: &str, usr: &str) -> TwitchClient {

        let (tx, rx): (tokio::sync::mpsc::UnboundedSender<Message>,tokio::sync::mpsc::UnboundedReceiver<Message>)
                       = tokio::sync::mpsc::unbounded_channel(); 
        let (output_tx, output_rx):(tokio::sync::mpsc::UnboundedSender<Message>,tokio::sync::mpsc::UnboundedReceiver<Message>) = tokio::sync::mpsc::unbounded_channel();

        let (ws_stream, _) = connect_async(url)
            .await
            .expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");

        let (write, read) = ws_stream.split();

        let something = rx.map(Ok).forward(write);
        
        let parsing_future = {
            output_rx.for_each(|message| async {
                        let data = message.clone().into_data();
                        println!("On WS Message: {:?}",std::str::from_utf8(&data).unwrap());
            })
        };

        let output = output_tx.clone();
        let ws_to_stdout = {
            read.for_each(move |message| async {
                match message {
                    Ok(data) => {
                        output.send(data);
                    },
                    Err(err) => println!("channel closed? {}", err)
                }
            })
        };

        let fut = async move { ws_to_stdout.await; println!("websocked closed");};

        let h2 = tokio::task::spawn(something);
        let handle = tokio::task::spawn(fut);

        let mut tc = TwitchClient{
            sender: tx.clone(),
            user: usr.to_string().clone(),
            ws_done_join_handle: handle,
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
