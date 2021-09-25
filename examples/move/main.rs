use ecs_rust::world::World;
use ecs_rust::entity_manager::EntityManager;
use ecs_rust::component::Component;
use ecs_rust::system::System;

struct Namable {
	name: &'static str
}

struct Position {
	x: f32,
	y: f32
}

struct Velocity {
	x: f32,
	y: f32
}

struct Step {
	num: u32
}

struct PrintStepSystem;
struct MoveSystem;
struct PrintPositionSystem;

impl Component for Namable {
}

impl Component for Position {
}

impl Component for Velocity {
}

impl Component for Step {
}

impl System for PrintStepSystem {
	fn update(&mut self, manager: &mut EntityManager) {
		let steps = manager.borrow_components_mut::<Step>().unwrap();
		for step in steps.iter_mut() {
			step.num += 1;
			println!("Step {}", step.num);
		}
	}
}

impl System for MoveSystem {
	fn update(&mut self, manager: &mut EntityManager) {
		let entity_ids = manager.get_entity_ids_for_pair::<Velocity, Position>();
		for id in entity_ids.iter() {
			let (velocity, mut position) = manager.borrow_component_pair_mut::<Velocity, Position>(*id).unwrap();
			position.x += velocity.x;
			position.y += velocity.y;
		}
	}
}

impl System for PrintPositionSystem {
	fn update(&mut self, manager: &mut EntityManager) {
		let entity_ids = manager.get_entity_ids_for_pair::<Namable, Position>();
		for id in entity_ids.iter() {
			let name = manager.borrow_component::<Namable>(*id).unwrap();
			let position = manager.borrow_component::<Position>(*id).unwrap();
			println!("{} is at ({}, {})", name.name, position.x, position.y);
		}
	}
}

fn main() {
	let mut world = World::new();

	world
		.register_component::<Step>()
		.register_component::<Namable>()
		.register_component::<Position>()
		.register_component::<Velocity>();

	{
		let entity_id = world.create_entity();
		world
			.add_component_to_entity(entity_id, Step {num: 0});
	}

	{
		let entity_id = world.create_entity();
		world
			.add_component_to_entity(entity_id, Namable {name: "Alice"})
			.add_component_to_entity(entity_id, Position {x: 0.0, y: 0.0})
			.add_component_to_entity(entity_id, Velocity {x: 1.0, y: 2.0});
	}

	{
		let entity_id = world.create_entity();
		world
			.add_component_to_entity(entity_id, Namable {name: "Bob"})
			.add_component_to_entity(entity_id, Position {x: 0.0, y: 0.0})
			.add_component_to_entity(entity_id, Velocity {x: -2.0, y: 1.0});
	}

	{
		// Unmovable
		let entity_id = world.create_entity();
		world
			.add_component_to_entity(entity_id, Namable {name: "Rock"})
			.add_component_to_entity(entity_id, Position {x: 0.0, y: 0.0});
	}

	world
		.add_system(PrintStepSystem {})
		.add_system(MoveSystem {})
		.add_system(PrintPositionSystem {});

	for _i in 0..3 {
		world.update();
	}
}
