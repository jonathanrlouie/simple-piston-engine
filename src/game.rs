use piston_window::PistonWindow;
use super::ecs;
use super::state::{State, StateTrans};
use super::asset_manager::AssetManager;
use super::config_loader;

pub struct Game {
  state_stack: Vec<Box<State>>
}

impl Game {
  pub fn new<T>(init_state: T) -> Game where T: State + 'static {
    Game {
      state_stack: vec![Box::new(init_state)]
    }
  }

  pub fn start_game(&mut self, config_path: &str) {
    let mut window: PistonWindow = config_loader::ConfigLoader.load_config(config_path);
    let mut world = ecs::World::new();
    let mut asset_manager = AssetManager::new();

    self.current_state().init(&mut window, &mut world, &mut asset_manager);

    while let Some(event) = window.next() {
      let state_trans = self.current_state().update(&mut window, event, &mut world, &mut asset_manager);
      match state_trans {
        StateTrans::None => (),
        StateTrans::Pop => {
          self.current_state().exit(&mut window, &mut world, &mut asset_manager);
          self.pop(&mut world)
        },
        StateTrans::Push(state) => {
          self.push(&mut world, state);
          self.current_state().init(&mut window, &mut world, &mut asset_manager);
        },
        StateTrans::Swap(state) => {
          self.current_state().exit(&mut window, &mut world, &mut asset_manager);
          self.switch(&mut world, state);
          self.current_state().init(&mut window, &mut world, &mut asset_manager);
        }
      }
    }
  }

  fn current_state(&mut self) -> &mut Box<State> {
    self.state_stack.last_mut().expect("Error: Could not find current state; empty state stack")
  }

  fn pop(&mut self, world: &mut ecs::World) {
    ecs::pop_state(world);
    self.state_stack.pop().expect("Error: Attempted to pop empty state stack");
  }

  fn push(&mut self, world: &mut ecs::World, state: Box<State>) {
    ecs::push_state(world);
    self.state_stack.push(state);
  }

  fn switch(&mut self, world: &mut ecs::World, state: Box<State>) {
    ecs::switch_state(world);
    self.state_stack.pop().expect("Error: Attempted to switch states with empty state stack");
    self.state_stack.push(state);
  }

}