use std::{io, net::SocketAddr, sync::Arc, str};
use tokio::{net::UdpSocket, sync::mpsc};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listen_addr = "0.0.0.0:9898";
    let socket = UdpSocket::bind(listen_addr.parse::<SocketAddr>().unwrap()).await?;
    let r = Arc::new(socket);
    let s = r.clone();
    let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
    tokio::spawn(async move {
        while let Some((bytes, addr)) = rx.recv().await {
            let len = s.send_to(&bytes, &addr).await.unwrap();
            println!("{:?} bytes echoed", len);
        }
    });

    let mut buf = [0; 65536];
    println!("Listening at {}", listen_addr);
    loop {
        let (len, addr) = r.recv_from(&mut buf).await?;
        println!("{:?} bytes received from {:?}", len, addr);
        let s = match str::from_utf8(&buf) {
            Ok(v) => v,
            Err(_) => "",
        };
        match &s[..5] {
            "flood" => {
                println!("A flood is received");
                let flood_buf = [0xFFu8; 1024];
                tx.send((buf[..len].to_vec(), addr)).await.unwrap();
                for _ in 0..10 {
                    tx.send((flood_buf[..1024].to_vec(), addr)).await.unwrap();
                }
            },
            _ => {},
        }
    }
}
