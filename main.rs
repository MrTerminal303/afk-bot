use azalea::prelude::*;
use std::env;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

#[derive(Default, Clone, Component)]
struct State {}

async fn handle(bot: Client, event: Event, _state: State) -> anyhow::Result<()> {
    match event {
        Event::Login => {
            println!("Bot connected.");
        }
        Event::Disconnect(reason) => {
            println!("Disconnected: {reason:?}");
        }
        _ => {}
    }
    Ok(())
}

fn spawn_health_server() {
    // Render needs an open HTTP port to detect the service as "up".
    thread::spawn(|| {
        let port = env::var("PORT").unwrap_or_else(|_| "10000".to_string());
        let listener = TcpListener::bind(format!("0.0.0.0:{port}")).unwrap();
        for stream in listener.incoming().flatten() {
            use std::io::Write;
            let mut stream = stream;
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK",
            );
        }
    });
}

#[tokio::main]
async fn main() {
    spawn_health_server();

    let server_addr = env::var("MC_SERVER").expect("Set MC_SERVER env var, e.g. play.example.com:25565");
    let username = env::var("MC_USERNAME").unwrap_or_else(|_| "AfkBot".to_string());

    loop {
        let account = Account::offline(&username);
        println!("Connecting to {server_addr} as {username}...");

        let result = ClientBuilder::new()
            .set_handler(handle)
            .start(account, server_addr.as_str())
            .await;

        if let Err(e) = result {
            println!("Connection ended: {e:?}. Reconnecting in 10s...");
        } else {
            println!("Session ended cleanly. Reconnecting in 10s...");
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}