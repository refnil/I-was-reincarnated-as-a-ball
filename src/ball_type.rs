use bevy::{
    ecs::{children, component::Component},
    prelude::*,
    transform::components::Transform,
};
use bevy_mod_gba::Sprite;

use crate::{
    level::Level,
    physic::{CircleCollider, Collision, PhysicObject, detect_collision},
    render::{AffineSprite, RepeatedSprite},
    sprite_loader::Sprites,
};

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(reduce_life);
        app.add_systems(
            PostUpdate,
            (rotate_balls, despawn).chain().after(detect_collision),
        );
    }
}

fn reduce_life(collision: Trigger<Collision>, mut balls: Query<(&mut Life, &Team)>) {
    let Some((self_entity, other_entity)) = collision.self_and_other(&collision.target()) else {
        return;
    };

    if balls
        .get(self_entity)
        .map_or(true, |(_life, team)| team.is_enemy())
    {
        return;
    }

    let Ok((mut enemy_life, Team::Enemy(_))) = balls.get_mut(other_entity) else {
        return;
    };

    if **enemy_life > 0 {
        // info!("collision causing reducing the life");
        **enemy_life -= 1
    }
}

fn rotate_balls(balls: Query<(&PhysicObject, &mut Transform), With<Team>>) {
    for (po, mut transform) in balls {
        let velocity = po.velocity.length_squared();

        if velocity > 5. {
            let side = if po.velocity.x >= 0. { 1. } else { -1. };
            let angle = side * (velocity / 10000.).clamp(0.01, 0.025);
            transform.rotate_z(angle);
        }
    }
}

fn despawn(
    mut commands: Commands,
    lifes: Query<(Entity, &Life, &Team, &GlobalTransform, &PhysicObject), Changed<Life>>,
    sprites: NonSend<Sprites>,
) {
    for (entity, life, team, transform, physic_object) in lifes {
        if **life == 0 {
            // info!("Destroying a ball");
            commands.entity(entity).despawn();

            if *team == Team::Enemy(EnemyBall::RedBlob) {
                let length = physic_object.impulse.length();
                let impulse = physic_object.impulse.normalize();
                let normal = Vec2::new(impulse.y, -impulse.x);

                let base = transform.translation();
                let impulse1 = impulse + normal;
                let pos1 = 9. * impulse1.extend(0.) + base;

                // info!("Spawn green {impulse1} {pos1}");

                let impulse2 = impulse - normal;
                let pos2 = 9. * impulse2.extend(0.) + base;

                commands
                    .spawn((
                        Transform::from_translation(pos1),
                        EnemyBall::GreenBlob.to_bundle(&sprites),
                    ))
                    .insert(PhysicObject {
                        impulse: impulse1 * length,
                        ..EnemyBall::GreenBlob.physic_object()
                    });

                commands
                    .spawn((
                        Transform::from_translation(pos2),
                        EnemyBall::GreenBlob.to_bundle(&sprites),
                    ))
                    .insert(PhysicObject {
                        impulse: impulse2 * length,
                        ..EnemyBall::GreenBlob.physic_object()
                    });
            }
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
#[require(Life, Transform, PhysicObject, Level)]
pub enum Team {
    Player(PlayerBall),
    Enemy(EnemyBall),
}

impl Team {
    pub fn is_friend(&self) -> bool {
        match self {
            Team::Player(_) => true,
            Team::Enemy(EnemyBall::Tree) => true,
            Team::Enemy(_) => false,
        }
    }

    pub fn is_enemy(&self) -> bool {
        !self.is_friend()
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Life(pub u8);

impl Default for Life {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PlayerBall {
    Boy,
    Princess,
    Dog,
}

impl PlayerBall {
    fn sprite(&self, sprites: &Sprites) -> Sprite {
        match self {
            Self::Boy => sprites.boy.clone(),
            Self::Princess => sprites.princess.clone(),
            Self::Dog => sprites.dog.clone(),
        }
    }
}

impl PlayerBall {
    pub fn to_bundle(&self, sprites: &Sprites, physic_enabled: bool) -> impl Bundle {
        (
            PhysicObject {
                enable: physic_enabled,
                ..Default::default()
            },
            Team::Player(*self),
            CircleCollider::from(8),
            children![(
                self.sprite(sprites),
                AffineSprite::enabled(),
                RepeatedSprite::default(),
            )],
        )
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnemyBall {
    GreenBlob,
    RedBlob,
    Snake,
    Ghost,
    Tree,
}

impl EnemyBall {
    pub fn sprite(&self, sprites: &Sprites) -> Sprite {
        match self {
            Self::GreenBlob => sprites.green_blob.clone(),
            Self::RedBlob => sprites.red_blob.clone(),
            Self::Snake => sprites.snake.clone(),
            Self::Ghost => sprites.ghost.clone(),
            Self::Tree => sprites.tree.clone(),
        }
    }

    pub fn size(&self) -> u8 {
        match self {
            Self::Tree => 16,
            _ => 8,
        }
    }

    pub fn mass(&self) -> f32 {
        match self {
            Self::Tree => 10000.,
            Self::Snake => 0.6,
            _ => 1.,
        }
    }

    pub fn life(&self) -> u8 {
        match self {
            Self::Tree => u8::MAX,
            Self::Ghost => 6,
            Self::Snake => 2,
            _ => 1,
        }
    }

    pub fn physic_object(&self) -> PhysicObject {
        PhysicObject {
            mass: self.mass(),
            ..default()
        }
    }

    pub fn to_bundle(&self, sprites: &Sprites) -> impl Bundle {
        (
            Team::Enemy(*self),
            Life(self.life()),
            CircleCollider::from(self.size()),
            self.physic_object(),
            children![(
                self.sprite(sprites),
                AffineSprite::enabled(),
                RepeatedSprite::default(),
            )],
        )
    }
}
