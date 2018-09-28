//! This implement the bullet system that is used as a score function.

use super::*;


#[derive(Debug, Clone)]
pub struct Bullet {
    // The tag of the shooter.
    pub tag: usize,
    // The position of the bullet through space.
    pub pos: Point2,
    // Where it's heading.
    pub facing: f32,
    // We can compute only once the thrust of the bullet when we create it.
    pub thrust_vector: Vector2,
    // The ncollide2d shape to handle collision detection.
    pub nshape: Ball<f32>,
    // and its isometry.
    pub iso: na::Isometry2<f32>,
    // his is the object other actors will interact with to detect collision and ray casting.
    pub body: Body,
    // The velocity of the bullet as a Vector2 handle by ggez.
    pub velocity: Vector2,
    // Store if a bullet should be removed from the gameboard next frame in case it's outside of
    // the gameboard or it touched a panda.
    pub to_remove: bool,
    // nalgebra velocity. Because ggez & nalgebra doesn't use the same object version
    // they are not interoperable with each other.
    // pub navelocity: na::Vector2<f32>,
    // The radius of a bullet's hitbox.
    radius: f32,
    // The color of the panda that fired this bullet.
    color: [f32; 4],
}

impl Bullet {
    pub fn new(tag: usize, pos: Point2, facing: f32, color: &[f32; 4]) -> Self {
        let thrust_vector: Vector2 = vec_from_angle(facing) * ACTOR_THRUST;
        let nshape = Ball::new(BULLET_RADIUS);
        let iso = na::Isometry2::new(na::Vector2::new(pos.x, pos.y), na::zero());

        Bullet {
            tag,
            // pos: pos.clone(),
            pos: Point2::new(pos.x, pos.y),
            facing,
            thrust_vector,
            iso,
            body: Body::new(tag, true, &nshape, &iso),
            nshape,
            velocity: thrust_vector,
            to_remove: false,
            radius: BULLET_RADIUS,
            color: *color,
            // color: color.clone(),
        }
    }

    /// Updates the position of a bullet.
    pub fn update(&mut self, ctx: &mut Context, dt: f32) -> GameResult<()> {
        // Clamp the velocity to the max efficiently
        let norm_sq = self.velocity.norm_squared();
        if norm_sq > MAX_PHYSICS_VEL.powi(2) {
            self.velocity = self.velocity / norm_sq.sqrt() * MAX_PHYSICS_VEL;
        }
        // self.pos += self.velocity * dt;
        self.pos += self.velocity * dt * BULLET_SPEED_FACTOR;

        self.iso.translation.vector.x = self.pos.x;
        self.iso.translation.vector.y = self.pos.y;

        self.body.update(&self.iso);

        // Checks wether we should remove this bullet from the gameboard if it goes out of scope.
        self.to_remove = !self.in_bbox(ctx);

        Ok(())
    }

    /// Draw the bullet on the gameboard.
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mb = &mut graphics::MeshBuilder::new();

        mb.circle(graphics::DrawMode::Fill, self.pos, self.radius, 0.1);

        let mesh: graphics::Mesh = mb
            .build(ctx)
            .unwrap_or_else(|_| panic!("Fail to build the bullet mesh with tag {}", self.tag));
        //     .expect(&format!(
        //     "Fail to build the bullet mesh with tag {}",
        //     self.tag
        // ));

        let drawparam = graphics::DrawParam {
            dest: self.pos,
            rotation: -self.facing as f32,
            offset: self.pos,
            // TODO: a bullet should have the same color as the panda that fired it.
            color: Some(self.color.into()),
            ..graphics::DrawParam::default()
        };

        graphics::draw_ex(ctx, &mesh, drawparam)?;

        Ok(())
    }

    /// Tells us if the bullet is outside the gameboard and should be removed from it.
    pub fn in_bbox(&self, ctx: &mut Context) -> bool {
        if self.pos.x < 0.0 || self.pos.x > ctx.conf.window_mode.width as f32 / 2.0 {
            return false;
        }
        if self.pos.y < 0.0 || self.pos.y > ctx.conf.window_mode.height as f32 / 2.0 {
            return false;
        }
        true
    }
}
