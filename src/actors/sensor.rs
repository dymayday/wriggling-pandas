//! Represents a sensor (like a field of vision) using ncollide2d and ray casting.

use super::*;

#[derive(Debug, Clone)]
pub struct Sensor {
    // The tag of the panda owning this sensor.
    tag: usize,
    // The distance from the origin of the ray.
    // NOTHINGNESS means seeing nothing.
    pub distance: f32,
    // Is it an actor we are seeing right now ?
    pub is_panda: f32,
    // Or is it a bullet ?
    pub is_bullet: f32,
    // The ray casting technology that "sees".
    pub ray: ncollide2d::query::Ray<f32>,
    // The position as ggez::Point2.
    pos: Point2,
    // The position from where the ray is casted.
    pub na_pos: na::Point2<f32>,
    // The direction where the ray is fired.
    pub facing: f32,
    // Facing representation as a Vector2.
    direction_vector: Vector2,
    // The color of the panda owning this sensor.
    pub color: [f32; 4],
    // This array is use to feed the A.I. engine to inform it about what a Panda 'sense'.
    pub output: [f32; SENSOR_OUTPUT_LEN],
}

impl Sensor {
    /// Returns a new sensor init from a position and an angle.
    pub fn new(tag: usize, na_pos: na::Point2<f32>, angle: f32, color: &[f32; 4]) -> Self {
        Sensor {
            tag,
            distance: -NOTHINGNESS,
            is_panda: 0.0,
            is_bullet: 0.0,
            ray: ncollide2d::query::Ray::new(na_pos, na_vec_from_angle(angle)),
            pos: Point2::new(na_pos.x, na_pos.y),
            na_pos,
            facing: angle,
            direction_vector: vec_from_angle(angle),
            color: *color,
            output: [0.0; SENSOR_OUTPUT_LEN],
        }
    }

    /// Updates the state of the sensor (position, facing angle, etc.)
    pub fn update(&mut self, na_pos: na::Point2<f32>, body_vec: &[Body]) -> GameResult<()> {
        self.na_pos = na_pos;
        self.pos.x = na_pos.coords[0];
        self.pos.y = na_pos.coords[1];
        self.ray = ncollide2d::query::Ray::new(na_pos, na_vec_from_angle(self.facing));

        self.direction_vector.x = self.ray.dir.data[0];
        self.direction_vector.y = self.ray.dir.data[1];

        self.sens(body_vec);
        self.build_output();

        Ok(())
    }

    /// Draws the graphical representation of a sensor as a strait line symbolizing a ray cast.
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mb = self.build_ray_mesh(ctx)?;
        let drawparam = graphics::DrawParam {
            dest: self.pos,
            rotation: -self.facing as f32,
            offset: self.pos,
            color: Some(self.color.into()),
            // Draws the sensors in a slightly more black color.
            // color: Some(BLACK2.into()),
            ..graphics::DrawParam::default()
        };

        graphics::draw_ex(ctx, &mb, drawparam)?;

        Ok(())
    }

    /// This is the graphical representation of the sensor using a MeshBuilder to easily rotate
    /// etc.
    fn build_ray_mesh(&self, ctx: &mut Context) -> GameResult<graphics::Mesh> {
        let mb = &mut graphics::MeshBuilder::new();
        mb.line(
            &[
                self.pos,
                Point2::new(
                    self.na_pos.x,
                    // Draws a shorter version of the sensor for a cleaner gameboard.
                    self.na_pos.y + SENSOR_MAX_DIST / 10.0,
                    // Draws the actual size of the sensor.
                    // self.na_pos.y + (SENSOR_MAX_DIST * 2.0),
                ),
            ],
            BULLET_RADIUS * 2.0, // The thickness of the Sensor's body
        );
        // mb.line(
        //     &[
        //         self.pos,
        //         Point2::new(
        //             self.na_pos.x,
        //             // Draws a shorter version of the sensor for a cleaner gameboard.
        //             // self.na_pos.y + SENSOR_MAX_DIST / 10.0,
        //             // Draws the actual size of the sensor.
        //             self.na_pos.y + (SENSOR_MAX_DIST * 2.0),
        //         ),
        //     ],
        //     0.25, // The thickness of the Sensor's body
        // );

        mb.build(ctx)
    }

    /// Returns the distance of an actor if the sensor 'sees' it, or nothing otherwise.
    pub fn sens(&mut self, body_vec: &[Body]) {
        self.distance = NOTHINGNESS;
        self.is_panda = 0.0;
        self.is_bullet = 0.0;
        // Here we iter through all body we can possibly interact with and get the distance from it
        // if the ray casting encounter it.
        for body in body_vec.iter() {
            // We need to filter the objects belonging to a panda to not interact with them.
            if body.tag != self.tag {
                let dist = self.get_distance(&body);
                if dist <= SENSOR_MAX_DIST {
                    if dist <= self.distance && body.is_bullet {
                        self.distance = dist;
                        self.is_bullet = 10.0;
                        self.is_panda = 0.0;
                    } else if dist <= self.distance && !body.is_bullet {
                        self.distance = dist;
                        self.is_bullet = 0.0;
                        self.is_panda = 1.0;
                    }
                }
            }
        }
        // This is a tweak to help the ANN to better precess distances.
        if self.distance == NOTHINGNESS {
            self.distance *= -1.0;
        } else {
            // Here we change the distance value of the object we 'see' to be inversely
            // proportionnate to its actual value so a closer object will have a higher
            // positive value than a distant one.
            self.distance = SENSOR_MAX_DIST - self.distance;
        }
    }

    /// Returns the distance from the object.
    fn get_distance(&self, body: &Body) -> f32 {
        match body.nshape.toi_with_ray(&body.isometry, &self.ray, true) {
            Some(toi_with_ray) => toi_with_ray,
            _ => NOTHINGNESS,
        }
    }

    /// Build the output of the Sensor that correspond to its state. It's use to feed the A.I.
    /// engine.
    fn build_output(&mut self) {
        let mut start_idx: usize = 0;

        // Indications about what we are looking at.
        self.output[start_idx] = self.is_bullet;
        start_idx += 1;
        self.output[start_idx] = self.is_panda;
        start_idx += 1;

        // Information on the whereabouts of the closest object we are looking at.
        self.output[start_idx] = self.distance;
    }
}

