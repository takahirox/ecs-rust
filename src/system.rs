use super::entity_manager::EntityManager;

pub trait System {
	fn update(&mut self, manager: &mut EntityManager);
}
