use std::env;
use std::{thread,time};

use url::Url;

use async_tungstenite::tokio::connect_async;
use async_tungstenite::tungstenite::Message;
use futures::prelude::*;
use tokio::prelude::*;
use tokio::sync::mpsc;

pub struct TwitchClient {
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
    rec: tokio::sync::mpsc::UnboundedReceiver<Message>,
    user: String,
}

impl TwitchClient {

    pub async fn send(&self, msg: &str) -> Result<(),()> {
        let msg_fmt = format!("PRIVMSG #{} :{}", self.user, msg);
        let ws_msg = Message::Text(msg_fmt.clone().into());
        println!("private message sent: {}", msg_fmt);
        return self.send_raw(ws_msg).await;
    }

    pub async fn send_raw(&self, msg: Message) -> Result<(),()> {
        let sender = self.sender.clone();

        match sender.send(msg) {
            Ok(_) => {
                println!("ws write ok");
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

        let (tx, rx): (tokio::sync::mpsc::UnboundedSender<Message>,tokio::sync::mpsc::UnboundedReceiver<Message>)
                       = tokio::sync::mpsc::unbounded_channel(); 
        let (output_tx, output_rx):(tokio::sync::mpsc::UnboundedSender<Message>,tokio::sync::mpsc::UnboundedReceiver<Message>) = tokio::sync::mpsc::unbounded_channel();

        let (ws_stream, _) = connect_async(url)
            .await
            .expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");

        let (write, read) = ws_stream.split();

        let something = rx.map(Ok).forward(write);
        //let ws_in = rx.forward(write);
        
        let ws_to_stdout = {
            read.for_each(|message| async {
                let mg = message.unwrap().clone();
                let data = mg.clone().into_data();
                println!("{:?}",&data);
            })
        };
        
        let handle = tokio::task::spawn( ws_to_stdout );

        let mut tc = TwitchClient{
            sender: tx.clone(),
            rec: output_rx,
            user: usr.to_string().clone(),
        };

        let twitch_auth = format!("PASS {}",auth);
        tc.send_raw(Message::Text(twitch_auth.into())).await;

        let twitch_nick = format!("NICK {}",usr);
        println!("sending the {}",twitch_nick);
        tc.send_raw(Message::Text(twitch_nick.into())).await;

        let twitch_join = format!("JOIN #{}",usr);
        println!("sending the {}", twitch_join);
        tc.send_raw(Message::Text(twitch_join.into())).await;

        return tc;
    }
}
