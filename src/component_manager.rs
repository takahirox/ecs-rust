use std::collections::HashMap;
use std::any::Any;

use super::component::Component;

pub trait ComponentManagerTrait {
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

pub struct ComponentManager<T: Component> {
	components: Vec<T>,
	entity_ids: Vec<usize>,
	entity_id_map: HashMap<usize, usize>
}

impl<T: Component> ComponentManager<T> {
	pub fn new() -> Self {
		ComponentManager {
			components: Vec::new(),
			entity_ids: Vec::new(), // Same order with components
			entity_id_map: HashMap::new() // entity_id -> index of components
		}
	}

	pub fn has(&self, entity_id: usize) -> bool {
		self.entity_id_map.contains_key(&entity_id)
	}

	pub fn borrow_component(&self, entity_id: usize) -> Option<&T> {
		let index = self.entity_id_map.get(&entity_id).unwrap();
		match *index < self.components.len() {
			true => Some(&self.components[*index]),
			false => None
		}
	}

	pub fn borrow_component_mut(&mut self, entity_id: usize) -> Option<&mut T> {
		let index = self.entity_id_map.get(&entity_id).unwrap();
		match *index < self.components.len() {
			true => Some(&mut self.components[*index]),
			false => None
		}
	}

	pub fn borrow_components(&self) -> &Vec<T> {
		&self.components
	}

	pub fn borrow_components_mut(&mut self) -> &mut Vec<T> {
		&mut self.components
	}

	pub fn borrow_entity_ids(&self) -> &Vec<usize> {
		&self.entity_ids
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
}
