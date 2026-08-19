use std::fs::read_dir;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, atomic::AtomicBool};
use std::thread;

use delve::engine::DelveEngine;
use delve::entities::{Cube, RectPrism, Text};
use delve::entities::{Entities, Plane};
use delve::input::listen_for_input;
use delve::log::DelveLogger;
use delve::scene::{Camera, Light, Scene};
use delve_shared::constants::*;
use delve_shared::{math::Vec3, traits::Entity};

use log::{Level, LevelFilter, Log, debug, set_logger, set_max_level};

static DELVE_LOGGER: DelveLogger =
    DelveLogger::new(Level::Info, true, DelveLogger::DEFAULT_AUTOFLUSH_SIZE);

fn main() -> std::io::Result<()> {
    let running = Arc::new(AtomicBool::new(false));
    let running_clone = running.clone();

    DELVE_LOGGER.init(None);
    let _ = set_logger(&DELVE_LOGGER);
    set_max_level(LevelFilter::Info);

    let movement_flags = Arc::new(AtomicU32::new(0));
    let mov_flags_clone = movement_flags.clone();

    let input_thread = thread::spawn(move || listen_for_input(running_clone, mov_flags_clone));

    let mut camera = Camera::new(Vec3::ORIGIN, std::f32::consts::PI / 2.0, 2.0, 0.4);
    camera.entity_fields_mut().position = Vec3::new(0.0, 1.5, -5.0);
    let mut scene = Scene::new(camera, AMBIENT_LIGHTING);

    scene.register_light(Light::new(Vec3::new(0.0, 2.0, -1.0), 0.5));
    let ground = Plane::new(20.0, 20.0, Vec3::Y);
    scene.register_shape(Entities::Plane(ground));

    // consider ascii art?
    let mut intro_text = Text::new(String::from("Welcome to Delve!"));
    intro_text.entity_fields_mut().position = Vec3::new(0.0, 1.0, 1.0);
    scene.register_shape(Entities::Text(intro_text));

    // draw a number of shapes based on current directory.
    let dir = read_dir(std::env::home_dir().expect("you don't have a home directory...?"))?;
    let mut new_objects = Vec::new();
    for entry in dir {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            // prism width proportional to # of files
            let subfiles = read_dir(entry.path())?.count() as f32;
            let width = 0.1 * subfiles;
            let prism = RectPrism::new(Vec3::new(width, 2.0, 0.1));
            new_objects.push(Entities::RectPrism(prism));
        } else {
            // cube size proportional to file size
            let width = (meta.len() as f32 / 10_024.0).clamp(0.1, 5.0);
            let cube = Cube::new(width);
            new_objects.push(Entities::Cube(cube));
        }
    }

    debug!(target: "main", "found objects: {:?}", new_objects);

    let z_offset = 10.0;

    let mut i = 0.0;
    for obj in new_objects {
        let new_position = Vec3::new(2.0 * i, 2.0, z_offset);
        debug!(target: "main", "spawning {obj:?} at {new_position:?}");
        match obj {
            Entities::Cube(mut c) => {
                c.entity_fields_mut().position = new_position;
                scene.register_shape(Entities::Cube(c));
            }
            Entities::RectPrism(mut r) => {
                r.entity_fields_mut().position = new_position;
                scene.register_shape(Entities::RectPrism(r));
            }
            _ => (),
        }

        i += 1.0;
    }

    let engine = DelveEngine::new(scene, movement_flags, running)?;
    engine.run()?;

    let _ = input_thread.join().expect("something went wrong");
    DELVE_LOGGER.flush();

    Ok(())
}
