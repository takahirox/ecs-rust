use std::f64;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use ecs_rust::world::World;
use ecs_rust::component::Component;
use ecs_rust::component_manager::ComponentsManager;
use ecs_rust::system::System;

const CANVAS_ID: &str = "canvas";

// For debug

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);
}

// JavaScript function helpers

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

struct UserInputBuffer {
	x: f64,
	y: f64
}

static mut USER_INPUT_BUFFER: UserInputBuffer = UserInputBuffer{x: -1.0, y: -1.0};

// @TODO: Any way to avoid unsafe block?

fn fetch_user_input_buffer() -> (f64, f64) {
	unsafe {
		let (x, y) = (USER_INPUT_BUFFER.x, USER_INPUT_BUFFER.y);
		USER_INPUT_BUFFER.x = -1.0;
		USER_INPUT_BUFFER.y = -1.0;
		(x, y)
	}
}

fn update_user_input_buffer(x: f64, y: f64) {
	unsafe {
		USER_INPUT_BUFFER.x = x;
		USER_INPUT_BUFFER.y = y;
	}
}

// Components and Systems define

struct CanvasSize {
	width: f64,
	height: f64
}

struct UserObject {
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
}

struct UserInputReflectSystem {
}

struct MoveSystem {
}

struct ReflectBoundarySystem {
}

struct CollisionSystem {
}

struct RenderSystem {
}

// Components and Systems implementation

impl Component for CanvasSize {
}

impl Component for UserObject {
}

impl Component for Position {
}

impl Component for Velocity {
}

impl Component for Circle {
}

impl Component for Collidable {
}

impl System for UserInputReflectSystem {
	fn update(&mut self, manager: &mut ComponentsManager) {
		let ids = manager.get_entity_ids_for_pair::<Position, UserObject>();
		for id in ids.iter() {
			let (x, y) = fetch_user_input_buffer();
			if x != -1.0 || y != -1.0 {
				let position = manager.borrow_component_mut::<Position>(*id).unwrap();
				position.x = x;
				position.y = y;
			}
		}
	}
}

impl System for MoveSystem {
	fn update(&mut self, manager: &mut ComponentsManager) {
		let ids = manager.get_entity_ids_for_pair::<Position, Velocity>();
		for id in ids.iter() {
			let (position, velocity) = manager.borrow_component_pair_mut::<Position, Velocity>(*id).unwrap();
			position.x += velocity.x;
			position.y += velocity.y;
		}
	}
}

impl System for ReflectBoundarySystem {
	fn update(&mut self, manager: &mut ComponentsManager) {
		let (canvas_width, canvas_height) = {
			let canvas_size = &manager.borrow_components::<CanvasSize>().unwrap()[0];
			(canvas_size.width, canvas_size.height)
		};
		let ids = manager.get_entity_ids_for_triple::<Position, Velocity, Circle>();
		for id in ids.iter() {
			let (position, velocity, circle) = manager.borrow_component_triple_mut::<Position, Velocity, Circle>(*id).unwrap();
			if position.x - circle.radius < 0.0 ||
				position.x + circle.radius >= canvas_width {
				velocity.x = -velocity.x;
				position.x = js_sys::Math::max(position.x, circle.radius);
				position.x = js_sys::Math::min(position.x, canvas_width - circle.radius);
			}
			if position.y - circle.radius < 0.0 ||
				position.y + circle.radius >= canvas_height {
				velocity.y = -velocity.y;
				position.y = js_sys::Math::max(position.y, circle.radius);
				position.y = js_sys::Math::min(position.y, canvas_height - circle.radius);
			}
		}
	}
}

impl System for CollisionSystem {
	fn update(&mut self, manager: &mut ComponentsManager) {
		let user_entity_id = manager.get_entity_ids_for_triple::<Position, Circle, UserObject>()[0];
		let ids = manager.get_entity_ids_for_triple::<Position, Circle, Collidable>();
		for id in ids.iter() {
			if CollisionSystem::check_collision(manager, user_entity_id, *id) {
				CollisionSystem::reflect(manager, user_entity_id, *id);
			}
		}
	}
}

impl CollisionSystem {
	fn check_collision(manager: &ComponentsManager, entity_id1: usize, entity_id2: usize) -> bool {
		let (x1, y1, radius1) = CollisionSystem::get_circle_param(manager, entity_id1);
		let (x2, y2, radius2) = CollisionSystem::get_circle_param(manager, entity_id2);
		let dx = x1 - x2;
		let dy = y1 - y2;
		(dx * dx + dy * dy).sqrt() < (radius1 + radius2)
	}

	fn reflect(manager: &mut ComponentsManager, user_entity_id: usize, entity_id: usize) {
		let (user_x, user_y, user_radius) = CollisionSystem::get_circle_param(manager, user_entity_id);
		let (x, y, radius) = CollisionSystem::get_circle_param(manager, entity_id);

		let dx = x - user_x;
		let dy = y - user_y;
		let theta = js_sys::Math::atan2(dy, dx);
		let new_distance = user_radius + radius;
		let new_x = user_x + new_distance * js_sys::Math::cos(theta);
		let new_y = user_y + new_distance * js_sys::Math::sin(theta);

		let velocity = manager.borrow_component::<Velocity>(entity_id).unwrap();
		let v_scalar = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
		let new_vx = v_scalar * js_sys::Math::cos(theta);
		let new_vy = v_scalar * js_sys::Math::sin(theta);

		let position = manager.borrow_component_mut::<Position>(entity_id).unwrap();
		position.x = new_x;
		position.y = new_y;

		let velocity = manager.borrow_component_mut::<Velocity>(entity_id).unwrap();
		velocity.x = new_vx;
		velocity.y = new_vy;
	}

	fn get_circle_param(manager: &ComponentsManager, entity_id: usize) -> (f64, f64, f64) {
		let position = manager.borrow_component::<Position>(entity_id).unwrap();
		let circle = manager.borrow_component::<Circle>(entity_id).unwrap();
		(position.x, position.y, circle.radius)
	}
}

impl System for RenderSystem {
	fn update(&mut self, manager: &mut ComponentsManager) {
		let (canvas_width, canvas_height) = RenderSystem::get_canvas_size(manager);

		// @TODO: Is getting context every frame costly?
		let context = get_context();
		RenderSystem::clear(&context, canvas_width, canvas_height);
		RenderSystem::render_other_circles(&context, manager);
		RenderSystem::render_user_circle(&context, manager);
	}
}

impl RenderSystem {
	fn get_canvas_size(manager: &ComponentsManager) -> (f64, f64) {
		let canvas_size = &manager.borrow_components::<CanvasSize>().unwrap()[0];
		(canvas_size.width, canvas_size.height)
	}

	fn clear(context: &web_sys::CanvasRenderingContext2d, width: f64, height: f64) {
		context.clear_rect(0.0, 0.0, width, height);
	}

	fn render_user_circle(context: &web_sys::CanvasRenderingContext2d, manager: &ComponentsManager) {
		let ids = manager.get_entity_ids_for_triple::<Position, Circle, UserObject>();
		for id in ids.iter() {
			let position = manager.borrow_component::<Position>(*id).unwrap();
			let circle = manager.borrow_component::<Circle>(*id).unwrap();
			RenderSystem::render_circle(&context, position.x, position.y, circle.radius, "red");
		}
	}

	fn render_other_circles(context: &web_sys::CanvasRenderingContext2d, manager: &ComponentsManager) {
		let ids = manager.get_entity_ids_for_triple::<Position, Circle, Collidable>();
		for id in ids.iter() {
			let position = manager.borrow_component::<Position>(*id).unwrap();
			let circle = manager.borrow_component::<Circle>(*id).unwrap();
			RenderSystem::render_circle(&context, position.x, position.y, circle.radius, "black");
		}
	}

	fn render_circle(context: &web_sys::CanvasRenderingContext2d, x: f64, y: f64, radius: f64, color_str: &str) {
		context.begin_path();
		context.set_fill_style(&JsValue::from_str(color_str));
		context
			.arc(x, y, radius, 0.0, f64::consts::PI * 2.0)
			.unwrap();
		context.fill();
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
		.register_component::<UserObject>()
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

	{
		let entity_id = world.create_entity();
		world
			.add_component_to_entity(entity_id, UserObject {})
			.add_component_to_entity(entity_id, Position {
				x: canvas_width * 0.5,
				y: canvas_height * 0.5
			})
			.add_component_to_entity(entity_id, Circle {
				radius: 25.0
			});
	}

	for _i in 0..100 {
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
			.add_component_to_entity(entity_id, Collidable {});
	}

	world
		.add_system(UserInputReflectSystem {})
		.add_system(MoveSystem {})
		.add_system(ReflectBoundarySystem {})
		.add_system(CollisionSystem {})
		.add_system(RenderSystem {});

	{
		let canvas = get_canvas();
		let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
			update_user_input_buffer(event.offset_x() as f64, event.offset_y() as f64);
		}) as Box<dyn FnMut(_)>);
		canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref()).unwrap();
		canvas.add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref()).unwrap();
		canvas.add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref()).unwrap();
		canvas.add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref()).unwrap();
		closure.forget();
	}

	{
		let f = Rc::new(RefCell::new(None));
		let g = f.clone();

		*g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
			request_animation_frame(f.borrow().as_ref().unwrap());
			world.update();
		}) as Box<dyn FnMut()>));

		request_animation_frame(g.borrow().as_ref().unwrap());
	}
}
