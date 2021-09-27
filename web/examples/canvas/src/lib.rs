use std::f64;
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

struct Position {
	x: f64,
	y: f64
}

struct Circle {
	radius: f64
}

impl Component for Position {
}

impl Component for Circle {
}

struct RenderSystem {
}

impl System for RenderSystem {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor) {
		let context = get_context();
		let ids = accessor.borrow_ids_for_pair::<Position, Circle>(manager).unwrap();
		for id in ids.iter() {
			let position = manager.borrow_component::<Position>(*id).unwrap();
			let circle = manager.borrow_component::<Circle>(*id).unwrap();

			context.begin_path();
			context
				.arc(position.x, position.y, circle.radius, 0.0, f64::consts::PI * 2.0)
				.unwrap();
			context.stroke();
		}
	}
}

#[wasm_bindgen(start)]
pub fn start() {
	let (canvas_width, canvas_height) = {
		let canvas = get_canvas();
		(canvas.width(), canvas.height())
	};

	let mut world = World::new();

	world
		.register_component::<Position>()
		.register_component::<Circle>();

	for _i in 0..100 {
		let entity_id = world.create_entity();
		world
			.add_component_to_entity(entity_id, Position {
				x: rand() * (canvas_width as f64),
				y: rand() * (canvas_height as f64)
			})
			.add_component_to_entity(entity_id, Circle {
				radius: 5.0 + (rand() * 25.0)
			});
	}

	world.add_system(RenderSystem {});

	world.update();
}
