//! A Panda is the main actor of the game and is controlled by the A.I. engine.



use super::*;

#[derive(Debug, Clone)]
pub struct Panda {
    // This is this id of our panda.
    pub tag: usize,
    // An array of float representing the color of our panda.
    pub color: [f32; 4],
    // This actual position of our panda.
    pub pos: Point2,
    // ncollide position as an Isometry2.
    pub isometry: na::Isometry2<f32>,
    // Where it's facing. Usefull for aiming/shooting/moving.
    facing: f32,
    // Facing representation as a Vector2.
    direction_vector: Vector2,
    // The velocity or 'speed'.
    velocity: Vector2,
    // Field Od Vision length.
    fov_length: f32,
    // nalgebra velocity. Because ggez & nalgebra doesn't use the same object version
    // they are not interoperable with each other.
    // pub navelocity: na::Vector2<f32>,
    // At this point in time, I'm not entirely sure what this is about ^^'.
    angle_vel: f32,
    // The size of the hitbox from the pos or the panda.
    hitbox_size: f32,
    // Radius of the hitbox.
    radius: f32,
    // ncollide shape to handle ray casting, collision detection etc.
    pub nshape: ncollide2d::shape::Ball<f32>,
    // This is the object other actors will interact with to detect collision and ray casting.
    // We build it during the creation of a Panda to avoid unnecessary allocation later.
    pub body: Body,
    // Ray Left. It's basically a sensor from a "virtual" left eye.
    pub sensor_left: Sensor,
    // Ray Right. It's basically a sensor from a "virtual" right eye.
    pub sensor_right: Sensor,
    // Time to wait between 2 shots.
    cooldown: f32,
    // The score of our lovely beast. How well it's doing in this harsh world.
    pub score: f32,
    // This array is use to feed the A.I. engine to inform it about the state of one Panda.
    pub input_to_ai: [f32; AI_ENGINE_INPUT_LEN],
}

impl Panda {
    /// Retruns a freshly borned panda with name as a usize tag.
    pub fn new(ctx: &Context, tag: usize, color: [f32; 4]) -> Self {
        let pos = Point2::new(
            thread_rng().gen_range(0.0, ctx.conf.window_mode.width as f32 / 2.0) as f32,
            thread_rng().gen_range(0.0, ctx.conf.window_mode.height as f32 / 2.0) as f32,
        );

        let na_pos = na::Point2::new(pos.x, pos.y);
        let facing = thread_rng().gen_range(0.0, 360.0);
        let nshape = ncollide2d::shape::Ball::new(HITBOX_RADIUS);
        let iso = na::Isometry2::new(na::Vector2::new(pos.x, pos.y), na::zero());

        Panda {
            tag,
            color,
            pos,
            isometry: iso,
            facing,
            direction_vector: vec_from_angle(facing),
            velocity: nalgebra::zero(),
            fov_length: 0.0,
            angle_vel: 0.0,
            hitbox_size: HITBOX_SIZE,
            radius: HITBOX_RADIUS,
            body: Body::new(tag, false, &nshape, &iso),
            nshape,
            sensor_left: Sensor::new(tag, na_pos, facing + 0.1, &color),
            sensor_right: Sensor::new(tag, na_pos, facing - 0.1, &color),
            cooldown: 0.0,
            score: 0.0,
            input_to_ai: [0.0; AI_ENGINE_INPUT_LEN],
        }
    }

    /// Updates our panda: cover everything from position to score etc.
    pub fn update(&mut self, ctx: &mut Context, body_vec: &[Body], wrap_world: bool, dt: f32) -> GameResult<()> {
        // Clamp the velocity to the max efficiently
        let norm_sq = self.velocity.norm_squared();
        if norm_sq > MAX_PHYSICS_VEL.powi(2) {
            self.velocity = self.velocity / norm_sq.sqrt() * MAX_PHYSICS_VEL;
            // self.navelocity = self.navelocity / self.navelocity.norm_squared() * MAX_PHYSICS_VEL;
        }
        let dv = self.velocity * dt;
        self.pos += dv;

        if wrap_world {
            self.wrap_position(ctx);
        } else {
            self.confine_position(ctx);
        }

        self.isometry.translation.vector.x = self.pos.x;
        self.isometry.translation.vector.y = self.pos.y;
        self.body.update(&self.isometry);

        // self.facing += self.angle_vel;
        let na_pos = na::Point2::new(self.pos.x, self.pos.y);

        // print!("\tL dist = ");
        self.sensor_left.update(na_pos, body_vec)?;
        // print!("\tR dist = ");
        self.sensor_right.update(na_pos, body_vec)?;
        // println!("");

        {
            // let ls_out = self.sensor_left.output.clone();
            let ls_out = self.sensor_left.output;
            // let rs_out = self.sensor_right.output.clone();
            let rs_out = self.sensor_right.output;
            self.build_output(ls_out, rs_out);
        }
        // Here we handle the possibility for a panda to shoot based on its cooldown.
        self.cooldown -= dt;

        Ok(())
    }

    /// Draw everything related to our panda.
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // We draw the sensors before the body of a Panda to avoid having the sensors's
        // strait lines cutting the their bodies.
        self.sensor_left.draw(ctx)?;
        self.sensor_right.draw(ctx)?;

        // Let's draw the body of our panda.
        {
            let mb = self.build_body_mesh(ctx)?;
            let drawparam = graphics::DrawParam {
                dest: self.pos,
                rotation: -self.facing as f32,
                offset: self.pos,
                // offset: Point2::new(0.5, 0.5),
                color: Some(self.color.into()),
                ..graphics::DrawParam::default()
            };

            graphics::draw_ex(ctx, &mb, drawparam)?;
        }



        // Let's draw the 'eye' of our panda, it's basically a visual indicator of
        // where it's actually looking.
        {
            let eye_mb = self.build_eye_mesh(ctx)?;
            let eye_drawparam = graphics::DrawParam {
                dest: self.pos,
                rotation: -self.facing as f32,
                offset: self.pos,
                color: Some(BLACK2.into()),
                ..graphics::DrawParam::default()
            };

            graphics::draw_ex(ctx, &eye_mb, eye_drawparam)?;
        }

        Ok(())
    }

    /// We need to use a 'MeshBuilder' in order to be able to rotate the whole
    /// actor structure when we turn.
    fn build_body_mesh(&mut self, ctx: &mut Context) -> GameResult<graphics::Mesh> {
        let mb = &mut graphics::MeshBuilder::new();
        mb.circle(graphics::DrawMode::Fill, self.pos, self.hitbox_size, 0.1);

        // Draws the body axis. Can be usefull diring debug.
        // mb.line(&[self.pos, Point2::new(self.pos[0], self.pos[1] + 300.0)], 0.50);

        mb.build(ctx)
    }

    /// This is a graphical representation of the direction in which our actor is currently facing.
    /// It's handled in a separate mesh so we can pick a different color from the body.
    fn build_eye_mesh(&mut self, ctx: &mut Context) -> GameResult<graphics::Mesh> {
        let mb = &mut graphics::MeshBuilder::new();

        let ellipse_pos_x = self.pos[0];
        let ellipse_pos_y = self.pos[1];
        // The position of the eye as an ellipse, must be computed
        // from the actual position of the actor and its body.
        let ellipse_pos = Point2::new(ellipse_pos_x, ellipse_pos_y + HITBOX_SIZE * (3.0 / 3.5));

        mb.ellipse(
            graphics::DrawMode::Fill,
            ellipse_pos,
            HITBOX_SIZE / 2.,
            HITBOX_SIZE / 4.0,
            0.1,
        );

        mb.build(ctx)
    }

    /// This is where the magic of movement takes place.
    pub fn handle_input(
        &mut self,
        input: &InputState,
        bullet_vector: &mut Vec<Bullet>,
        dt: f32,
    ) {
        let turn = dt * ACTOR_TURN_RATE * input.xaxis;
        self.facing += turn;
        self.direction_vector = vec_from_angle(self.facing);

        // Turn the sensors along the entire body.
        self.sensor_left.facing += turn;
        self.sensor_right.facing += turn;

        if input.yaxis != 0.0 {
            let thrust_vector = self.direction_vector * (ACTOR_THRUST);

            if input.yaxis > 0.0 {
                self.velocity += thrust_vector * (dt);
            } else {
                self.velocity -= thrust_vector * (dt);
            }
        } else {
            self.velocity *= 0.0;
        }

        // Freeze all motor functions.
        if input.freeze > 0.0 {
            self.velocity = nalgebra::zero();
        }

        // Compute this only once.
        let atan_direction_vector = self.direction_vector.y.atan2(self.direction_vector.x);

        // FOV input handling part.
        // Now it's used to handle the synchronized left and right Sensor movements.
        let future_fov_turn = dt * SENSOR_TURN_RATE * input.fov_axis;
        let future_fov_vector = vec_from_angle(self.sensor_right.facing + future_fov_turn);
        let tan_diff = atan_direction_vector - future_fov_vector.y.atan2(future_fov_vector.x);


        // println!("input.fov_axis = {} , tan_diff = {}", input.fov_axis, tan_diff);

        // Only a test on one of the sensor is enough to check if the next movement is in
        // range of the sensors angle limitation.
        if -2.0 <= tan_diff && tan_diff <= -0.01 {
            // Open or close sensors in sync.
            self.sensor_right.facing += dt * SENSOR_TURN_RATE * input.fov_axis;
            self.sensor_left.facing -= dt * SENSOR_TURN_RATE * input.fov_axis;

            let rs_vector = vec_from_angle(self.sensor_right.facing);
            let ls_vector = vec_from_angle(self.sensor_left.facing);
            // let tan_diff = rs_vector.y.atan2(rs_vector.x) - ls_vector.y.atan2(ls_vector.x);
            // println!("tan_diff = {}", tan_diff);
            self.fov_length = rs_vector.y.atan2(rs_vector.x) - ls_vector.y.atan2(ls_vector.x);
        }


        // Here we handle the fire situations.
        if input.fire && self.cooldown < 0.0 {
            self.cooldown = SHOOTING_COOLDOWN;

            let bullet: Bullet = Bullet::new(self.tag, self.pos, self.facing, &self.color);
            bullet_vector.push(bullet);
        }
    }

    /// This is where the A.I. engine works its magic.
    /// This is where we convert the orders from the A.I. engine to a set of commands to a Panda.
    pub fn build_input_from_ai(input: &[f32]) -> InputState {
        // Thruster handler.
        // let yaxis = input[0].abs();// + -input[1];
        // let yaxis = input[0].abs() - (input[1].abs() * 0.25);
        let yaxis = input[0].abs() - input[1].abs();
        // let yaxis = input[0];

        // Turn handler.
        // input[1] == Turn left and input[2] == turn right
        // let xaxis = input[1] - input[2];
        let xaxis = input[2].abs() - input[3].abs();

        // Sensor movement handler.
        // let rs_axis = input[3] - input[4];
        let fov_axis = input[4].abs() - input[5].abs();
        // let fov_axis = input[4].log(10.0) - input[5].log(10.0);
        // let fov_axis = input[4].fract();

        // Handle the firing part based on an arbitrary threshold.
        let fire = {
            // if input[5] > 0.0 {
            if input[6] > 0.0 {
                true
            } else {
                false
            }
        };

        InputState {
            xaxis,
            yaxis,
            fov_axis,
            freeze: 0.0,
            fire,
        }
    }

    /// Build the array we need to feed the A.I. engine with from the state of our Panda and its
    /// Sensors.
    fn build_output(&mut self, left_sensor_output: [f32; SENSOR_OUTPUT_LEN], right_sensor_output: [f32; SENSOR_OUTPUT_LEN]) {
        let mut start_idx: usize = 0;

        if self.cooldown <= 0.0 {
            // Signify that we can shoot.
            self.input_to_ai[start_idx] = 1.0;
        } else {
            self.input_to_ai[start_idx] = 0.0;
        }
        start_idx += 1;

        // FOV handler.
        self.input_to_ai[start_idx] = self.fov_length;
        start_idx += 1;

        for value in left_sensor_output.iter() {
            self.input_to_ai[start_idx] = *value;
            start_idx += 1;
        }

        for value in right_sensor_output.iter() {
            self.input_to_ai[start_idx] = *value;
            start_idx += 1;
        }

        // Print this for debugging purpose.
        // println!("tag \tpos.x \tpos.y \tdVec.x \tdVec.y \tvel.x \tvel.y \
        //          \tLdV.x \tLdV.y \tdist \tBullet \tPanda \tRdV.x \tRdV.y \tdist \tBullet \tPanda");
        // print!("{}", self.tag);
        // for i in self.output.iter() {
        //     print!(" \t{:.2}", i);
        // }
        // println!("");
    }

    /// Takes a Panda and wraps its position to bounds of the gameboard, so if it goes off the left
    /// side of the gameboard it will reappear on the right side and so on.
    fn wrap_position(&mut self, ctx: &mut Context) {
        let height = ctx.conf.window_mode.height as f32;
        let width = ctx.conf.window_mode.width as f32;

        let x_bound = width / 2.0;
        let y_bound = height / 2.0;

        let offset: f32 = self.hitbox_size * 2.0;

        if self.pos.x < 0.0 - offset / 2.0 {
            self.pos.x += x_bound + offset;
        } else if self.pos.x > x_bound + offset / 2.0 {
            self.pos.x -= x_bound + offset;
        }

        if self.pos.y < 0.0 - offset / 2.0 {
            self.pos.y += y_bound + offset;
        } else if self.pos.y > y_bound + offset / 2.0 {
            self.pos.y -= y_bound + offset;
        }
    }


    /// Do not wrap the positions the Panda and confine them in the gamboad instead.
    fn confine_position(&mut self, ctx: &mut Context) {
        let height = ctx.conf.window_mode.height as f32;
        let width = ctx.conf.window_mode.width as f32;

        let x_bound = width / 2.0;
        let y_bound = height / 2.0;

        let offset: f32 = self.hitbox_size * 2.0;

        if self.pos.x < 0.0 - offset / 2.0 {
            self.pos.x = 0.0 + offset;
        } else if self.pos.x > x_bound + offset / 2.0 {
            self.pos.x = x_bound - offset;
        }

        if self.pos.y < 0.0 - offset / 2.0 {
            self.pos.y = 0.0 + offset;
        } else if self.pos.y > y_bound + offset / 2.0 {
            self.pos.y = y_bound - offset;
        }
    }
}
