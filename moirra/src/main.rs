use structopt::StructOpt;
use std::net::TcpListener;
use std::env;
use std::{thread,time};
use std::io::{self, Read, Write};

use url::Url;
use async_tungstenite::tokio::connect_async;
use async_tungstenite::tungstenite::Message;

use futures::sync::mpsc;
use futures::{Future, Sink, Stream};
use chat::clients::twitch_client::TwitchClient;

#[derive(Debug,StructOpt)]
struct Cli {
    #[structopt(long = "config", short = "c")]
    config: Option<String>,

    #[structopt(long = "user", short = "u")]
    user: Option<String>,

    
}

#[tokio::main]
async fn main() {
    let args = Cli::from_args();

    let key = "TWITCH_TOKEN";
    let res = env::var(key).expect("Need TWITCH_TOKEN env var");

    match args.config {
        Some(_) => println!("lajsdf;"),
        _ => println!("no match"),
    }

    //TODO: make user required
    let user = match args.user {
        Some(usr) => usr,
        _ => "klavenx".to_string(),
    };

    let twitch_client = TwitchClient::new("wss://irc-ws.chat.twitch.tv:443",&res,&user).await;

    let res = twitch_client.send("whats up twitch").await;

    println!("test: {:?}", res);
    println!("Hello, world!");
    
    twitch_client.done().await;

    loop {};
}

fn read(msg: &str) {
    println!("{}",msg);
}


