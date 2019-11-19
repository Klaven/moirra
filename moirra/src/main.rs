use structopt::StructOpt;
use std::net::TcpListener;
use std::env;
use url::Url;
use std::thread;
use std::io::{self, Read, Write};

use futures::sync::mpsc;
use futures::{Future, Sink, Stream};
use tungstenite::protocol::Message;

use tokio_tungstenite::connect_async;
use tokio_tungstenite::stream::PeerAddr;

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
    
    let (stdin_tx, stdin_rx) = mpsc::channel(0);
    thread::spawn(|| read_stdin(stdin_tx));
    let stdin_rx = stdin_rx.map_err(|_| panic!());

    let mut stdout = io::stdout();
    let client = connect_async(Url::parse("wss://irc-ws.chat.twitch.tv:443").unwrap())
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
            //let send_stdin = stdin_rx.forward(sink);
            let write_stdout = stream.for_each(move |message| {
                stdout.write_all(&message.into_data()).unwrap();
                Ok(())
            });

            sink.send(Message::binary("test"))
                .then(|_| Ok(()))

            // Wait for either of futures to complete.
            /*
            send_stdin
                .map(|_| ())
                .select(write_stdout.map(|_| ()))
                .then(|_| Ok(()))
            */
        })
        .map_err(|e| {
            println!("Error during the websocket handshake occurred: {}", e);
            io::Error::new(io::ErrorKind::Other, e)
        });

    // And now that we've got our client, we execute it in the event loop!
    tokio::runtime::run(client.map_err(|_e| ()));

/*
    let (mut socket, response) = connect(Url::parse("wss://irc-ws.chat.twitch.tv:443").unwrap()).expect("Can't connect");

    let twitch_auth = format!("PASS {}",res);

    socket
        .write_message(Message::Text(twitch_auth.into()))
        .unwrap();
    socket.write_message(Message::Text("NICK klavenx".into())).unwrap();

    socket.write_message(Message::Text("JOIN #klavenx".into())).unwrap();
    socket.write_message(Message::Text("PRIVMSG #klavenx asdf".into())).unwrap();

    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }

    println!("Hello, world!");
*/
}


fn read_stdin(mut tx: mpsc::Sender<Message>) {
    let mut stdin = io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf) {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx = tx.send(Message::binary(buf)).wait().unwrap();
    }
}
