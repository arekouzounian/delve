use std::sync::atomic::AtomicU32;
use std::sync::{Arc, atomic::AtomicBool};
use std::thread;

use delve::constants::*;
use delve::engine::DelveEngine;
use delve::input::listen_for_input;
use delve::math::Vec3;
use delve::scene::{Camera, Light, Scene};
use delve::shapes::{Cube, Shape, Sphere};

fn main() -> std::io::Result<()> {
    let running = Arc::new(AtomicBool::new(false));
    let running_clone = running.clone();

    let movement_flags = Arc::new(AtomicU32::new(0));
    let mov_flags_clone = movement_flags.clone();

    let input_thread = thread::spawn(move || listen_for_input(running_clone, mov_flags_clone));

    let camera = Camera::new(
        Vec3::new(4.384, 0.0, -1.402),
        Vec3::ORIGIN,
        std::f32::consts::PI / 2.0,
    );
    let mut scene = Scene::new(camera, AMBIENT_LIGHTING);

    scene.register_light(Light::new(Vec3::new(1.0, 2.0, -1.0), 0.5));
    scene.register_light(Light::new(Vec3::new(1.0, 0.0, 0.0), 0.6));

    let sphere = Sphere::new(Vec3::new(2.0, 0.0, 2.0), 1.0);
    scene.register_shape(String::from("friendly_sphere"), Shape::Sphere(sphere));

    let cube = Cube::new(1.0, Vec3::new(5.0, 0.0, 5.0));
    scene.register_shape(String::from("evil_cube"), Shape::Cube(cube));

    let mut engine = DelveEngine::new(scene, movement_flags, running)?;
    engine.run()?;

    let _ = input_thread.join().expect("something went wrong");

    Ok(())
}
