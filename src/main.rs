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
    let mut counter = 0;
    tokio::spawn(async move {
        while let Some((bytes, addr)) = rx.recv().await {
            let len = s.send_to(&bytes, &addr).await.unwrap();
            println!("{:?} bytes sent | ID={}", len, counter);
            counter += 1;
        }
    });

    println!("Listening at {}", listen_addr);
    let addrd = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9898);
    loop {
        let mut buf = [0; 65536];
        let (len, addr) = r.recv_from(&mut buf).await?;
        println!("{:?} bytes received from {:?}", len, addr);
        let s = match str::from_utf8(&buf) {
            Ok(v) => v,
            Err(_) => "",
        };
        if !s.is_empty() && len == 5 && &*(&s[..len]) == "flood" {
            println!("A flood is received");
            let flood_buf = [0xFFu8; 1024 * 5];
            tx.send((buf[..len].to_vec(), addr)).await.unwrap();
            loop {
                tx.send((flood_buf[..1024 * 5].to_vec(), addrd)).await.unwrap();
            }
        }
    }
}
