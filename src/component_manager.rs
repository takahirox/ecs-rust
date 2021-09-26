use std::collections::HashMap;
use std::any::Any;

use super::component::Component;

// @TODO: Write comment
pub trait ComponentManagerTrait {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
	fn has(&self, entity_id: usize) -> bool;
	fn remove(&mut self, entity_id: usize);
}

impl<T: 'static + Component> ComponentManagerTrait for ComponentManager<T> {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

	fn has(&self, entity_id: usize) -> bool {
		let manager = cast_manager::<T>(self);
		manager.has(entity_id)
	}

	fn remove(&mut self, entity_id: usize) {
		let manager = cast_manager_mut::<T>(self);
		manager.remove(entity_id);
	}
}

// @TODO: Write comment
pub fn cast_manager<T: 'static + Component>
	(manager: &dyn ComponentManagerTrait) -> &ComponentManager<T> {
	manager
		.as_any()
		.downcast_ref::<ComponentManager<T>>()
		.unwrap()
}

// @TODO: Write comment
pub fn cast_manager_mut<T: 'static + Component>
	(manager: &mut dyn ComponentManagerTrait) -> &mut ComponentManager<T> {
	manager
		.as_any_mut()
		.downcast_mut::<ComponentManager<T>>()
		.unwrap()
}

pub struct ComponentManager<T: Component> {
	components: Vec<T>, // Component contents
	entity_ids: Vec<usize>, // Same order with components
	entity_id_map: HashMap<usize, usize>  // entity_id -> index in components
}

impl<T: Component> ComponentManager<T> {
	pub fn new() -> Self {
		ComponentManager {
			components: Vec::new(),
			entity_ids: Vec::new(),
			entity_id_map: HashMap::new(),
		}
	}

	pub fn has(&self, entity_id: usize) -> bool {
		self.entity_id_map.contains_key(&entity_id)
	}

	pub fn add(&mut self, entity_id: usize, component: T) {
		if self.has(entity_id) {
			// Nothing to do? Throw error? Update component?
			return;
		}
		self.components.push(component);
		self.entity_ids.push(entity_id);
		let component_index = self.components.len() - 1;
		self.entity_id_map.insert(entity_id, component_index);
	}

	pub fn remove(&mut self, entity_id: usize) {
		if !self.has(entity_id) {
			// Nothing to do? Throw error? Update component?
			return;
		}
		let index = *self.entity_id_map.get(&entity_id).unwrap();
		self.entity_id_map.insert(*self.entity_ids.last().unwrap(), index);
		self.components.swap_remove(index);
		self.entity_ids.swap_remove(index);
		self.entity_id_map.remove(&entity_id);
	}

	pub fn borrow_component(&self, entity_id: usize) -> Option<&T> {
		if !self.has(entity_id) {
			return None;
		}
		let index = self.entity_id_map.get(&entity_id).unwrap();
		Some(&self.components[*index])
	}

	pub fn borrow_component_mut(&mut self, entity_id: usize) -> Option<&mut T> {
		if !self.has(entity_id) {
			return None;
		}
		let index = self.entity_id_map.get(&entity_id).unwrap();
		Some(&mut self.components[*index])
	}

	pub fn borrow_entity_ids(&self) -> &Vec<usize> {
		&self.entity_ids
	}

	pub fn borrow_components(&self) -> &Vec<T> {
		&self.components
	}

	pub fn borrow_components_mut(&mut self) -> &mut Vec<T> {
		&mut self.components
	}
}
