use super::entity_manager::EntityManager;
use super::component::Component;
use super::system::System;

pub struct World {
	entity_manager: EntityManager,
	systems: Vec<Box<dyn System>>
}

impl World {
	pub fn new() -> Self {
		World {
			entity_manager: EntityManager::new(),
			systems: vec![]
		}
	}

	pub fn create_entity(&mut self) -> usize {
		self.entity_manager.create_entity()
	}

	pub fn remove_entity(&mut self, entity_id: usize) {
		self.entity_manager.remove_entity(entity_id);
	}

	pub fn register_component<T: 'static + Component>(&mut self) -> &mut Self {
		self.entity_manager.register::<T>();
		self
	}

	pub fn add_system<T: 'static + System>(&mut self, system: T) -> &mut Self {
		self.systems.push(Box::new(system));
		self
	}

	pub fn add_component_to_entity<T: 'static + Component>(&mut self, entity_id: usize, component: T) -> &mut Self {
		self.entity_manager.add_component_to_entity(entity_id, component);
		self
	}

	pub fn update(&mut self) {
		for system in self.systems.iter_mut() {
			system.update(&mut self.entity_manager);
		}
	}
}
