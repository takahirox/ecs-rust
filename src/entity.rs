pub struct Entity {
	alive: bool
}

impl Entity {
	pub fn new() -> Self {
		Entity {
			alive: true
		}
	}

	pub fn is_alive(&self) -> bool {
		self.alive
	}

	pub fn invalid(&mut self) {
		self.alive = false;
	}

	pub fn reset(&mut self) {
		self.alive = true;
	}
}
