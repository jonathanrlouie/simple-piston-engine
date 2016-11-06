use std::collections::HashMap;
use piston_window::G2dTexture;

use std::fs;

pub struct AssetManager {
  textures: HashMap<String, G2dTexture<'static>>,
  sounds: HashMap<String, fs::File>
}

impl AssetManager {
  pub fn new() -> AssetManager {
    AssetManager {
      textures: HashMap::new(),
      sounds: HashMap::new()
    }
  }

  pub fn add_texture(&mut self, name: &str, tex: G2dTexture<'static>) {
    self.textures.insert(name.into(), tex);
  }

  pub fn load_texture(&self, name: &str) -> &G2dTexture {
    self.textures.get(name).expect("No texture with the given name was found")
  }

  pub fn add_sound(&mut self, name: &str, sound: fs::File) {
    self.sounds.insert(name.into(), sound);
  }

  pub fn get_sound(&self, name: &str) -> &fs::File {
    self.sounds.get(name).expect("No sound with the given name was found")
  }
}