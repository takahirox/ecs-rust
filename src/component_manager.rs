use std::collections::HashMap;
use std::any::{Any, TypeId};

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
	manager_index_map: HashMap<TypeId, usize>,
	manager_vec: Vec<Box<dyn ComponentManagerTrait>>
}

impl ComponentsManager {
	pub fn new() -> Self {
		ComponentsManager {
			manager_index_map: HashMap::new(),
			manager_vec: Vec::new()
		}
	}

	pub fn register<T: 'static + Component>(&mut self) -> &mut Self {
		let type_id = TypeId::of::<T>();
		// @TODO: Error handling if already registered?
		if ! self.manager_index_map.contains_key(&type_id) {
			self.manager_index_map.insert(type_id, self.manager_vec.len());
			self.manager_vec.push(Box::new(ComponentManager::<T>::new()));
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
	pub fn get_entity_ids<T: 'static + Component, U: 'static + Component>(&self) -> Vec<usize> {
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

	pub fn borrow_components_ref_mut
		<T: 'static + Component, U: 'static + Component>
		(&mut self, entity_id: usize) -> Option<(&T, &mut U)> {
		if !self.has_component_manager::<T>() ||
			!self.has_component_manager::<U>() {
			return None;
		}

		let (index1, index2) = self.get_manager_indices::<T, U>();

		let (manager_ref, manager_mut) = if index1 < index2 {
			let (left, right) = self.manager_vec.split_at_mut(index2);
			(&left[index1], &mut right[0])
		} else {
			let (left, right) = self.manager_vec.split_at_mut(index1);
			(&right[0], &mut left[index2])
		};

		let component_ref = cast_manager::<T>(manager_ref).borrow_component(entity_id).unwrap();
		let component_mut = cast_manager_mut::<U>(manager_mut).borrow_component_mut(entity_id).unwrap();

		Some((component_ref, component_mut))
	}

	pub fn borrow_components_mut_mut
		<T: 'static + Component, U: 'static + Component>
		(&mut self, entity_id: usize) -> Option<(&mut T, &mut U)> {
		if !self.has_component_manager::<T>() ||
			!self.has_component_manager::<U>() {
			return None;
		}

		let (index1, index2) = self.get_manager_indices::<T, U>();

		let (manager_mut1, manager_mut2) = if index1 < index2 {
			let (left, right) = self.manager_vec.split_at_mut(index2);
			(&mut left[index1], &mut right[0])
		} else {
			let (left, right) = self.manager_vec.split_at_mut(index1);
			(&mut right[0], &mut left[index2])
		};

		let component_mut1 = cast_manager_mut::<T>(manager_mut1).borrow_component_mut(entity_id).unwrap();
		let component_mut2 = cast_manager_mut::<U>(manager_mut2).borrow_component_mut(entity_id).unwrap();

		Some((component_mut1, component_mut2))
	}

	fn has_component_manager<T: 'static + Component>(&self) -> bool {
		let type_id = TypeId::of::<T>();
		self.manager_index_map.contains_key(&type_id)
	}

	fn borrow_component_manager<T: 'static + Component>(&self) -> &ComponentManager<T> {
		let type_id = TypeId::of::<T>();
		let index = *self.manager_index_map.get(&type_id).unwrap();
		cast_manager(&self.manager_vec[index])
	}

	fn borrow_component_manager_mut<T: 'static + Component>(&mut self) -> &mut ComponentManager<T> {
		let type_id = TypeId::of::<T>();
		let index = *self.manager_index_map.get(&type_id).unwrap();
		cast_manager_mut(&mut self.manager_vec[index])
	}

	fn get_manager_indices
		<T: 'static + Component, U: 'static + Component>
		(&self) -> (usize, usize) {
		let (type_id1, type_id2) = get_type_ids::<T, U>();
		let index1 = *self.manager_index_map.get(&type_id1).unwrap();
		let index2 = *self.manager_index_map.get(&type_id2).unwrap();
		(index1, index2)
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

fn get_type_ids<T: 'static + Component, U: 'static + Component>() -> (TypeId, TypeId) {
	let type_id1 = TypeId::of::<T>();
	let type_id2 = TypeId::of::<U>();
	(type_id1, type_id2)
}
