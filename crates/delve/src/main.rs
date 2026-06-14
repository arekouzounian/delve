use std::sync::atomic::AtomicU32;
use std::sync::{Arc, atomic::AtomicBool};
use std::thread;

use delve::engine::DelveEngine;
use delve::entities::Plane;
#[allow(unused)]
use delve::entities::{Cube, RectPrism, Shape, Sphere};
use delve::input::listen_for_input;
use delve::scene::{Camera, Light, Scene};
use delve_shared::constants::*;
use delve_shared::{math::Vec3, traits::Entity};

fn main() -> std::io::Result<()> {
    let running = Arc::new(AtomicBool::new(false));
    let running_clone = running.clone();

    let movement_flags = Arc::new(AtomicU32::new(0));
    let mov_flags_clone = movement_flags.clone();

    let input_thread = thread::spawn(move || listen_for_input(running_clone, mov_flags_clone));

    let mut camera = Camera::new(Vec3::ORIGIN, std::f32::consts::PI / 2.0);
    *camera.position_mut() = Vec3::new(0.0, 1.0, -5.0);
    let mut scene = Scene::new(camera, AMBIENT_LIGHTING);

    scene.register_light(Light::new(Vec3::new(0.0, 2.0, -1.0), 0.5));

    let mut sphere = Sphere::new(1.0);
    *sphere.position_mut() = Vec3::new(0.0, 1.0, 0.0);

    scene.register_shape(String::from("friendly_sphere"), Box::new(sphere));

    // ground
    let ground = Plane::new(Vec3::new(10.0, 10.0, 10.0), Vec3::Y);
    scene.register_shape(String::from("ground"), Box::new(ground));

    // let cube = Cube::new(1.0, Vec3::new(5.0, 0.0, 5.0));
    // scene.register_shape(String::from("evil_cube"), Shape::Cube(cube));

    // rotate about y axis
    // let rect = RectPrism::new(Vec3::new(0.5, 0.5, 4.0), Vec3::new(0.0, 0.0, 1.0));
    // scene.register_shape(String::from("long_prism"), Shape::RectPrism(rect));

    let engine = DelveEngine::new(scene, movement_flags, running)?;
    engine.run()?;

    let _ = input_thread.join().expect("something went wrong");

    Ok(())
}
