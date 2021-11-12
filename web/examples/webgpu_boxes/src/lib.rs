#[path = "../../utils/window.rs"]
mod window;

use std::f64;
use wasm_bindgen::prelude::*;
use ecs_rust::{
	world::World,
	entity_manager::{
		EntityIdAccessor,
		EntityManager,
	},
	component::Component,
	system::System,
};
use wgpu_rust_renderer::{
	geometry::geometry::Geometry,
	math::{
		color::Color,
		vector3::Vector3,
	},
	renderer::wgpu_renderer::WGPURendererOptions,
	resource::resource::{
		ResourceId,
		ResourcePools,
	},
	scene::{
		camera::PerspectiveCamera,
		mesh::Mesh,
		node::Node,
		scene::Scene,
	},
	utils::{
		geometry_helper::GeometryHelper,
		material_helper::MaterialHelper,
	},
	web::wgpu_web_renderer::WGPUWebRenderer,
};
use window::{
	create_window,
	get_window_device_pixel_ratio,
	get_window_inner_size,
};
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::web::WindowExtWebSys,
};

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);
}

fn rand() -> f64 {
	js_sys::Math::random()
}

struct RotationSpeed {
	speed: [f32; 3],
}

struct SceneComponent {
	scene: ResourceId<Scene>,
}

struct CameraComponent {
	camera: ResourceId<PerspectiveCamera>,
	_node: ResourceId<Node>,
}

struct BoxComponent {
	_mesh: ResourceId<Mesh>,
	node: ResourceId<Node>,
}

struct Pools {
	pools: ResourcePools,
}

struct Renderer {
	renderer: WGPUWebRenderer,
}

impl Component for RotationSpeed {}
impl Component for SceneComponent {}
impl Component for CameraComponent {}
impl Component for BoxComponent {}
impl Component for Pools {}
impl Component for Renderer {}

struct AnimationSystem {}
struct SceneUpdateSystem {}
struct RenderSystem {}

impl System for AnimationSystem {
	fn update(&mut self, manager: &mut EntityManager, accessor: &mut EntityIdAccessor) {
		let ids = accessor.borrow_ids_for_pair::<BoxComponent, RotationSpeed>(manager).unwrap();
		for id in ids {
			let rotation_speed = manager.borrow_component::<RotationSpeed>(*id).unwrap().speed;
			let box_rid = manager.borrow_component::<BoxComponent>(*id).unwrap().node;
			let pools = &mut manager.borrow_components_mut::<Pools>().unwrap()[0].pools;
			let node = pools.borrow_mut::<Node>().borrow_mut(&box_rid).unwrap();
			Vector3::add(
				node.borrow_rotation_mut(),
				&rotation_speed,
			);
		}
	}
}

impl System for SceneUpdateSystem {
	fn update(&mut self, manager: &mut EntityManager, _accessor: &mut EntityIdAccessor) {
		let (scene, pools) = {
			let (scenes, poolses) = manager.borrow_components_pair_mut::<
				SceneComponent,
				Pools,
			>().unwrap();
			(&scenes[0].scene, &mut poolses[0].pools)
		};

		pools.borrow::<Scene>()
			.borrow(scene)
			.unwrap()
			.update_matrices(pools);
	}
}

impl System for RenderSystem {
	fn update(&mut self, manager: &mut EntityManager, _accessor: &mut EntityIdAccessor) {
		let (renderer, scene, camera, pools) = {
			let (renderers, scenes, cameras, poolses) = manager.borrow_components_quad_mut::<
				Renderer,
				SceneComponent,
				CameraComponent,
				Pools,
			>().unwrap();
			(&mut renderers[0].renderer, &scenes[0].scene, &cameras[0].camera, &poolses[0].pools)
		};
		renderer.render(pools, scene, camera);
	}
}

async fn create_renderer(window: &winit::window::Window) -> WGPUWebRenderer {
	let inner_size = get_window_inner_size();
	let pixel_ratio = get_window_device_pixel_ratio();

	let mut renderer = WGPUWebRenderer::new(&window, window.canvas(), WGPURendererOptions::default()).await;
	renderer.set_size(inner_size.0 as f64, inner_size.1 as f64);
	renderer.set_pixel_ratio(pixel_ratio as f64);
	renderer
}

fn create_camera(pools: &mut ResourcePools, scene: &mut Scene) -> (ResourceId<PerspectiveCamera>, ResourceId<Node>) {
	let inner_size = get_window_inner_size();
	let camera_rid = pools.borrow_mut::<PerspectiveCamera>().add(
		PerspectiveCamera::new(
			60.0_f32.to_radians(),
			(inner_size.0 / inner_size.1) as f32,
			0.1,
			1000.0,
		),
	);

	let mut node = Node::new();
	Vector3::set(
		node.borrow_position_mut(),
		0.0, 0.0, 10.0,
	);

	let node_rid = pools.borrow_mut::<Node>().add(node);
	scene.add_node(&node_rid);
	scene.assign(&node_rid, &camera_rid);

	(camera_rid, node_rid)
}

fn create_geometry(pools: &mut ResourcePools) -> ResourceId<Geometry> {
	GeometryHelper::create_box(
		pools,
		1.0,
		1.0,
		1.0,
	)
}

fn create_box(
	pools: &mut ResourcePools,
	scene: &mut Scene,
	geometry_rid: &ResourceId<Geometry>,
) -> (ResourceId<Mesh>, ResourceId<Node>) {
	let material_rid = MaterialHelper::create_basic_material(
		pools,
		Color::set(&mut Color::create(), 0.5, 0.5, 1.0),
	);

	let mesh_rid = pools.borrow_mut::<Mesh>().add(Mesh::new(*geometry_rid, material_rid));
	let node_rid = pools.borrow_mut::<Node>().add(Node::new());
	scene.add_node(&node_rid);
	scene.assign(&node_rid, &mesh_rid);

    Vector3::set(
		pools.borrow_mut::<Node>().borrow_mut(&node_rid).unwrap().borrow_position_mut(),
		(rand() as f32 - 0.5) * 10.0,
		(rand() as f32 - 0.5) * 10.0,
		(rand() as f32) * -10.0,
    );

    Vector3::set(
		pools.borrow_mut::<Node>().borrow_mut(&node_rid).unwrap().borrow_rotation_mut(),
		(rand() as f32 * 360.0).to_radians(),
		(rand() as f32 * 360.0).to_radians(),
		(rand() as f32 * 360.0).to_radians(),
    );

	let scale = (rand() as f32 * 0.5) + 0.5;
    Vector3::set(
		pools.borrow_mut::<Node>().borrow_mut(&node_rid).unwrap().borrow_scale_mut(),
		scale,
		scale,
		scale,
    );

	(mesh_rid, node_rid)
}

#[wasm_bindgen(start)]
pub async fn start() {
	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
	console_log::init().expect("could not initialize logger");

	let event_loop = EventLoop::new();
	let window = create_window(&event_loop);

	web_sys::window()
		.and_then(|win| win.document())
		.and_then(|doc| doc.body())
		.and_then(|body| {
			body.append_child(&web_sys::Element::from(window.canvas()))
				.ok()
		})
		.expect("couldn't append canvas to document body");

	let mut world = World::new();

	world
		.register_component::<RotationSpeed>()
		.register_component::<SceneComponent>()
		.register_component::<CameraComponent>()
		.register_component::<BoxComponent>()
		.register_component::<Pools>()
		.register_component::<Renderer>();

	let mut pools = ResourcePools::new();
	let mut scene = Scene::new();

	{
		let geometry_rid = create_geometry(&mut pools);
		for _i in 0..100 {
			let entity_id = world.create_entity();
			let (mesh_rid, node_rid) = create_box(&mut pools, &mut scene, &geometry_rid);
			world
				.add_component_to_entity(entity_id, BoxComponent {
					_mesh: mesh_rid,
					node: node_rid,
				})
				.add_component_to_entity(entity_id, RotationSpeed {
					speed: [
						(rand() as f32 * 2.0 - 1.0).to_radians(),
						(rand() as f32 * 2.0 - 1.0).to_radians(),
						(rand() as f32 * 2.0 - 1.0).to_radians(),
					],
				});
		}
	}

	{
		let entity_id = world.create_entity();
		let renderer = create_renderer(&window).await;
		world
			.add_component_to_entity(entity_id, Renderer {
				renderer: renderer,
			});
	}

	{
		let entity_id = world.create_entity();
		let (camera_rid, node_rid) = create_camera(&mut pools, &mut scene);
		world
			.add_component_to_entity(entity_id, CameraComponent {
				camera: camera_rid,
				_node: node_rid,
			});
	}

	{
		let entity_id = world.create_entity();
		let scene_rid = pools.borrow_mut::<Scene>().add(scene);
		world
			.add_component_to_entity(entity_id, SceneComponent {
				scene: scene_rid,
			});
	}

	{
		let entity_id = world.create_entity();
		world
			.add_component_to_entity(entity_id, Pools {
				pools: pools,
			});
	}

	world
		.add_system(AnimationSystem {})
		.add_system(SceneUpdateSystem {})
		.add_system(RenderSystem {});

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent {
				event: WindowEvent::Resized(_size),
				..
			} => {
              // @TODO: Support Resize
			},
			Event::RedrawEventsCleared => {
                window.request_redraw();
            },
			Event::RedrawRequested(_) => {
				world.update();
			},
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => {
				*control_flow = ControlFlow::Exit;
			},
			_ => {}
		}
	});
}
