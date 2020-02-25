use structopt::StructOpt;
use std::net::TcpListener;
use std::env;
use std::{thread,time};
use std::io::{self, Read, Write};

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

    // twitch_client.done().await;
    loop {
      thread::sleep_ms(30000);
    }
}

fn read(msg: &str) {
    println!("{}",msg);
}


