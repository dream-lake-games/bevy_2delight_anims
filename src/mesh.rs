use bevy::{ecs::world::DeferredWorld, prelude::*, utils::HashMap};

#[derive(Resource, Default)]
pub(super) struct AnimQuickMesh {
    map: HashMap<(u32, u32), Handle<Mesh>>,
}
impl AnimQuickMesh {
    /// Will try to use an existing mesh2d if one with the right size exists and is loaded, otherwise
    /// makes a new one
    pub(super) fn get_or_make_mesh2d_from_world(
        world: &mut DeferredWorld,
        size: UVec2,
        rep: UVec2,
    ) -> Mesh2d {
        let quick_mesh = world.resource::<AnimQuickMesh>();
        let key = (size.x * rep.x, size.y * rep.y);
        let mut valid_hand = None;
        if let Some(maybe_hand) = quick_mesh.map.get(&key) {
            let ass = world.resource::<AssetServer>();
            if ass.is_loaded(maybe_hand.id()) {
                valid_hand = Some(maybe_hand.clone());
            } else {
                let mut quick_mesh = world.resource_mut::<AnimQuickMesh>();
                quick_mesh.map.remove(&key);
            }
        }
        match valid_hand {
            Some(ass_id) => Mesh2d(ass_id),
            None => {
                let underlying = Mesh::from(Rectangle::new(
                    size.x as f32 * rep.x as f32,
                    size.y as f32 * rep.y as f32,
                ));
                let new_hand = world.resource_mut::<Assets<Mesh>>().add(underlying);
                let mut quick_mesh = world.resource_mut::<AnimQuickMesh>();
                quick_mesh
                    .map
                    .insert((size.x * rep.x, size.y * rep.y), new_hand.clone_weak());
                Mesh2d(new_hand)
            }
        }
    }
}
