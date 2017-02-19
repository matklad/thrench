extern crate byteorder;

use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

const PACKET_LIMIT: usize = 32;

#[derive(Clone, Copy, Debug)]
pub struct Packet {
    len: usize,
    data: [u8; PACKET_LIMIT],
}

impl Packet {
    pub fn new() -> Packet {
        Packet { len: 0, data: [0; PACKET_LIMIT] }
    }

    pub fn write<W: Write>(&self, w: &mut W) {
        w.write_u32::<LittleEndian>(self.len as u32)
            .expect("Failed to write packet len");

        w.write_all(&self.data[..self.len])
            .expect("Failed to write packet data");
    }

    pub fn read<R: Read>(r: &mut R) -> Packet {
        let mut result = Packet { len: 0, data: [0; PACKET_LIMIT] };
        let len = r.read_u32::<LittleEndian>()
            .expect("Failed to read packet len") as usize;
        result.len = len;
        r.read_exact(&mut result.data[..len])
            .expect("Failed to read packet data");
        result
    }
}

