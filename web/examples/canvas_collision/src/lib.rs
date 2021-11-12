use std::f64;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use ecs_rust::world::World;
use ecs_rust::entity_manager::{EntityIdAccessor, EntityManager};
use ecs_rust::component::Component;
use ecs_rust::system::System;

const CANVAS_ID: &str = "canvas";

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
	let window = web_sys::window().expect("no global `window` exists");
	window
		.request_animation_frame(f.as_ref().unchecked_ref())
		.expect("should register `requestAnimationFrame` OK");
}

fn rand() -> f64 {
	js_sys::Math::random()
}

fn get_canvas() -> web_sys::HtmlCanvasElement {
	let document = web_sys::window().unwrap().document().unwrap();
	document.get_element_by_id(CANVAS_ID)
		.unwrap()
		.dyn_into::<web_sys::HtmlCanvasElement>()
		.map_err(|_| ())
		.unwrap()
}

fn get_context() -> web_sys::CanvasRenderingContext2d {
	get_canvas()
		.get_context("2d")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::CanvasRenderingContext2d>()
		.unwrap()
}

struct CanvasSize {
	width: f64,
	height: f64
}

struct Position {
	x: f64,
	y: f64
}

struct Velocity {
	x: f64,
	y: f64
}

struct Circle {
	radius: f64
}

struct Collidable {
	collided: bool
}

impl Component for CanvasSize {
}

impl Component for Position {
}

impl Component for Velocity {
}

impl Component for Circle {
}

impl Component for Collidable {
}

struct MoveSystem {
}

struct ReflectBoundarySystem {
}

struct CollisionCheckSystem {
}

struct RenderSystem {
}

impl System for MoveSystem {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor) {
		let ids = accessor.borrow_ids_for_pair::<Position, Velocity>(manager).unwrap();
		for id in ids.iter() {
			let (position, velocity) = manager.borrow_component_pair_mut::<Position, Velocity>(*id).unwrap();
			position.x += velocity.x;
			position.y += velocity.y;
		}
	}
}

impl System for ReflectBoundarySystem {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor) {
		let (canvas_width, canvas_height) = {
			let canvas_size = &manager.borrow_components::<CanvasSize>().unwrap()[0];
			(canvas_size.width, canvas_size.height)
		};
		let ids = accessor.borrow_ids_for_triple::<Position, Velocity, Circle>(manager).unwrap();
		for id in ids.iter() {
			let (position, velocity, circle) = manager.borrow_component_triple_mut::<Position, Velocity, Circle>(*id).unwrap();
			if position.x - circle.radius < 0.0 ||
				position.x + circle.radius >= canvas_width {
				velocity.x = -velocity.x;
			}
			if position.y - circle.radius < 0.0 ||
				position.y + circle.radius >= canvas_height {
				velocity.y = -velocity.y;
			}
		}
	}
}

impl System for CollisionCheckSystem {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor) {
		let ids = accessor.borrow_ids_for_triple::<Position, Circle, Collidable>(manager).unwrap();
		for id in ids.iter() {
			let collidable = manager.borrow_component_mut::<Collidable>(*id).unwrap();
			collidable.collided = false;
		}
		for i in 0..ids.len() {
			for j in (i+1)..ids.len() {
				if self.check_collision(manager, ids[i], ids[j]) {
					let collidable = manager.borrow_component_mut::<Collidable>(ids[i]).unwrap();
					collidable.collided = true;
					let collidable = manager.borrow_component_mut::<Collidable>(ids[j]).unwrap();
					collidable.collided = true;
				}
			}
		}
	}
}

impl CollisionCheckSystem {
	fn check_collision(&self, manager: &EntityManager, entity_id1: usize, entity_id2: usize) -> bool {
		let position1 = manager.borrow_component::<Position>(entity_id1).unwrap();
		let circle1 = manager.borrow_component::<Circle>(entity_id1).unwrap();
		let position2 = manager.borrow_component::<Position>(entity_id2).unwrap();
		let circle2 = manager.borrow_component::<Circle>(entity_id2).unwrap();
		let dx = position1.x - position2.x;
		let dy = position1.y - position2.y;
		(dx * dx + dy * dy).sqrt() < (circle1.radius + circle2.radius)
	}
}

impl System for RenderSystem {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor) {
		let (canvas_width, canvas_height) = {
			let canvas_size = &manager.borrow_components::<CanvasSize>().unwrap()[0];
			(canvas_size.width, canvas_size.height)
		};

		// @TODO: Is getting context every frame costly?
		let context = get_context();
		context.clear_rect(0.0, 0.0, canvas_width, canvas_height);

		let ids = accessor.borrow_ids_for_triple::<Position, Circle, Collidable>(manager).unwrap();
		for id in ids.iter() {
			let position = manager.borrow_component::<Position>(*id).unwrap();
			let circle = manager.borrow_component::<Circle>(*id).unwrap();
			let collidable = manager.borrow_component::<Collidable>(*id).unwrap();

			let color_str = match collidable.collided {
				true => "red",
				false => "black"
			};

			context.begin_path();
			context.set_fill_style(&JsValue::from_str(color_str));
			context
				.arc(position.x, position.y, circle.radius, 0.0, f64::consts::PI * 2.0)
				.unwrap();
			context.fill();
		}
	}
}

#[wasm_bindgen(start)]
pub fn start() {
	let (canvas_width, canvas_height) = {
		let canvas = get_canvas();
		(canvas.width() as f64, canvas.height() as f64)
	};

	let mut world = World::new();

	world
		.register_component::<CanvasSize>()
		.register_component::<Position>()
		.register_component::<Velocity>()
		.register_component::<Circle>()
		.register_component::<Collidable>();

	{
		let entity_id = world.create_entity();
		world
			.add_component_to_entity(entity_id, CanvasSize {
				width: canvas_width,
				height: canvas_height
			});
	}

	for _i in 0..50 {
		let entity_id = world.create_entity();
		world
			.add_component_to_entity(entity_id, Position {
				x: 25.0 + rand() * (canvas_width - 45.0),
				y: 25.0 + rand() * (canvas_height - 45.0)
			})
			.add_component_to_entity(entity_id, Velocity {
				x: rand() * 5.0 - 2.5,
				y: rand() * 5.0 - 2.5
			})
			.add_component_to_entity(entity_id, Circle {
				radius: 5.0 + (rand() * 15.0)
			})
			.add_component_to_entity(entity_id, Collidable {
				collided: false
			});
	}

	world.add_system(MoveSystem {});
	world.add_system(ReflectBoundarySystem {});
	world.add_system(CollisionCheckSystem {});
	world.add_system(RenderSystem {});

	let f = Rc::new(RefCell::new(None));
	let g = f.clone();

	*g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
		request_animation_frame(f.borrow().as_ref().unwrap());
		world.update();
	}) as Box<dyn FnMut()>));

	request_animation_frame(g.borrow().as_ref().unwrap());
}
