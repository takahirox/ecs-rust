use std::collections::HashMap;
use std::any::TypeId;
use std::mem::transmute;
use std::vec;

use super::entity::Entity;
use super::component::Component;
use super::component_manager::{ComponentManager, ComponentManagerTrait};

pub struct EntityManager {
	entities: Vec<Entity>,
	manager_map: HashMap<TypeId, Box<dyn ComponentManagerTrait>>
}

impl EntityManager {
	pub fn new() -> Self {
		EntityManager {
			entities: vec![],
			manager_map: HashMap::new()
		}
	}

	pub fn register<T: 'static + Component>(&mut self) -> &mut Self {
		// @TODO: Error handling if already registered?
		if ! self.has_component_manager::<T>() {
			let type_id = TypeId::of::<T>();
			self.manager_map.insert(type_id, Box::new(ComponentManager::<T>::new()));
		}
		self
	}

	pub fn create_entity(&mut self) -> usize {
		let entity = Entity::new();
		self.entities.push(entity);
		self.entities.len() - 1
	}

	pub fn add_component_to_entity<T: 'static + Component>(&mut self, entity_id: usize, component: T) -> &mut Self {
		if ! self.has_component_manager::<T>() {
			// @TODO: Better error handling
			println!("Unknown component");
			return self;
		}
		self.borrow_component_manager_mut::<T>()
			.add(entity_id, component);
		self
	}

	pub fn borrow_entity_ids<T: 'static + Component>(&self) -> &Vec<usize> {
		if ! self.has_component_manager::<T>() {
			// @TODO: Better error handling
			panic!("Unknown component");
		}

		self.borrow_component_manager::<T>().borrow_entity_ids()
	}

	// @TODO: Optimize. Doing this in every world.update() is very inefficient.
	pub fn get_entity_ids_for_pair<T: 'static + Component, U: 'static + Component>(&self) -> Vec<usize> {
		let mut v = Vec::new();

		if ! self.has_component_manager::<T>() ||
			! self.has_component_manager::<U>() {
			// @TODO: Better error handling
			println!("Unknown component");
			return v;
		}

		let entity_ids = self.borrow_component_manager::<T>().borrow_entity_ids();
		let manager = self.borrow_component_manager::<U>();
		for id in entity_ids.iter() {
			if manager.has(*id) {
				v.push(*id);
			}
		}
		v
	}

	// @TODO: Optimize. Doing this in every world.update() is very inefficient.
	pub fn get_entity_ids_for_triple
		<T: 'static + Component, U: 'static + Component, V: 'static + Component>(&self) -> Vec<usize> {
		let mut v = Vec::new();

		if ! self.has_component_manager::<T>() ||
			! self.has_component_manager::<U>() ||
			! self.has_component_manager::<V>() {
			// @TODO: Better error handling
			println!("Unknown component");
			return v;
		}

		let entity_ids = self.borrow_component_manager::<T>().borrow_entity_ids();
		let manager1 = self.borrow_component_manager::<U>();
		let manager2 = self.borrow_component_manager::<V>();
		for id in entity_ids.iter() {
			if manager1.has(*id) && manager2.has(*id) {
				v.push(*id);
			}
		}
		v
	}

	// @TODO: Optimize. Doing this in every world.update() is very inefficient.
	pub fn get_entity_ids_for_quad
		<T: 'static + Component, U: 'static + Component, V: 'static + Component, W: 'static + Component>(&self) -> Vec<usize> {
		let mut v = Vec::new();

		if ! self.has_component_manager::<T>() ||
			! self.has_component_manager::<U>() ||
			! self.has_component_manager::<V>() ||
			! self.has_component_manager::<W>() {
			// @TODO: Better error handling
			println!("Unknown component");
			return v;
		}

		let entity_ids = self.borrow_component_manager::<T>().borrow_entity_ids();
		let manager1 = self.borrow_component_manager::<U>();
		let manager2 = self.borrow_component_manager::<V>();
		let manager3 = self.borrow_component_manager::<W>();
		for id in entity_ids.iter() {
			if manager1.has(*id) && manager2.has(*id) && manager3.has(*id) {
				v.push(*id);
			}
		}
		v
	}

	pub fn borrow_component<T: 'static + Component>(&self, entity_id: usize) -> Option<&T> {
		match self.has_component_manager::<T>() {
			true => self.borrow_component_manager::<T>()
				.borrow_component(entity_id),
			false => None
		}
	}

	pub fn borrow_component_mut<T: 'static + Component>(&mut self, entity_id: usize) -> Option<&mut T> {
		match self.has_component_manager::<T>() {
			true => self.borrow_component_manager_mut::<T>()
				.borrow_component_mut(entity_id),
			false => None
		}
	}

	pub fn borrow_components<T: 'static + Component>(&self) -> Option<&Vec<T>> {
		match self.has_component_manager::<T>() {
			true => Some(
				self.borrow_component_manager::<T>()
					.borrow_components()
			),
			false => None
		}
	}

	pub fn borrow_components_mut<T: 'static + Component>(&mut self) -> Option<&mut Vec<T>> {
		match self.has_component_manager::<T>() {
			true => Some(
				self.borrow_component_manager_mut::<T>()
					.borrow_components_mut()
			),
			false => None
		}
	}

	pub fn borrow_component_pair_mut
		<T: 'static + Component, U: 'static + Component>(&mut self, entity_id: usize)
		-> Option<(&mut T, &mut U)> {
		if ! self.has_component_manager::<T>() ||
			! self.has_component_manager::<U>() {
			return None;
		}

		let type_id1 = TypeId::of::<T>();
		let type_id2 = TypeId::of::<U>();

		let manager1 = cast_manager_mut_unsafe(self.manager_map.get(&type_id1).unwrap());
		let manager2 = cast_manager_mut_unsafe(self.manager_map.get(&type_id2).unwrap());

		if !manager1.has(entity_id) || !manager2.has(entity_id) {
			return None;
		}

		Some((
			manager1.borrow_component_mut(entity_id).unwrap(),
			manager2.borrow_component_mut(entity_id).unwrap()
		))
	}

	pub fn borrow_component_triple_mut
		<T: 'static + Component, U: 'static + Component, V: 'static + Component>
		(&mut self, entity_id: usize)
		-> Option<(&mut T, &mut U, &mut V)> {
		if ! self.has_component_manager::<T>() ||
			! self.has_component_manager::<U>() ||
			! self.has_component_manager::<V>() {
			return None;
		}

		let type_id1 = TypeId::of::<T>();
		let type_id2 = TypeId::of::<U>();
		let type_id3 = TypeId::of::<V>();

		let manager1 = cast_manager_mut_unsafe(self.manager_map.get(&type_id1).unwrap());
		let manager2 = cast_manager_mut_unsafe(self.manager_map.get(&type_id2).unwrap());
		let manager3 = cast_manager_mut_unsafe(self.manager_map.get(&type_id3).unwrap());

		if !manager1.has(entity_id) || !manager2.has(entity_id) || !manager3.has(entity_id) {
			return None;
		}

		Some((
			manager1.borrow_component_mut(entity_id).unwrap(),
			manager2.borrow_component_mut(entity_id).unwrap(),
			manager3.borrow_component_mut(entity_id).unwrap()
		))
	}

	pub fn borrow_component_quad_mut
		<T: 'static + Component, U: 'static + Component, V: 'static + Component, W: 'static + Component>
		(&mut self, entity_id: usize)
		-> Option<(&mut T, &mut U, &mut V, &mut W)> {
		if ! self.has_component_manager::<T>() ||
			! self.has_component_manager::<U>() ||
			! self.has_component_manager::<V>() ||
			! self.has_component_manager::<W>() {
			return None;
		}

		let type_id1 = TypeId::of::<T>();
		let type_id2 = TypeId::of::<U>();
		let type_id3 = TypeId::of::<V>();
		let type_id4 = TypeId::of::<W>();

		let manager1 = cast_manager_mut_unsafe(self.manager_map.get(&type_id1).unwrap());
		let manager2 = cast_manager_mut_unsafe(self.manager_map.get(&type_id2).unwrap());
		let manager3 = cast_manager_mut_unsafe(self.manager_map.get(&type_id3).unwrap());
		let manager4 = cast_manager_mut_unsafe(self.manager_map.get(&type_id4).unwrap());

		if !manager1.has(entity_id) || !manager2.has(entity_id) || !manager3.has(entity_id) || !manager4.has(entity_id) {
			return None;
		}

		Some((
			manager1.borrow_component_mut(entity_id).unwrap(),
			manager2.borrow_component_mut(entity_id).unwrap(),
			manager3.borrow_component_mut(entity_id).unwrap(),
			manager4.borrow_component_mut(entity_id).unwrap()
		))
	}

	fn has_component_manager<T: 'static + Component>(&self) -> bool {
		let type_id = TypeId::of::<T>();
		self.manager_map.contains_key(&type_id)
	}

	fn borrow_component_manager<T: 'static + Component>(&self) -> &ComponentManager<T> {
		let type_id = TypeId::of::<T>();
		cast_manager(self.manager_map.get(&type_id).unwrap())
	}

	fn borrow_component_manager_mut<T: 'static + Component>(&mut self) -> &mut ComponentManager<T> {
		let type_id = TypeId::of::<T>();
		cast_manager_mut(self.manager_map.get_mut(&type_id).unwrap())
	}
}

fn cast_manager<T: 'static + Component>
	(manager: &Box<dyn ComponentManagerTrait>) -> &ComponentManager<T> {
	manager
		.as_any()
		.downcast_ref::<ComponentManager<T>>()
		.unwrap()
}

fn cast_manager_mut<T: 'static + Component>
	(manager: &mut Box<dyn ComponentManagerTrait>) -> &mut ComponentManager<T> {
	manager
		.as_any_mut()
		.downcast_mut::<ComponentManager<T>>()
		.unwrap()
}

fn cast_manager_mut_unsafe<T: 'static + Component>
	(manager: &Box<dyn ComponentManagerTrait>) -> &mut ComponentManager<T> {
	let ptr = cast_manager(manager)
		as *const ComponentManager<T> as *mut ComponentManager<T>;
	unsafe { transmute(ptr) }
}
