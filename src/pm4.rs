use std::option::Option;
use std::iter::Iterator;
use num;
use cs::Packet3;

pub struct Packet {
	pub header: u32,
	pub words: Vec<u32>
}

pub fn split(stream: &mut Iterator<Item=u32>) -> Vec<Packet> {
	let mut packets = Vec::new();
	while let Some(packet_header) = stream.next() {
		let packet_type = packet_header >> 30;
		let packet_length = ((packet_header >> 16) & 0x3fff) + 1u32;
		if packet_type != 3u32 { panic!("can't handle"); }
		let packet: Vec<u32> = stream.take(packet_length as usize).collect();
		packets.push(Packet{header: packet_header, words: packet});
	}
	packets
}

pub fn split_and_print(stream: &[u32]) {
	for packet in split(&mut stream.iter().map(|a|*a)) {
		let packet_type: Option<Packet3> = num::FromPrimitive::from_u32((packet.header >> 8) & 0xff);
		println!("{:08x} {:?}", packet.header, packet_type);
		for word in packet.words {
			println!("{:08x}", word);
		}
		println!("");
	}
}
