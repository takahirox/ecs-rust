use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::mem::transmute;

use super::component::Component;

trait ComponentManagerTrait {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static + Component> ComponentManagerTrait for ComponentManager<T> {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

struct ComponentManager<T: Component> {
	components: Vec<T>,
	entity_ids: Vec<usize>,
	entity_id_map: HashMap<usize, usize>
}

impl<T: Component> ComponentManager<T> {
	fn new() -> Self {
		ComponentManager {
			components: Vec::new(),
			entity_ids: Vec::new(), // Same order with components
			entity_id_map: HashMap::new() // entity_id -> index of components
		}
	}

	fn has(&self, entity_id: usize) -> bool {
		self.entity_id_map.contains_key(&entity_id)
	}

	fn borrow_component(&self, entity_id: usize) -> Option<&T> {
		let index = self.entity_id_map.get(&entity_id).unwrap();
		match *index < self.components.len() {
			true => Some(&self.components[*index]),
			false => None
		}
	}

	fn borrow_component_mut(&mut self, entity_id: usize) -> Option<&mut T> {
		let index = self.entity_id_map.get(&entity_id).unwrap();
		match *index < self.components.len() {
			true => Some(&mut self.components[*index]),
			false => None
		}
	}

	fn borrow_components(&self) -> &Vec<T> {
		&self.components
	}

	fn borrow_components_mut(&mut self) -> &mut Vec<T> {
		&mut self.components
	}

	fn borrow_entity_ids(&self) -> &Vec<usize> {
		&self.entity_ids
	}

	fn add(&mut self, entity_id: usize, component: T) {
		if self.has(entity_id) {
			// Nothing to do? Throw error? Update component?
			return;
		}
		self.components.push(component);
		self.entity_ids.push(entity_id);
		let component_index = self.components.len() - 1;
		self.entity_id_map.insert(entity_id, component_index);
	}
}

pub struct ComponentsManager {
	manager_map: HashMap<TypeId, Box<dyn ComponentManagerTrait>>
}

impl ComponentsManager {
	pub fn new() -> Self {
		ComponentsManager {
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
