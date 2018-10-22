//! This is where all our actors will be stored.

mod panda;
mod sensor;
mod bullet;

pub use self::panda::*;
pub use self::sensor::*;
pub use self::bullet::*;


use color_picker::BLACK2;
use gameboard_controller::InputState;
use ggez::graphics::{Point2, Vector2};
use ggez::{graphics, nalgebra, Context, GameResult};
use rand::{thread_rng, Rng};

use na;
use ncollide2d;
use ncollide2d::query::{self, RayCast};
use ncollide2d::shape::Ball;

// The area around a panda that trigger some score.
// const HITBOX_SIZE: f32 = 2.5;
const HITBOX_SIZE: f32 = 3.0;
// The radius of the hitbox.
const HITBOX_RADIUS: f32 = HITBOX_SIZE / 2.0;
// The maximum velocity our stuff can reach.
const MAX_PHYSICS_VEL: f32 = 250.0;
// Acceleration in pixels per second.
const ACTOR_THRUST: f32 = 2000.0;
// Rotation in radians per second.
const ACTOR_TURN_RATE: f32 = 1.0;
// Turn rate of the sensor, it also define how close the sensor can get with each other.
const SENSOR_TURN_RATE: f32 = 1.0;
// The value of nothingness from a sensor.
const NOTHINGNESS: f32 = 999.0;
// The distance a Sensor / eye can 'see'.
// const SENSOR_MAX_DIST: f32 = HITBOX_SIZE * 100.0;
const SENSOR_MAX_DIST: f32 = HITBOX_SIZE * 50.0;
// The radius of a bullet hitbox.
const BULLET_RADIUS: f32 = HITBOX_RADIUS / 1.5;
// Bullet speed factor.
const BULLET_SPEED_FACTOR: f32 = 3.00;
// Time to wait between 2 shots in second.
const SHOOTING_COOLDOWN: f32 = 1.0;
// Number of output from each Sensor.
const SENSOR_OUTPUT_LEN: usize = 3;
// The length of the output array that will be passed to the A.I. engine.
pub const AI_ENGINE_INPUT_LEN: usize = 8;
// The length of the output computed from the A.I. engine.
// It correspond to the range of instructions a Panda
// can receive from its 'brain'.
pub const AI_ENGINE_OUTPUT_LEN: usize = 7;


/// Some helper function.

/// Create a unit vector representing the
/// given angle (in radians)
fn vec_from_angle(angle: f32) -> Vector2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

fn na_vec_from_angle(angle: f32) -> na::Vector2<f32> {
    let vx = angle.sin();
    let vy = angle.cos();
    na::Vector2::new(vx, vy)
}



/// Tis is a helper structure which aim at easing the interaction with ncollide2d and ray casting
/// on Pandas and Bullets.
#[derive(Debug, Clone)]
pub struct Body {
    // This is the tag of a Panda or its bullet.
    pub tag: usize,
    // Tells us if it's a bullet or not in order to prioritize bullets over anything else.
    pub is_bullet: bool,
    // The ncollide2d shape of the body we want to cast a ray upon.
    pub nshape: Ball<f32>,
    // And its isometry.
    pub isometry: na::Isometry2<f32>,
}

impl Body {
    pub fn new(
        tag: usize,
        is_bullet: bool,
        nshape: &Ball<f32>,
        isometry: &na::Isometry2<f32>,
    ) -> Self {
        Body {
            tag,
            is_bullet,
            nshape: nshape.clone(),
            isometry: *isometry,
        }
    }

    /// Update the position of a Body from the na::Isometry2 of its owner.
    pub fn update(&mut self, iso: &na::Isometry2<f32>) {
        self.isometry.translation.vector.x = iso.translation.vector.x;
        self.isometry.translation.vector.y = iso.translation.vector.y;
    }

    /// Retruns if 2 bodies are in contact with each other.
    pub fn in_contact(&self, body: &Body) -> bool {
        let dist = query::distance(&self.isometry, &self.nshape, &body.isometry, &body.nshape);
        dist <= 0.0
    }
}
