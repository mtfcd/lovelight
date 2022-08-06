#[macro_use] extern crate rocket;
use std::{vec, sync::Arc};

use tokio::{net::TcpListener, sync::{mpsc, Mutex}, io::AsyncWriteExt};


#[get("/on")]
async fn switch_on(switch: &rocket::State<mpsc::Sender<u8>>) -> &'static str {
    switch.send(1).await.unwrap();
    "ok"
}
#[get("/off")]
async fn switch_off(switch: &rocket::State<mpsc::Sender<u8>>) -> &'static str {
    switch.send(0).await.unwrap();
    "ok"
}

#[launch]
async fn rocket() -> _ {
    let (tx, mut rx) = mpsc::channel(32);
    let listener  = TcpListener::bind("0.0.0.0:10002").await.unwrap();

    let clients = Arc::new(Mutex::new(vec![]));
    let new_clients = clients.clone();
    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            dbg!("new connection.");
            new_clients.lock().await.push(stream);
        }
    });
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await  {
            println!("GOT = {}", message);
            let command = [message];
            let mut streams = clients.lock().await;
            for (i, stream) in streams.iter_mut().enumerate() {
                dbg!("stream", i);
                match stream.write_all(&command).await {
                    Ok(_) => {
                        println!("command sent.");
                    },
                    Err(e) => {
                        println!("error {}", e);
                    },
                }
            }
        }
    });
    
    rocket::build().manage(tx).mount("/", routes![switch_on, switch_off])
}