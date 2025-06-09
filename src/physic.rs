use agb::display::{HEIGHT, WIDTH};
use bevy::prelude::*;
pub struct PhysicPlugin;

impl Plugin for PhysicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, move_physic_objects);
        app.init_resource::<PhysicConfig>();
        app.add_systems(
            PostUpdate,
            (detect_collision, keep_object_in_boundary)
                .chain()
                .after(TransformSystem::TransformPropagate),
        );
        app.add_observer(handle_collision);
    }
}

#[derive(Resource, Default)]
pub struct PhysicConfig {
    pub boundary: Rect,
}

impl PhysicConfig {
    pub fn screen_boundary() -> Self {
        let mut new_val = Self::default();
        new_val.set_screen_boundary();
        new_val
    }

    pub fn set_screen_boundary(&mut self) {
        self.boundary.min = Vec2::ZERO;
        self.boundary.max = Vec2::new(WIDTH as f32, HEIGHT as f32);
    }

    pub fn with_screen_boundary(mut self) -> Self {
        self.set_screen_boundary();
        self
    }
}

#[derive(Component)]
#[require(PhysicObject, Transform)]
pub struct CircleCollider {
    pub radius: u8,
}

impl CircleCollider {
    pub fn center(&self, global_transform: &GlobalTransform) -> Vec3 {
        global_transform.translation() + Vec3::new(self.radius as f32, self.radius as f32, 0.)
    }
}

impl From<u8> for CircleCollider {
    fn from(value: u8) -> Self {
        Self { radius: value }
    }
}

#[derive(Component, Debug)]
pub struct PhysicObject {
    pub enable: bool,
    pub mass: f32,
    pub impulse: Vec2,
    pub velocity: Vec2,
    pub drag: f32,
}

impl Default for PhysicObject {
    fn default() -> Self {
        Self {
            enable: true,
            mass: 1.,
            impulse: Vec2::ZERO,
            velocity: Vec2::ZERO,
            drag: 0.5,
        }
    }
}

const SPEED_SQUARED_TO_ZERO: f32 = 1.;

fn move_physic_objects(
    time: Res<Time<Fixed>>,
    mut physic_objects: Query<(&mut PhysicObject, &mut Transform)>,
) {
    let elapsed = time.delta_secs();
    for (mut object, mut transform) in &mut physic_objects {
        if !object.enable {
            continue;
        }

        let impulse = object.impulse;
        let mass = object.mass;
        let velocity = object.velocity;
        let drag = object.drag;
        object.velocity += impulse / mass - (velocity * drag) * elapsed;

        transform.translation += (object.velocity * elapsed).extend(0.);

        if object.velocity.length_squared() < SPEED_SQUARED_TO_ZERO {
            object.velocity = Vec2::ZERO;
        }

        object.impulse = Vec2::ZERO;
    }
}

#[derive(Event, Copy, Clone)]
pub struct Collision {
    pub entity1: Entity,
    pub entity2: Entity,
}

impl Collision {
    pub fn self_and_other_trigger<T>(&self, trigger: Trigger<T>) -> Option<(Entity, Entity)> {
        self.self_and_other(&trigger.target())
    }

    pub fn self_and_other(&self, target: &Entity) -> Option<(Entity, Entity)> {
        if *target == self.entity1 {
            Some((self.entity1, self.entity2))
        } else if *target == self.entity2 {
            Some((self.entity2, self.entity1))
        } else {
            None
        }
    }
}

pub fn detect_collision(
    mut commands: Commands,
    collider_query: Query<(Entity, &PhysicObject, &CircleCollider, &GlobalTransform)>,
) {
    let colliders: Vec<(Entity, &PhysicObject, &CircleCollider, &GlobalTransform)> =
        collider_query.iter().collect();

    for (entity, object, collider, transform) in &colliders {
        if !object.enable {
            continue;
        }

        for (other_entity, other_object, other_collider, other_transform) in &colliders {
            if entity == other_entity {
                break;
            }

            if !other_object.enable {
                continue;
            }

            let distance_for_collision = (collider.radius + other_collider.radius) as f32;
            let distance_squared_for_collision = distance_for_collision * distance_for_collision;
            let distance_squared = collider
                .center(transform)
                .distance_squared(other_collider.center(other_transform));

            if distance_squared < distance_squared_for_collision {
                let collision = Collision {
                    entity1: *entity,
                    entity2: *other_entity,
                };
                commands.trigger_targets(collision, [entity.clone(), other_entity.clone()]);
            }
        }
    }
}

fn bounce(physic_object: &mut PhysicObject, normal: Vec2) {
    let velocity = physic_object.velocity;

    let velocity_along_normal = velocity.dot(normal);
    if velocity_along_normal >= 0. {
        // Nothing to do since the object is already going in the right direction
        return;
    }

    let impulse = -2. * velocity_along_normal * physic_object.mass * normal;
    physic_object.impulse += impulse;
}

fn keep_object_in_boundary(
    config: Res<PhysicConfig>,
    collider_query: Query<(&mut PhysicObject, &CircleCollider, &GlobalTransform)>,
) {
    let left = config.boundary.min.x;
    let top = config.boundary.min.y;
    let right = config.boundary.max.x;
    let bottom = config.boundary.max.y;

    for (mut physic_object, collider, transform) in collider_query {
        if !physic_object.enable {
            continue;
        }

        let position = collider.center(transform);
        let radius = collider.radius as f32;

        if position.y - radius < top {
            // The circle is hiting the top side
            bounce(&mut physic_object, Vec2::Y);
        } else if position.y + radius > bottom {
            // The circle is hiting the bottom side
            bounce(&mut physic_object, Vec2::NEG_Y);
        }

        if position.x - radius < left {
            // The circle is hiting the left side
            bounce(&mut physic_object, Vec2::X);
        } else if position.x + radius > right {
            // The circle is hiting the right side
            bounce(&mut physic_object, Vec2::NEG_X);
        }
    }
}

// https://code.tutsplus.com/how-to-create-a-custom-2d-physics-engine-the-basics-and-impulse-resolution--gamedev-6331t
fn handle_collision(
    collision: Trigger<Collision>,
    mut physic_objects: Query<(&mut PhysicObject, &GlobalTransform)>,
) -> Result {
    let target = collision.target();
    let entity1 = collision.entity1;
    let entity2 = collision.entity2;

    if target == entity2 {
        // Event are duplicated on each physic objects.
        // We only handle the collision on one of them.
        return Ok(());
    }

    let [(mut po1, gt1), (mut po2, gt2)] = physic_objects.get_many_mut([entity1, entity2])?;

    let t1 = gt1.translation();
    let t2 = gt2.translation();

    let normal = (t2 - t1).normalize().truncate();

    let rv = po2.velocity - po1.velocity;

    let vel_along_normal = normal.dot(rv);

    if vel_along_normal > 0. {
        return Ok(());
    }

    let m1 = 1. / po1.mass;
    let m2 = 1. / po2.mass;

    let e: f32 = 1.;

    let mut j = -(1. + e) * vel_along_normal;
    j /= m1 + m2;

    let impulse = j * normal;

    po1.impulse -= m1 * impulse;
    po2.impulse += m2 * impulse;

    Ok(())
}
