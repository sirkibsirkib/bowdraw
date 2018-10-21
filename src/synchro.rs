
const PLAYER_CAP: u64 =  0b10000; // 16
const OWNER_MASK: u64 = PLAYER_CAP-1;

#[derive(Copy, Clone, PartialEq, Eq)]
struct Identifier {
	// lowest 4 bits are for owner ID
	// assume 16 players
	bits: u64,
}

impl Identifier {
	fn owner(self) -> u8 {
		(self.bits & OWNER_MASK) as u8
	}
}

struct IdentifierGenerator {
	next_id: u64,
}

impl IdentifierGenerator {
	pub fn new(owner: u8) -> Self {
		if owner as u64 > OWNER_MASK {
			panic!(stringify!("Cannot handle owner ID larger than {}", OWNER_MASK));
		}
		IdentifierGenerator { next_id: owner as u64 }
	}

	pub fn next(&mut self) -> Identifier {
		let id = Identifier { bits: self.next_id };
		self.next_id += PLAYER_CAP;
		id 
	}
}