use std::vec;
use super::entity::Entity;

pub struct EntityManager {
	entities: Vec<Entity>
}

impl EntityManager {
	pub fn new() -> Self {
		EntityManager {
			entities: vec![]
		}
	}

	pub fn create(&mut self) -> usize {
		let entity = Entity::new();
		self.entities.push(entity);
		self.entities.len() - 1
	}
}