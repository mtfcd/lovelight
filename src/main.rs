#[macro_use] extern crate rocket;
use std::vec;

use tokio::{net::{TcpListener}, io::AsyncWriteExt, sync::mpsc};

struct Switch {
    listener: TcpListener,
    receiver: mpsc::Receiver<u8>
}

impl Switch {
    async fn new(rev: mpsc::Receiver<u8>) ->Self {
        let switch = Self {
            listener: TcpListener::bind("0.0.0.0:6379").await.unwrap(),
            receiver: rev
        };

        return switch
    }

    async fn serv(&mut self) {
        // The second item contains the IP and port of the new connection.
        dbg!(1);

        let (mut socket, _) = self.listener.accept().await.unwrap();
        dbg!(2);
        while let Some(message) = self.receiver.recv().await {
            dbg!(3);
            println!("GOT = {}", message);
            let command = [message];
            socket.write_all(&command).await.unwrap();
            dbg!(4);
        }
    }

}

#[get("/")]
async fn index(switch: &rocket::State<mpsc::Sender<u8>>) -> &'static str {
    switch.send(65).await.unwrap();
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    let (tx, rx) = mpsc::channel(32);
    let mut s = Switch::new(rx).await;
    tokio::spawn(async move {
        s.serv().await;
    });
    
    rocket::build().manage(tx).mount("/", routes![index])
}