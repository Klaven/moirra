use structopt::StructOpt;
use std::net::TcpListener;
use std::env;
use std::{thread,time};
use std::io::{self, Read, Write};

use tungstenite::{connect, Message};
use url::Url;

use futures::sync::mpsc;
use futures::{Future, Sink, Stream};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::stream::PeerAddr;
use chat::clients::twitch_client::TwitchClient;

#[derive(Debug,StructOpt)]
struct Cli {
    #[structopt(long = "config", short = "c")]
    config: Option<String>,
}

fn main() {
    let args = Cli::from_args();

    let key = "TWITCH_TOKEN";
    let res = env::var(key).expect("Need TWITCH_TOKEN env var");

    match args.config {
        Some(_) => println!("lajsdf;"),
        _ => println!("no match"),
    }
    let twitch_client = TwitchClient::new("wss://irc-ws.chat.twitch.tv:443",&res,"klavenx");

    println!("Hello, world!");

    loop {};
}

