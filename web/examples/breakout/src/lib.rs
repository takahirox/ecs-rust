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

struct Vaus {
}

struct Ball {
	radius: f64
}

struct Brick {
}

struct Position {
	x: f64,
	y: f64
}

struct Velocity {
	x: f64,
	y: f64
}

struct Rectangle {
	width: f64,
	height: f64
}

struct UserInputReflectSystem {
}

struct MoveSystem {
}

struct ReflectBoundarySystem {
}

struct BallVausCollisionSystem {
}

struct BallBricksCollisionSystem {
}

struct RenderSystem {
}

// Components and Systems implementation

impl Component for CanvasSize {
}

impl Component for Vaus {
}

impl Component for Ball {
}

impl Component for Brick {
}

impl Component for Position {
}

impl Component for Velocity {
}

impl Component for Rectangle {
}

impl System for UserInputReflectSystem {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor) {
		let ids = accessor.borrow_ids_for_pair::<Position, Vaus>(manager).unwrap();
		for id in ids.iter() {
			let (x, _y) = fetch_user_input_buffer();
			if x != -1.0 {
				let position = manager.borrow_component_mut::<Position>(*id).unwrap();
				position.x = x;
			}
		}
	}
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
		let ids = accessor.borrow_ids_for_triple::<Position, Velocity, Ball>(manager).unwrap();
		for id in ids.iter() {
			let (position, velocity, ball) = manager.borrow_component_triple_mut::<Position, Velocity, Ball>(*id).unwrap();
			if position.x - ball.radius < 0.0 ||
				position.x + ball.radius >= canvas_width {
				velocity.x = -velocity.x;
			}
			if position.y - ball.radius < 0.0 ||
				position.y + ball.radius >= canvas_height {
				velocity.y = -velocity.y;
			}
		}
	}
}

fn check_ball_rect_collision(manager: &EntityManager, ball_entity_id: usize, rect_entity_id: usize) -> bool {
	let (ball_x, ball_y, ball_radius) = get_ball_param(manager, ball_entity_id);
	let (rect_x, rect_y, rect_width, rect_height) = get_rect_param(manager, rect_entity_id);
	// @TODO: Can be oprimized
	let rect_left = rect_x - rect_width * 0.5;
	let rect_right = rect_x + rect_width * 0.5;
	let rect_top = rect_y - rect_height * 0.5;
	let rect_bottom = rect_y  + rect_height * 0.5;
	if ball_x >= rect_left &&
		ball_x <= rect_right &&
		ball_y >= rect_top - ball_radius &&
		ball_y <= rect_bottom + ball_radius {
		return true;
	}
	if ball_x >= rect_left - ball_radius &&
		ball_x <= rect_right + ball_radius &&
		ball_y >= rect_top &&
		ball_y <= rect_bottom {
		return true;
	}
	false
}

fn get_ball_param(manager: &EntityManager, entity_id: usize) -> (f64, f64, f64) {
	let position = manager.borrow_component::<Position>(entity_id).unwrap();
	let ball = manager.borrow_component::<Ball>(entity_id).unwrap();
	(position.x, position.y, ball.radius)
}

fn get_rect_param(manager: &EntityManager, entity_id: usize) -> (f64, f64, f64, f64) {
	let position = manager.borrow_component::<Position>(entity_id).unwrap();
	let rect = manager.borrow_component::<Rectangle>(entity_id).unwrap();
	(position.x, position.y, rect.width, rect.height)
}

impl System for BallVausCollisionSystem {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor) {
		let ball_entity_id = accessor.borrow_ids::<Ball>(manager).unwrap()[0];
		let vaus_entity_id = accessor.borrow_ids::<Vaus>(manager).unwrap()[0];
		if check_ball_rect_collision(manager, ball_entity_id, vaus_entity_id) {
			BallVausCollisionSystem::reflect(manager, ball_entity_id, vaus_entity_id);
		}
	}
}

impl BallVausCollisionSystem {
	fn reflect(manager: &mut EntityManager, ball_entity_id: usize, vaus_entity_id: usize) {
		let (ball_x, ball_y, _ball_radius) = get_ball_param(manager, ball_entity_id);
		let (vaus_x, vaus_y, _vaus_width, _vaus_height) = get_rect_param(manager, vaus_entity_id);

		let dx = ball_x - vaus_x;
		let dy = ball_y - vaus_y;
		let theta = dy.atan2(dx);

		let theta = theta * 0.8 + f64::consts::PI * match theta >= 0.0 {
			true => 0.1, false => -0.1
		};

		let velocity = manager.borrow_component::<Velocity>(ball_entity_id).unwrap();
		let v_scalar = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
		let new_vx = v_scalar * theta.cos();
		let new_vy = v_scalar * theta.sin();

		let velocity = manager.borrow_component_mut::<Velocity>(ball_entity_id).unwrap();
		velocity.x = new_vx;
		velocity.y = new_vy;
	}
}

impl System for BallBricksCollisionSystem {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor) {
		let ball_entity_id = accessor.borrow_ids_for_triple::<Ball, Position, Velocity>(manager).unwrap()[0];
		let ids = accessor.borrow_ids_for_pair::<Brick, Position>(manager).unwrap();
		for id in ids.iter() {
			if check_ball_rect_collision(manager, ball_entity_id, *id) {
				BallBricksCollisionSystem::reflect(manager, ball_entity_id, *id);
				BallBricksCollisionSystem::remove_brick(manager, *id);
			}
		}
	}
}

impl BallBricksCollisionSystem {
	fn reflect(manager: &mut EntityManager, ball_entity_id: usize, brick_entity_id: usize) {
		let ball_x = manager.borrow_component::<Position>(ball_entity_id).unwrap().x;
		let (brick_x, _brick_y, brick_width, _brick_height) = get_rect_param(manager, brick_entity_id);
		let brick_left = brick_x - brick_width * 0.5;
		let brick_right = brick_x + brick_width * 0.5;

		let velocity = manager.borrow_component_mut::<Velocity>(ball_entity_id).unwrap();

		let (new_v_x, new_v_y) = if ball_x < brick_left || ball_x > brick_right {
			(-velocity.x, velocity.y)
		} else {
			(velocity.x, -velocity.y)
		};

		velocity.x = new_v_x;
		velocity.y = new_v_y;
	}

	fn remove_brick(manager: &mut EntityManager, entity_id: usize) {
		manager.remove_entity(entity_id);
	}
}

impl System for RenderSystem {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor) {
		let (canvas_width, canvas_height) = RenderSystem::get_canvas_size(manager);

		// @TODO: Is getting context every frame costly?
		let context = get_context();
		RenderSystem::clear(&context, canvas_width, canvas_height);
		RenderSystem::render_ball(&context, manager, accessor);
		RenderSystem::render_vaus(&context, manager, accessor);
		RenderSystem::render_bricks(&context, manager, accessor);
	}
}

impl RenderSystem {
	fn get_canvas_size(manager: &EntityManager) -> (f64, f64) {
		let canvas_size = &manager.borrow_components::<CanvasSize>().unwrap()[0];
		(canvas_size.width, canvas_size.height)
	}

	fn clear(context: &web_sys::CanvasRenderingContext2d, width: f64, height: f64) {
		context.clear_rect(0.0, 0.0, width, height);
	}

	fn render_ball(context: &web_sys::CanvasRenderingContext2d, manager: &EntityManager, accessor: &mut EntityIdAccessor) {
		let ids = accessor.borrow_ids_for_pair::<Position, Ball>(manager).unwrap();
		for id in ids.iter() {
			let (x, y, radius) = get_ball_param(manager, *id);
			RenderSystem::render_circle(&context, x, y, radius, "red");
		}
	}

	fn render_vaus(context: &web_sys::CanvasRenderingContext2d, manager: &EntityManager, accessor: &mut EntityIdAccessor) {
		let ids = accessor.borrow_ids_for_pair::<Position, Vaus>(manager).unwrap();
		for id in ids.iter() {
			let (x, y, width, height) = get_rect_param(manager, *id);
			RenderSystem::render_rect(&context, x, y, width, height, "black");
		}
	}

	fn render_bricks(context: &web_sys::CanvasRenderingContext2d, manager: &EntityManager, accessor: &mut EntityIdAccessor) {
		let ids = accessor.borrow_ids_for_pair::<Position, Brick>(manager).unwrap();
		for id in ids.iter() {
			let (x, y, width, height) = get_rect_param(manager, *id);
			RenderSystem::render_rect(&context, x, y, width, height, "gray");
		}
	}

	fn render_rect(context: &web_sys::CanvasRenderingContext2d, x: f64, y: f64, width: f64, height: f64, color_str: &str) {
		let x = x - width * 0.5;
		let y = y - height * 0.5;
		context.set_fill_style(&JsValue::from_str(color_str));
		context.fill_rect(x, y, width, height);
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

	let scale_h = canvas_width / 480.0;
	let scale_v = canvas_height / 480.0;

	let mut world = World::new();

	world
		.register_component::<CanvasSize>()
		.register_component::<Vaus>()
		.register_component::<Ball>()
		.register_component::<Brick>()
		.register_component::<Position>()
		.register_component::<Velocity>()
		.register_component::<Rectangle>();

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
			.add_component_to_entity(entity_id, Vaus {})
			.add_component_to_entity(entity_id, Position {
				x: canvas_width * 0.5,
				y: canvas_height - 60.0 * scale_v
			})
			.add_component_to_entity(entity_id, Rectangle {
				width: 100.0 * scale_h,
				height: 10.0 * scale_v
			});
	}

	{
		let entity_id = world.create_entity();
		world
			.add_component_to_entity(entity_id, Ball {
				radius: 10.0
			})
			.add_component_to_entity(entity_id, Position {
				x: canvas_width * 0.5,
				y: canvas_height * 0.5
			})
			.add_component_to_entity(entity_id, Velocity {
				x: 0.0,
				y: 6.0 * scale_v
			});
	}

	for i in 0..5 {
		for j in 0..5 {
			let entity_id = world.create_entity();
			world
				.add_component_to_entity(entity_id, Brick {})
				.add_component_to_entity(entity_id, Position {
					x: (j as f64 * 85.0 + 70.0) * scale_h,
					y: (i as f64 * 25.0 + 40.0) * scale_v
				})
				.add_component_to_entity(entity_id, Rectangle {
					width: 80.0 * scale_h,
					height: 20.0 * scale_v
				});
		}
	}

	world
		.add_system(UserInputReflectSystem {})
		.add_system(MoveSystem {})
		.add_system(ReflectBoundarySystem {})
		.add_system(BallVausCollisionSystem {})
		.add_system(BallBricksCollisionSystem {})
		.add_system(RenderSystem {});

	{
		let canvas = get_canvas();
		let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
			update_user_input_buffer(event.offset_x() as f64, event.offset_y() as f64);
		}) as Box<dyn FnMut(_)>);
		canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref()).unwrap();
		closure.forget();
	}

	{
		let canvas = get_canvas();
		let closure = Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
			let page_x = event.touches().get(0).unwrap().page_x();
			let page_y = event.touches().get(0).unwrap().page_y();
			let offset_left = event.touches().get(0).unwrap().target().unwrap().dyn_ref::<web_sys::HtmlElement>().unwrap().offset_left();
			let offset_top = event.touches().get(0).unwrap().target().unwrap().dyn_ref::<web_sys::HtmlElement>().unwrap().offset_top();
			update_user_input_buffer(
				(page_x - offset_left) as f64,
				(page_y - offset_top) as f64
			);
		}) as Box<dyn FnMut(_)>);
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
