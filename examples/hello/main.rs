use ecs_rust::world::World;
use ecs_rust::entity_manager::{EntityIdAccessor, EntityManager};
use ecs_rust::component::Component;
use ecs_rust::system::System;

struct Person {
	name: &'static str
}

struct HelloSystem;

impl Component for Person {
}

impl System for HelloSystem {
	fn update(&mut self, manager: &mut EntityManager, _accessor: &mut EntityIdAccessor) {
		let people = manager.borrow_components::<Person>().unwrap();
		for p in people.iter() {
			println!("Hello {}", p.name);
		}
	}
}

fn main() {
	let mut world = World::new();

	world
		.register_component::<Person>();

	{
		let entity_id = world.create_entity();
		world.add_component_to_entity(entity_id, Person {name: "Alice"});
	}

	{
		let entity_id = world.create_entity();
		world.add_component_to_entity(entity_id, Person {name: "Bob"});
	}

	{
		let entity_id = world.create_entity();
		world.add_component_to_entity(entity_id, Person {name: "Carol"});
	}

	world.add_system(HelloSystem {});

	world.update();
}
