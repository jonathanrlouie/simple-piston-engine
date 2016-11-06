use yaml_rust::YamlLoader;
use piston_window::{PistonWindow, OpenGL, WindowSettings};
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::Read;

pub struct ConfigLoader;

impl ConfigLoader {
  pub fn load_config(&self, config_path: &str) -> PistonWindow {
    let path = Path::new(config_path);
    let display = path.display();
    let mut file = match File::open(&path) {
      Err(why) => panic!("Error: Couldn't open {}: {}", display,
                         why.description()),
      Ok(file) => file,
    };
    let mut file_str = String::new();
    match file.read_to_string(&mut file_str) {
      Err(why) => panic!("Error: Couldn't read {}: {}", display,
                         why.description()),
      Ok(_) => (),
    };
    let docs = match YamlLoader::load_from_str(&file_str) {
      Err(why) => panic!("Error: Couldn't load YAML docs from {}: {}", display,
                         why.description()),
      Ok(docs) => docs,
    };
    let doc = &docs[0];
    let title = doc["title"][0].as_str().unwrap_or("Game");
    let window_width = doc["width"][0].as_i64().unwrap_or(640) as u32;
    let window_height = doc["height"][0].as_i64().unwrap_or(480) as u32;
    let samples = doc["samples"][0].as_i64().unwrap_or(0) as u8;
    let fullscreen = doc["fullscreen"][0].as_bool().unwrap_or(false);
    let exit_on_esc = doc["exit_on_esc"][0].as_bool().unwrap_or(false);
    let vsync = doc["vsync"][0].as_bool().unwrap_or(false);
    let srgb = doc["srgb"][0].as_bool().unwrap_or(true);
    let resizable = doc["resizable"][0].as_bool().unwrap_or(true);
    let decorated = doc["decorated"][0].as_bool().unwrap_or(true);
    let controllers = doc["controllers"][0].as_bool().unwrap_or(true);

    let opengl = OpenGL::V3_2;
    WindowSettings::new(title, [window_width, window_height])
      .samples(samples)
      .fullscreen(fullscreen)
      .exit_on_esc(exit_on_esc)
      .vsync(vsync)
      .srgb(srgb)
      .resizable(resizable)
      .decorated(decorated)
      .controllers(controllers)
      .opengl(opengl)
      .build()
      .unwrap_or_else(|e| { panic!("Error: Failed to build PistonWindow: {}", e) })
  }
}