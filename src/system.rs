use super::entity_manager::{EntityIdAccessor, EntityManager};

pub trait System {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor);
}
