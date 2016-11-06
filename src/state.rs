use piston_window;
use super::ecs;
use super::asset_manager;

pub enum StateTrans {
  Pop,
  Push(Box<State>),
  Swap(Box<State>),
  None
}

pub trait State {
  fn init(&mut self, window: &mut piston_window::PistonWindow, world: &mut ecs::World, asset_manager: &mut asset_manager::AssetManager) {}
  fn update(&mut self, window: &mut piston_window::PistonWindow, event: piston_window::Event, world: &mut ecs::World, asset_manager: &mut asset_manager::AssetManager) -> StateTrans;
  fn exit(&mut self, window: &mut piston_window::PistonWindow, world: &mut ecs::World, asset_manager: &mut asset_manager::AssetManager) {}
}