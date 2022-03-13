use std::{io, sync::Arc, str};
use tokio::{net::UdpSocket, sync::mpsc};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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
        let addrd = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9898);
        println!("{:?} bytes received from {:?}", len, addr);
        let s = match str::from_utf8(&buf) {
            Ok(v) => v,
            Err(_) => "",
        };
        match &s[..5] {
            "flood" => {
                println!("A flood is received");
                let flood_buf = [0xFFu8; 1024 * 5];
                tx.send((buf[..len].to_vec(), addr)).await.unwrap();
                for _ in 0..10 {
                    tx.send(("flood".as_bytes().to_vec(), addrd)).await.unwrap();
                }
            },
            _ => {},
        }
    }
}
