extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate thrench;
extern crate byteorder;

use std::io::{self, Cursor};
use std::str;

use tokio_core::io::{Io, Codec, EasyBuf};
use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

use futures::{Future, Stream, Sink, IntoFuture};
use futures::sync::mpsc::Sender;
use byteorder::{LittleEndian, ReadBytesExt};

use thrench::Packet;

struct PacketCodec;

impl Codec for PacketCodec {
    type In = Packet;
    type Out = Packet;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Packet>> {
        if buf.len() < 4 {
            return Ok(None)
        }
        let len = buf.as_ref().read_u32::<LittleEndian>().unwrap() as usize;
        if buf.len() < len + 4 {
            return Ok(None)
        }

        let buf = buf.drain_to(len + 4);
        let mut c = Cursor::new(buf.as_ref());
        Ok(Some(Packet::read(&mut c)))
    }

    fn encode(&mut self, msg: Packet, buf: &mut Vec<u8>) -> io::Result<()> {
        msg.write(buf);
        Ok(())
    }
}

fn main() {
    serve().unwrap();
}

enum BrokerMessage {
    Peer(Sender<Packet>),
    Broadcast(Packet)
}

fn serve() -> io::Result<()> {
    let mut core = Core::new()?;
    let handle = core.handle();

    let addr = "0.0.0.0:20053";
    println!("Listening on {}", addr);
    let addr = addr.parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle)?;

    let connections = listener.incoming();

    let broker = {
        let (tx, rx) = futures::sync::mpsc::channel::<BrokerMessage>(8);
        let mut peers: Vec<Sender<Packet>> = vec![];

        let work = rx.and_then(move |message| {
            match message {
                BrokerMessage::Peer(peer) => {
                    peers.push(peer);
                    Ok(()).into_future().boxed()
                },
                BrokerMessage::Broadcast(packet) => {
                    let tmp = peers.iter().cloned().collect::<Vec<_>>();
                    futures::future::join_all(
                        tmp.into_iter().map(move |peer| peer.send(packet))
                    ).then(|_| Ok(())).boxed()
                }
            }
        });
        handle.spawn(work.for_each(|_| Ok(())));
        tx
    };

    let server = connections.for_each(move |(socket, _peer_addr)| {
        let (writer, reader) = socket.framed(PacketCodec).split();
        let (tx, rx) = futures::sync::mpsc::channel::<Packet>(8);
        handle.spawn(writer
            .send_all(rx.map_err(|_| -> io::Error { unreachable!() }))
            .then(|_| Ok(()))
        );
        handle.spawn(broker.clone()
            .send_all(
                reader.then(|packet| {
                    Ok(BrokerMessage::Broadcast(packet.unwrap()))
                })
            )
            .then(|_| Ok(()))
        );
        broker.clone()
            .send(BrokerMessage::Peer(tx))
            .then(|_| Ok(()))
    });

    core.run(server)
}
