use super::component_manager::ComponentsManager;

pub trait System {
	fn update(&mut self, component_manager: &mut ComponentsManager);
}
