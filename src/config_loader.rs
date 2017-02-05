use yaml_rust::{Yaml, YamlLoader};
use piston_window::{PistonWindow, OpenGL, WindowSettings};
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::Read;

struct ConfigSettings<'a> {
  title: &'a str,
  width: u32,
  height: u32,
  samples: u8,
  fullscreen: bool,
  exit_on_esc: bool,
  vsync: bool,
  srgb: bool,
  resizable: bool,
  decorated: bool,
  controllers: bool
}

impl<'a> ConfigSettings<'a> {

  fn new(
    title: &'a str,
    width: u32,
    height: u32,
    samples: u8,
    fullscreen: bool,
    exit_on_esc: bool,
    vsync: bool,
    srgb: bool,
    resizable: bool,
    decorated: bool,
    controllers: bool
  ) -> ConfigSettings {
    ConfigSettings {
      title: title,
      width: width,
      height: height,
      samples: samples,
      fullscreen: fullscreen,
      exit_on_esc: exit_on_esc,
      vsync: vsync,
      srgb: srgb,
      resizable: resizable,
      decorated: decorated,
      controllers: controllers
    }
  }
}

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
    let doc: &Yaml = &docs[0];

    let settings = self.read_config(doc);

    let opengl = OpenGL::V3_2;
    WindowSettings::new(settings.title, [settings.width, settings.height])
      .samples(settings.samples)
      .fullscreen(settings.fullscreen)
      .exit_on_esc(settings.exit_on_esc)
      .vsync(settings.vsync)
      .srgb(settings.srgb)
      .resizable(settings.resizable)
      .decorated(settings.decorated)
      .controllers(settings.controllers)
      .opengl(opengl)
      .build()
      .unwrap_or_else(|e| { panic!("Error: Failed to build PistonWindow: {}", e) })
  }

  fn read_config<'a>(&'a self, doc: &'a Yaml) -> ConfigSettings {
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
    ConfigSettings::new(title, window_width, window_height, samples, fullscreen,
      exit_on_esc, vsync, srgb, resizable, decorated, controllers)
  }
}

#[cfg(test)]
mod config_tests {
  use std::path::Path;
  use std::fs::File;
  use std::error::Error;
  use std::io::Read;
  use yaml_rust::{Yaml, YamlLoader};

  // describe: the config loader

  // it should properly load default settings when given an invalid yaml
  #[test]
  fn test_load_default() {
    let file_str =
    "
    foo:
        - list1
    bar:
        - 1
    ";
    let docs = match YamlLoader::load_from_str(file_str) {
      Err(why) => panic!("Error: Couldn't load YAML docs from string: {}",
                         why.description()),
      Ok(docs) => docs,
    };
    let doc: &Yaml = &docs[0];

    let config_loader = super::ConfigLoader;
    let settings = config_loader.read_config(doc);

    assert_eq!(settings.title, "Game");
    assert_eq!(settings.width, 640);
    assert_eq!(settings.height, 480);
    assert_eq!(settings.samples, 0);
    assert_eq!(settings.fullscreen, false);
    assert_eq!(settings.exit_on_esc, false);
    assert_eq!(settings.vsync, false);
    assert_eq!(settings.srgb, true);
    assert_eq!(settings.resizable, true);
    assert_eq!(settings.decorated, true);
    assert_eq!(settings.controllers, true);
  }

  // it should properly load the given settings
  #[test]
  fn test_load_config() {
    let file_str =
    "
    title:
        - test title
    width:
        - 1024
    height:
        - 720
    ";
    let docs = match YamlLoader::load_from_str(file_str) {
      Err(why) => panic!("Error: Couldn't load YAML docs from string: {}",
                         why.description()),
      Ok(docs) => docs,
    };
    let doc: &Yaml = &docs[0];

    let config_loader = super::ConfigLoader;
    let settings = config_loader.read_config(doc);

    assert_eq!(settings.title, "test title");
    assert_eq!(settings.width, 1024);
    assert_eq!(settings.height, 720);
    assert_eq!(settings.samples, 0);
    assert_eq!(settings.fullscreen, false);
    assert_eq!(settings.exit_on_esc, false);
    assert_eq!(settings.vsync, false);
    assert_eq!(settings.srgb, true);
    assert_eq!(settings.resizable, true);
    assert_eq!(settings.decorated, true);
    assert_eq!(settings.controllers, true);
  }
}