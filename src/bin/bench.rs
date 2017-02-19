extern crate byteorder;
extern crate rayon;

use std::io::{Write, Read};
use std::net;
use std::str::FromStr;
use std::time;

use byteorder::{LittleEndian, WriteBytesExt};
use rayon::prelude::*;

fn main() {
    c10k();
    println!("\n\nBenchmarks finished");
}

fn c10k() {
    let addr: net::SocketAddr = FromStr::from_str("0.0.0.0:20053").unwrap();
    let mut socks = Vec::new();
    let message = {
        let hello = b"Hello, World!\n";
        let mut buff = vec![];
        buff.write_u32::<LittleEndian>(hello.len() as u32).unwrap();
        buff.write(hello).unwrap();
        buff
    };
    let start = time::Instant::now();
    for n_cons in 0..3000 {
        if n_cons % 50 == 0 {
            println!("{} concurrent connections, {:.2} seconds",
                     n_cons, (time::Instant::now() - start).as_secs());
        }
        let mut sock = net::TcpStream::connect(&addr).unwrap();
        if n_cons + 1 == 10_000 {
            println!("c10k!");
        }
        sock.write_all(&message).unwrap();
        socks.push(sock);
        socks.par_iter_mut().for_each(|sock| {
            let mut buff = [0u8; 128];
            sock.read_exact(&mut buff[..message.len()]).unwrap();
            assert_eq!(&buff[..message.len()], &message[..]);
        });
    }

    let end = time::Instant::now();
    let duration = end - start;

    println!("time {:.2} seconds", duration.as_secs());
}
