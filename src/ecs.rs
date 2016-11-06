use std::any::TypeId;
use std::collections::{hash_map, HashMap, HashSet};
use std::collections::hash_set;
use mopa::Any;

use std::usize;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Entity(pub usize);

pub trait Component: Any + Sized {}

trait Store: Any {
  fn store_remove(&mut self, e: Entity);
}

mopafy!(Store);

struct ComponentStore<T: Component> {
  data: HashMap<Entity, T>,
}

impl<T: Component> Store for ComponentStore<T> {
  fn store_remove(&mut self, e: Entity) {
    self.remove(e);
  }
}

impl<T: Component> ComponentStore<T> {
  fn new() -> ComponentStore<T> {
    ComponentStore { data: HashMap::new() }
  }

  fn insert(&mut self, e: Entity, comp: T) {
    self.data.insert(e, comp);
  }

  fn iter(&self) -> hash_map::Iter<Entity, T>{
    self.data.iter()
  }

  fn iter_mut(&mut self) -> hash_map::IterMut<Entity, T> {
    self.data.iter_mut()
  }

  fn remove(&mut self, e: Entity) -> Option<T> {
    self.data.remove(&e)
  }
}

struct WorldState {
  current_id: usize,
  reusable_ids: Vec<usize>,
  active: HashSet<Entity>,
  components: HashMap<((), TypeId), Box<Store>>
}

impl WorldState {
  fn new() -> WorldState {
    WorldState {
      current_id: 0,
      reusable_ids: Vec::new(),
      active: HashSet::new(),
      components: HashMap::new()
    }
  }
}

pub struct World {
  world_state_stack: Vec<WorldState>
}

// we don't want to expose these 3 functions to the client with the rest of World's methods
pub fn push_state(world: &mut World) {
  world.world_state_stack.push(WorldState::new());
}

pub fn pop_state(world: &mut World) {
  world.world_state_stack.pop().expect("Error: Attempted to pop empty ecs world state stack");
}

pub fn switch_state(world: &mut World) {
  pop_state(world);
  push_state(world);
}

impl World {
  pub fn new() -> World {
    World {
      world_state_stack: vec![
        WorldState {
          current_id: 0,
          reusable_ids: Vec::new(),
          active: HashSet::new(),
          components: HashMap::new()
        }
      ]
    }
  }

  fn current_state(&self) -> &WorldState {
    self.world_state_stack.last().expect("Error: Could not find ecs world state")
  }

  fn current_state_mut(&mut self) -> &mut WorldState {
    self.world_state_stack.last_mut().expect("Error: Could not find ecs world state (mut)")
  }

  pub fn create(&mut self) -> Entity {
    let mut world_state = self.current_state_mut();
    if world_state.current_id <= usize::MAX {
      let new_id = world_state.reusable_ids.pop().unwrap_or(
        {
          let current_id = world_state.current_id;
          world_state.current_id += 1;
          current_id
        }
      );
      let entity = Entity(new_id);
      world_state.active.insert(entity);
      entity
    } else {
      panic!("Error: Exceeded maximum entity limit")
    }
  }

  pub fn register_comp<T: Component>(&mut self) {
    let mut world_state = self.current_state_mut();
    world_state.components.insert(((), TypeId::of::<T>()), Box::new(ComponentStore::<T>::new()));
  }

  pub fn add_comp<T: Component>(&mut self, e: Entity, comp: T) {
    let mut world_state = self.current_state_mut();
    world_state.components.get_mut(&((), TypeId::of::<T>()))
      .and_then(|store| store.downcast_mut::<ComponentStore<T>>()
        .map(|typed_store| typed_store.insert(e, comp)))
      .expect("Error: Could not add component to entity; Could not find corresponding registered component type")
  }

  pub fn get_comp<T: Component>(&self) -> hash_map::Iter<Entity, T> {
    let world_state = self.current_state();
    world_state.components.get(&((), TypeId::of::<T>()))
      .and_then(|store| store.downcast_ref::<ComponentStore<T>>()
        .map(|typed_store| typed_store.iter()))
      .expect("Error: Could not find component of given type to retrieve")
  }

  pub fn get_comp_mut<T: Component>(&mut self) -> hash_map::IterMut<Entity, T> {
    let mut world_state = self.current_state_mut();
    world_state.components.get_mut(&((), TypeId::of::<T>()))
      .and_then(|store| store.downcast_mut::<ComponentStore<T>>()
        .map(|typed_store| typed_store.iter_mut()))
      .expect("Error: Could not find component of given type to retrieve (mut)")
  }

  pub fn contains(&self, e: Entity) -> bool {
    let world_state = self.current_state();
    world_state.active.contains(&e)
  }

  pub fn remove(&mut self, e: Entity) {
    let world_state = self.current_state_mut();
    if world_state.active.contains(&e) {
      world_state.reusable_ids.push(e.0);
      world_state.active.remove(&e);
      for comp_store in world_state.components.values_mut() {
        comp_store.store_remove(e);
      }
    }
  }

  pub fn iter(&self) -> hash_set::Iter<Entity> {
    let world_state = self.current_state();
    world_state.active.iter()
  }
}

#[cfg(test)]
mod ecs_tests {
  use super::*;

  // describe: an ECS World

  // it should allow a new Entity to be created
  #[test]
  fn test_create_entity() {
    let mut test_world = World::new();
    test_world.create();
  }

  // it should create entities with ascending ids
  #[test]
  fn test_entity_ids() {
    let mut test_world = World::new();
    let entity0 = test_world.create();
    let entity1 = test_world.create();
    assert_eq!(entity0.0, 0);
    assert_eq!(entity1.0, 1);
  }

  struct TestComponent {
    x: usize
  }

  impl Component for TestComponent {}

  // it should correctly add a component to an entity
  #[test]
  fn test_add_comp() {
    let mut test_world = World::new();
    let entity = test_world.create();
    test_world.register_comp::<TestComponent>();
    test_world.add_comp(entity, TestComponent{ x: 6 });
    let test_comp = test_world.get_comp::<TestComponent>();
    assert_eq!(test_comp.len(), 1);
    for (e, comp) in test_comp {
      assert_eq!(e.0, 0);
      assert_eq!(comp.x, 6);
    }
  }

  // it should allow an entity's components to be modified
  #[test]
  fn test_modify_comp() {
    let mut test_world = World::new();
    let entity = test_world.create();
    test_world.register_comp::<TestComponent>();
    test_world.add_comp(entity, TestComponent{ x: 6 });
    {
      let test_comp = test_world.get_comp_mut::<TestComponent>();
      assert_eq!(test_comp.len(), 1);
      for (e, comp) in test_comp {
        assert_eq!(e.0, 0);
        assert_eq!(comp.x, 6);
        comp.x += 1;
        assert_eq!(comp.x, 7);
      }
    }
    let test_comp_again = test_world.get_comp::<TestComponent>();
    assert_eq!(test_comp_again.len(), 1);
    for (e, comp) in test_comp_again {
      assert_eq!(e.0, 0);
      assert_eq!(comp.x, 7);
    }
  }

  // it should allow an entity to be removed
  #[test]
  fn test_remove() {
    let mut test_world = World::new();
    let entity = test_world.create();
    assert_eq!(test_world.contains(entity), true);
    test_world.remove(entity);
    assert_eq!(test_world.contains(entity), false);
  }

  // it should properly handle pushing a new state on to the state stack
  #[test]
  fn test_push() {
    let mut test_world = World::new();
    let entity = test_world.create();
    assert_eq!(test_world.contains(entity), true);
    push_state(&mut test_world);
    assert_eq!(test_world.contains(entity), false);
  }

  // it should properly handle popping a state off of the state stack
  #[test]
  fn test_pop() {
    let mut test_world = World::new();
    let entity = test_world.create();
    assert_eq!(entity.0, 0);
    test_world.register_comp::<TestComponent>();
    test_world.add_comp(entity, TestComponent{ x: 6 });
    {
      let test_comp = test_world.get_comp::<TestComponent>();
      assert_eq!(test_comp.len(), 1);
      for (_, comp) in test_comp {
        assert_eq!(comp.x, 6);
      }
    }
    push_state(&mut test_world);
    test_world.register_comp::<TestComponent>();
    let entity_pushed = test_world.create();
    assert_eq!(entity_pushed.0, 0);
    test_world.add_comp(entity_pushed, TestComponent { x: 3 });
    {
      let test_comp_pushed = test_world.get_comp::<TestComponent>();
      assert_eq!(test_comp_pushed.len(), 1);
      for (_, comp) in test_comp_pushed {
        assert_eq!(comp.x, 3);
      }
    }
    pop_state(&mut test_world);
    let test_comp_after_pop = test_world.get_comp::<TestComponent>();
    assert_eq!(test_comp_after_pop.len(), 1);
    for (_, comp) in test_comp_after_pop {
      assert_eq!(comp.x, 6);
    }
  }

  // it should properly handle swapping states on the state stack
  #[test]
  fn test_switch() {
    let mut test_world = World::new();
    let entity = test_world.create();
    assert_eq!(entity.0, 0);
    test_world.register_comp::<TestComponent>();
    test_world.add_comp(entity, TestComponent{ x: 6 });
    {
      let test_comp = test_world.get_comp::<TestComponent>();
      assert_eq!(test_comp.len(), 1);
      for (_, comp) in test_comp {
        assert_eq!(comp.x, 6);
      }
    }
    push_state(&mut test_world);
    test_world.register_comp::<TestComponent>();
    let entity_pushed = test_world.create();
    assert_eq!(entity_pushed.0, 0);
    test_world.add_comp(entity_pushed, TestComponent { x: 3 });
    {
      let test_comp_pushed = test_world.get_comp::<TestComponent>();
      assert_eq!(test_comp_pushed.len(), 1);
      for (_, comp) in test_comp_pushed {
        assert_eq!(comp.x, 3);
      }
    }
    switch_state(&mut test_world);
    assert_eq!(test_world.contains(entity_pushed), false);
    pop_state(&mut test_world);
    let test_comp_after_pop = test_world.get_comp::<TestComponent>();
    assert_eq!(test_comp_after_pop.len(), 1);
    for (_, comp) in test_comp_after_pop {
      assert_eq!(comp.x, 6);
    }
  }

  // it should panic when trying to pop an empty world state stack
  #[test]
  #[should_panic(expected = "Error: Attempted to pop empty ecs world state stack")]
  fn test_pop_empty_world_state_stack() {
    let mut test_world = World::new();
    pop_state(&mut test_world);
    pop_state(&mut test_world);
  }

  // it should panic when trying to switch states with an empty world state stack
  #[test]
  #[should_panic(expected = "Error: Attempted to pop empty ecs world state stack")]
  fn test_switch_empty_state_stack() {
    let mut test_world = World::new();
    pop_state(&mut test_world);
    switch_state(&mut test_world);
  }

  // it should panic when trying to get a reference to the current state from an empty state stack
  #[test]
  #[should_panic(expected = "Error: Could not find ecs world state")]
  fn test_current_state_empty_state_stack() {
    let mut test_world = World::new();
    pop_state(&mut test_world);
    test_world.current_state();
  }

  // it should panic when trying to get a mutable reference to the current state from an empty state stack
  #[test]
  #[should_panic(expected = "Error: Could not find ecs world state (mut)")]
  fn test_current_mut_state_empty_state_stack() {
    let mut test_world = World::new();
    pop_state(&mut test_world);
    test_world.current_state_mut();
  }

  // it should panic when trying to add a new component to an entity before registering the component
  #[test]
  #[should_panic(expected = "Error: Could not add component to entity; Could not find corresponding registered component type")]
  fn test_add_comp_no_registration() {
    let mut test_world = World::new();
    let entity = test_world.create();
    test_world.add_comp(entity, TestComponent{ x: 6 });
  }

  // it should panic when trying to get a reference to an unregistered component
  #[test]
  #[should_panic(expected = "Error: Could not find component of given type to retrieve")]
  fn test_get_comp_no_registration() {
    let mut test_world = World::new();
    test_world.create();
    test_world.get_comp::<TestComponent>();
  }

  // it should panic when trying to get a mutable reference to an unregistered component
  #[test]
  #[should_panic(expected = "Error: Could not find component of given type to retrieve (mut)")]
  fn test_get_comp_mut_no_registration() {
    let mut test_world = World::new();
    test_world.create();
    test_world.get_comp_mut::<TestComponent>();
  }

  // it should return Unit when trying to remove an entity that is not active
  #[test]
  fn test_remove_inactive() {
    let mut test_world = World::new();
    assert_eq!(test_world.remove(Entity(50)), ());
  }

}