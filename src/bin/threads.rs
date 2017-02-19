extern crate thrench;

use std::thread;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::net::{TcpListener, TcpStream};

use thrench::Packet;

#[derive(Debug)]
enum BrokerMessage {
    NewPeer(SyncSender<Packet>),
    Broadcast(Packet)
}

fn main() {
    let addr = "0.0.0.0:20053";
    println!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).unwrap();

    let broker = spawn_broker();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        stream.set_nodelay(true).unwrap();
        spawn_peer(broker.clone(), stream);
    }
}

fn spawn_broker() -> SyncSender<BrokerMessage> {
    let (tx, rx) = sync_channel(128);
    thread::spawn(move || {
        let mut peers = vec![];
        loop {
            let message = rx.recv().unwrap();
            match message {
                BrokerMessage::NewPeer(peer) => peers.push(peer),
                BrokerMessage::Broadcast(p) => for peer in peers.iter() {
                    peer.send(p).unwrap();
                }
            }
        }
    });
    tx
}

fn spawn_peer(broker: SyncSender<BrokerMessage>, mut sock: TcpStream) {
    thread::Builder::new()
        .stack_size(1)
        .spawn(move || {
            let (tx, rx) = sync_channel(8);
            broker.send(BrokerMessage::NewPeer(tx)).unwrap();
            let packet = Packet::read(&mut sock);
            broker.send(BrokerMessage::Broadcast(packet)).unwrap();

            loop {
                let packet = rx.recv().unwrap();
                packet.write(&mut sock);
            }
        }).unwrap();
}
