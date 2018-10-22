//! Hold our game state.

use actors::{Body, Bullet, Panda};
use color_picker::*;
use gameboard_controller::InputState;
use ggez::event::{self, Keycode, Mod};
use ggez::{graphics, timer, Context, GameResult};
use rayon::prelude::*;
use fnv::FnvHashMap;

use fluffy_penguin::genetic_algorithm::Specimen;
use fluffy_penguin::genetic_algorithm::Population;

// The desired FPS (or speed so to speak) our world
// will run at. It's destined to be modified by the user
// later on.
const DESIRED_FPS: u32 = 90;
const SAVE_DIR: &str = "tmp/save/";
// Game speed value.
const GAME_SPEED: f32 = DESIRED_FPS as f32 * 2.0;
// The step at which rate we want to modify the speed of the simulation at run time.
const SPEED_STEP: f32 = 5.0;
// This is the final contdown ! tududu du tududududu...
// This is the remaining value before next auto evolution pop.
const COUNTDOWN: usize = 500;
// const COUNTDOWN: usize = 5_000;
// Every this value tick we trigger a structural mutation.
const EXPLORATION_TICK: usize = 50;
// Font size of text that will be printed
// on the screen to inform the user.
const FONT_SIZE: u32 = 12;
// Number of actor per board.
const ACTOR_NUMBER_PER_BOARD: usize = 128;
// Number of bullet maximum on a gameboard.
const BULLET_NUMBER_PER_BOARD: usize = ACTOR_NUMBER_PER_BOARD * 4;
// Number of score point win when a panda shot an other panda.
const POINT_WIN_PER_SUCCESSFUL_SHOT: f32 = 33.0;
// Number of score point lost when shot.
const POINT_LOST_WHEN_SHOT: f32 = 77.0;
// All the color a panda can wear.
const COLOR_ARRAY: [[f32; 4]; 8] = [WHITE, AQUA, RED, GREEN, BLUE, ORANGE, PURPLE, YELLOW];
// Probability for any mutation to apply on each specimen during exploration phase.
// Usually set between 0.05 and 0.1 (5 and 10 %).
// TODO: Make this configurable <08-08-18, dymayday> //
const MUTATION_PROBABILITY: f32 = 0.05;

/// This is our main state data handler.
pub struct State {
    //text: graphics::Text,
    font: graphics::Font,
    panda_vector: Vec<Panda>,
    population: Population<f32>,
    bullet_vector: Vec<Bullet>,
    input: InputState,
    speed: f32,
    generation: usize,
    countdown: usize,
    wrap_world: bool,
    save_dir: String,
}

impl State {
    pub fn new(ctx: &mut Context) -> GameResult<State> {
        use actors::{AI_ENGINE_INPUT_LEN, AI_ENGINE_OUTPUT_LEN};
        // Let's set our background with a nice "blackish" color
        // from the material theme
        graphics::set_background_color(ctx, BLACK.into());

        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.
        //let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf", FONT_SIZE)?;
        let font = graphics::Font::new(ctx, "/FiraSans-Regular.ttf", FONT_SIZE)?;
        //let text = graphics::Text::new(ctx, "Hello world!", &font)?;

        // Create a iterator from which we can cycle through to give our pandas roughfly different
        // colors.
        let mut color_iter_cycle = COLOR_ARRAY.iter().cycle();

        let mut panda_vector: Vec<Panda> = Vec::with_capacity(ACTOR_NUMBER_PER_BOARD);
        for tag in 0..ACTOR_NUMBER_PER_BOARD {
            let panda_color = color_iter_cycle
                .next()
                .expect("Fail to cycle through the available color.");
            let mut panda = Panda::new(ctx, tag as usize, *panda_color);
            panda_vector.push(panda);
        }
        // let panda_vector = State::new_actor_population(ctx)?;

        let population_size: usize = ACTOR_NUMBER_PER_BOARD;
        let input_size: usize = AI_ENGINE_INPUT_LEN;
        let output_size: usize = AI_ENGINE_OUTPUT_LEN;
        let mutation_probability: f32 = MUTATION_PROBABILITY;
        let mut population: Population<f32> = Population::new(
            population_size,
            input_size,
            output_size,
            mutation_probability,
        );
        population
            // .set_lambda((ACTOR_NUMBER_PER_BOARD / 2 ) as usize)
            .set_s_rank(1.5);
        population.exploration();

        let bullet_vector: Vec<Bullet> = Vec::with_capacity(BULLET_NUMBER_PER_BOARD);

        Ok(State {
            font,
            panda_vector,
            population,
            bullet_vector,
            input: InputState::default(),
            speed: GAME_SPEED,
            generation: 0,
            countdown: COUNTDOWN,
            wrap_world: true,
            save_dir: SAVE_DIR.to_string(),
        })
    }


    /// Set the number of actor on the gameboard.
    pub fn with_actor_capacity(mut self, actor_size: usize) -> Self {
        // If there is enought actor it's easy, we just slice them from the original popilation.
        if actor_size <= self.panda_vector.len() {
            self.panda_vector = self.panda_vector[..actor_size].to_vec();
        } else {
            // But if there is not enough Panda to satisfy our appetite, we need to cycle through
            // the ones we have in stock.
            let mut new_panda_vector: Vec<Panda> = Vec::with_capacity(actor_size);
            let mut panda_iter_cycle = self.panda_vector.into_iter().cycle();
            for _ in 0..actor_size {
                let panda = panda_iter_cycle
                    .next()
                    .expect("Fail to cycle through the vector of Panda.");

                new_panda_vector.push(panda.to_owned());
            }
            self.panda_vector = new_panda_vector;
        }
        self
    }


    /// Determines if the position of each Panda will be wrap in a toric world, or if they will be
    /// stuck on the imaginary walls of the arena.
    /// True by default.
    pub fn wrap_world(mut self, b: bool) -> Self {
        self.wrap_world = b;
        self
    }


    /// Update the default game save directory.
    pub fn set_save_directory(mut self, save_dir: &str) -> Self {
        self.save_dir = save_dir.to_string();
        self
    }


    /// Reset the population of Panda on the gameboard.
    fn new_actor_population(ctx: &mut Context, actor_size: usize) -> GameResult<Vec<Panda>> {
        // Create a iterator from which we can cycle through to give our pandas roughfly different
        // colors.
        let mut color_iter_cycle = COLOR_ARRAY.iter().cycle();

        let mut panda_vector: Vec<Panda> = Vec::with_capacity(actor_size);
        for tag in 0..actor_size {
            let panda_color = color_iter_cycle
                .next()
                .expect("Fail to cycle through the available color.");
            let mut panda = Panda::new(ctx, tag as usize, *panda_color);
            panda_vector.push(panda);
        }
        Ok(panda_vector)
    }

    /// This is where the collision between the pandas and the bullets are handled.
    fn handle_collisions(&mut self) -> GameResult<()> {
        // This HashMap let us update the score of panda that successfully shoot someone.
        let mut successfull_panda_shot_hashmap: FnvHashMap<usize, f32> =
            FnvHashMap::with_capacity_and_hasher(self.panda_vector.len(), Default::default());

        for mut panda in &mut self.panda_vector {
            for mut bullet in &mut self.bullet_vector {
                if panda.tag != bullet.tag && panda.body.in_contact(&bullet.body) {
                    panda.score -= POINT_LOST_WHEN_SHOT;
                    bullet.to_remove = true;

                    let score = successfull_panda_shot_hashmap
                        .entry(bullet.tag)
                        .or_insert(0.0);
                    *score += POINT_WIN_PER_SUCCESSFUL_SHOT;
                }
            }
        }

        // Here we update the score of each Panda whose bullet hit a target.
        for (tag, score) in &successfull_panda_shot_hashmap {
            self.panda_vector[*tag].score += score;
        }

        Ok(())
    }

    /// Print FPS to screen
    fn draw_fps(&mut self, ctx: &mut Context) -> GameResult<()> {
        let fps_string = format!("{:.1} fps", timer::get_fps(ctx));
        let gfps = graphics::Text::new(ctx, &fps_string, &self.font)?;

        // Drawables are drawn from their top-left corner.
        let dest_point = graphics::Point2::new(0.0, 0.0);

        graphics::set_color(ctx, WHITE.into())?;
        graphics::draw(ctx, &gfps, dest_point, 0.0)?;

        Ok(())
    }

    /// Print the score on the screen.
    fn draw_scores(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut y_pos = FONT_SIZE as f32 * 2.0;

        // Print the countdown before next evolution.
        let countdown = graphics::Text::new(ctx, &format!("Countdown : {:>6}", self.countdown), &self.font)?;
        let dest_point = graphics::Point2::new(10.0, y_pos);
        graphics::set_color(ctx, WHITE.into())?;
        graphics::draw(ctx, &countdown, dest_point, 0.0)?;
        y_pos += FONT_SIZE as f32 + 2.0;

        // Print the generation number we are currently at.
        let generation = graphics::Text::new(ctx, &format!("Generation : {:>4}", self.generation), &self.font)?;
        let dest_point = graphics::Point2::new(10.0, y_pos);
        graphics::set_color(ctx, WHITE.into())?;
        graphics::draw(ctx, &generation, dest_point, 0.0)?;
        y_pos += FONT_SIZE as f32 + 2.0;

        // Print details about what this column of number actually means.
        let gscore_header = graphics::Text::new(ctx, "Scores :", &self.font)?;
        // Drawables are drawn from their top-left corner.
        let dest_point = graphics::Point2::new(10.0, y_pos);
        graphics::set_color(ctx, WHITE.into())?;
        graphics::draw(ctx, &gscore_header, dest_point, 0.0)?;

        y_pos += FONT_SIZE as f32 + 2.0;
        for panda in &self.panda_vector {
            let score_string = format!("{:3} : {:4}", panda.tag + 1, panda.score);
            let gscore = graphics::Text::new(ctx, &score_string, &self.font)?;

            // Drawables are drawn from their top-left corner.
            let dest_point = graphics::Point2::new(10.0, panda.tag as f32 + y_pos);
            graphics::set_color(ctx, panda.color.into())?;
            graphics::draw(ctx, &gscore, dest_point, 0.0)?;

            y_pos += FONT_SIZE as f32;
        }
        Ok(())
    }

    /// Draw the gameboard grid.
    fn _draw_grid(&mut self, ctx: &mut Context) -> GameResult<()> {
        let board_graticul_size: f32 = 10.0;
        let height: f32 = ctx.conf.window_mode.height as f32;
        let width: f32 = ctx.conf.window_mode.width as f32;

        let x_line_number = (width / board_graticul_size) as usize;
        let y_line_number = (height / board_graticul_size) as usize;

        let mb = &mut graphics::MeshBuilder::new();

        for i in 0..x_line_number {
            let ix = (i as f32) * board_graticul_size;
            let start_point = graphics::Point2::new(ix, 0.0);
            let end_point = graphics::Point2::new(ix, height);
            if i % 10 != 0 {
                mb.line(&[start_point, end_point], 1.0);
            } else {
                mb.line(&[start_point, end_point], 1.5);
            }
        }
        for i in 0..y_line_number {
            let iy = (i as f32) * board_graticul_size;
            let start_point = graphics::Point2::new(0.0, iy);
            let end_point = graphics::Point2::new(width, iy);
            if i % 10 != 0 {
                mb.line(&[start_point, end_point], 1.0);
            } else {
                mb.line(&[start_point, end_point], 1.5);
            }
        }

        let drawparam = graphics::DrawParam {
            color: Some(BLACK2.into()),
            ..graphics::DrawParam::default()
        };

        let mbb = mb.build(ctx).expect("Fail to draw the gameboardd gridd.");
        graphics::draw_ex(ctx, &mbb, drawparam)?;

        Ok(())
    }


    /// Update the population from the A.I. engine.
    fn evolve(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Let's keep track of how far we can get.
        self.generation += 1;

        // Update the fitness value of each Specimen with the score of its associated Panda.
        for (mut panda, mut specimen) in &mut self.panda_vector.iter().zip(&mut self.population.species) {
            specimen.fitness = panda.score;
        }

        // if self.generation % EXPLORATION_TICK == 0 {
        //     let mut pop_sorted = self.population.clone();
        //     pop_sorted.sort_species_by_fitness();
        //
        //     info!("Rendering Specimens...");
        //     pop_sorted.render("tmp/sorted_vizualisation/", false, false);
        //     self.save_to_file();
        // }

        // Evolve the population by mating them together.
        self.population.evolve();

        // self.population.render(&format!("tmp/vizualisation/gen_{:0>3}/", self.generation), false, false);

        if self.generation % EXPLORATION_TICK == 0 {
            info!("Generation {:>3} : Structural Exploration.", self.generation);
            self.population.exploration();

            info!("Rendering Specimens...");
            self.population.render("tmp/vizualisation/", false, false);
            self.save_to_file();
        } else {
            info!("Generation {:>3} : Parametric Exploitation.", self.generation);
            self.population.exploitation();
        }

        self.reset_board(ctx)?;
        Ok(())
    }


    /// Save the Panda's brains to file.
    fn save_to_file(&self) {
        use chrono::prelude::*;

        let date = Local::now().format("%FT%Hh%Mm%Ss");
        let file_name = format!("{}/{}_Population-gen{:03}.bc", SAVE_DIR, date, self.generation);
        info!("Saving Population to '{}'.", file_name);
        self.population.save_to_file(&file_name);
    }


    /// Load the Panda's brains from file.
    fn load_population_from_file(&mut self, file_name: &str) -> Result<(), ()> {
        match fluffy_penguin::genetic_algorithm::Population::load_from_file(file_name) {
            Ok(population) => {
                self.generation = population.generation_counter;
                self.population = population;
                return Ok(())
            },
            Err(e) => {
                crit!("{}", &format!("{:?}", e));
                return Err(())
            }
        }
    }


    /// Load the Panda's brains from the last previous save file.
    pub fn reload_population_from_last_saved_game(&mut self) {
        use glob::glob;

        let wild_card = &format!("{}/*.bc", SAVE_DIR);
        let mut fpl: Vec<String> = glob(wild_card).expect("Failed to read glob pattern")
            .filter_map(|p| Some(p.unwrap().to_str().unwrap().to_string()))
            .collect::<Vec<String>>();
        fpl.sort();
        debug!("fpl = {:#?}", fpl);

        let file_name = fpl.last().unwrap().to_owned();
        match self.load_population_from_file(&file_name) {
            Ok(_) => info!("Loading Game from '{}'.", file_name),
            Err(_) => warn!("Fail to load the game from '{}'.", file_name),
        };
    }


    /// Wipe clean the entire gameboard.
    fn reset_board(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.panda_vector = State::new_actor_population(ctx, self.panda_vector.len())?;

        // Clean all the bullets as well.
        self.bullet_vector.clear();

        self.countdown = COUNTDOWN;

        Ok(())
    }
}

impl event::EventHandler for State {
    /// We need to implement at least two functions in order to use properly ggez: update & draw

    /// This is the update one.
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // This function will return true if the time since the last update() call has been
        // equal to or greater to the update FPS indicated by the DESIRED_FPS.
        while timer::check_update_time(ctx, DESIRED_FPS) {
            let dt: f32 = 1.0 / self.speed;

            // Here we clean the gameboard from all unnecessary bullet.
            let mut bullet_to_keep_vector: Vec<Bullet> =
                Vec::with_capacity(self.bullet_vector.len());

            for mut bullet in &mut self.bullet_vector {
                if !bullet.to_remove {
                    bullet.update(ctx, dt)?;
                    bullet_to_keep_vector.push(bullet.to_owned());
                }
            }
            self.bullet_vector = bullet_to_keep_vector;
            {
                // Here we build a vector containing all the object each panda can interact with: the
                // other pandas and the bullets.
                let cap: usize = self.panda_vector.len() + self.bullet_vector.len();
                let mut body_vector: Vec<Body> = Vec::with_capacity(cap);
                for panda in &self.panda_vector {
                    body_vector.push(Body::new(panda.tag, false, &panda.nshape, &panda.isometry));
                }

                for bullet in &self.bullet_vector {
                    body_vector.push(Body::new(bullet.tag, true, &bullet.nshape, &bullet.iso));
                }

                // Let's update all the pandas.
                for i in 0..self.panda_vector.len() {
                    let mut panda: &mut Panda = &mut self.panda_vector[i];
                    let mut specimen: &mut Specimen<f32> = &mut self.population.species[i];

                    // We manually update the input values we feed to the ANN.
                    specimen.update_input(&panda.input_to_ai);
                }


                // Here we evaluate each specimen in parallel.
                let mut input_state_v: Vec<InputState> = Vec::with_capacity(self.panda_vector.len());
                self.population.species.par_iter_mut()
                    .map(|specimen| {
                        // Input commands computed by the ANN from the A.I. engine.
                        Panda::build_input_from_ai(&specimen.evaluate())
                    }).collect_into_vec(&mut input_state_v);

                // // Un-parallelized version.
                // let mut input_state_v: Vec<InputState> = self.population.species.iter_mut()
                //     .map(|specimen| {
                //         // Input commands computed by the ANN from the A.I. engine.
                //         Panda::build_input_from_ai(&specimen.evaluate())
                //     }).collect();


                // Let's update all the pandas.
                for i in 0..self.panda_vector.len() {
                    let mut panda: &mut Panda = &mut self.panda_vector[i];

                    // Input commands computed by the ANN from the A.I. engine.
                    panda.handle_input(&input_state_v[i], &mut self.bullet_vector, dt);
                    panda.update(ctx, &body_vector, self.wrap_world, dt)?;
                }

                // self.panda_vector[0].handle_input(&self.input, &mut self.bullet_vector, dt);
                // self.panda_vector[0].update(ctx, &body_vector, dt)?;


            }

            {
                self.handle_collisions()?;
            }
        }

        // Run the countdown before next auto evolution triggers.
        if self.countdown <= 0 {
            self.evolve(ctx)?;
            self.countdown = COUNTDOWN;
        } else {
            self.countdown -= 1;
        }

        Ok(())
    }

    /// This is where our world is drawn from the abysses.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        // Draw the gameboard grid. Maybe there is a lazy implementation of the draw call but I
        // haven't found it yet.
        // self._draw_grid(ctx)?; // TODO: Uncomment this when we are in release mode.

        for mut panda in &mut self.panda_vector {
            panda.draw(ctx)?;
        }

        for mut bullet in &mut self.bullet_vector {
            bullet.draw(ctx)?;
        }

        self.draw_fps(ctx)?;
        self.draw_scores(ctx)?;


        // Finally we call graphics::present to cycle the gpu's framebuffer and display
        // the new frame we just drew.
        graphics::present(ctx);

        // And yield the timeslice
        // This tells the OS that we're done using the CPU but it should
        // get back to this program as soon as it can.
        // This ideally prevents the game from using 100% CPU all the time
        // even if vsync is off.
        // The actual behavior can be a little platform-specific.
        timer::yield_now();
        Ok(())
    }

    /// Handle key events. These just map keyboard events
    /// and alter our input state appropriately.
    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Up => {
                self.input.yaxis = 1.0;
            }
            Keycode::Down => {
                self.input.yaxis = -1.0;
            }
            Keycode::Left => {
                self.input.xaxis = 1.0;
            }
            Keycode::Right => {
                self.input.xaxis = -1.0;
            }
            Keycode::C => {
                // Close the FOV.
                self.input.fov_axis = 1.0;
            }
            Keycode::V => {
                // Open the FOV.
                self.input.fov_axis = -1.0;
            }
            Keycode::L => {
                self.reload_population_from_last_saved_game();
                self.reset_board(ctx)
                    .expect("Fail to reset the Gameboard after loading from last game's save.");
            }
            Keycode::Space => {
                self.input.fire = true;
            }
            Keycode::B => {
                self.input.freeze = 1.0;
            }
            Keycode::P => {
                let img = graphics::screenshot(ctx).expect("Could not take screenshot");
                img.encode(ctx, graphics::ImageFormat::Png, "/screenshot.png")
                    .expect("Could not save screenshot");
            }
            Keycode::E => {
                println!("Evolving...");
                self.evolve(ctx).expect("Fail to evolve A.I. population.");
                if self.generation % 5 == 0 {
                    println!("Structural Exploration.");
                    self.population.exploration();
                } else {
                    println!("Parametric Exploitation.");
                    self.population.exploitation();
                }
            }
            Keycode::R => {
                println!("Rendering Specimens...");
                self.population.render("tmp/", false, false);
            }
            Keycode::S => {
                self.save_to_file();
            }
            Keycode::W => {
                println!("Parametric Exploitation.");
                self.population.exploitation();
            }
            Keycode::PageDown => {
                self.speed += SPEED_STEP;
                println!("FPS = {}", self.speed);
            }
            Keycode::PageUp => {
                if self.speed > SPEED_STEP {
                    self.speed -= SPEED_STEP;
                    println!("FPS = {}", self.speed);
                }
            }
            Keycode::N => {
                self.speed = GAME_SPEED;
                println!("Reset game speed to {}.", self.speed);
            }
            Keycode::Escape => ctx.quit().unwrap(),
            _ => (), // Do nothing
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Up | Keycode::Down => {
                self.input.yaxis = 0.0;
            }
            Keycode::Left | Keycode::Right => {
                self.input.xaxis = 0.0;
            }
            Keycode::C | Keycode::V => {
                self.input.fov_axis = 0.0;
            }
            Keycode::Space => {
                self.input.fire = false;
            }
            Keycode::B => {
                self.input.freeze = 0.0;
            }
            _ => (), // Do nothing
        }
    }
}
