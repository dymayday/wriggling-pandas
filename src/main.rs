extern crate ggez;
extern crate nalgebra as na;
extern crate ncollide2d;
extern crate rand;
extern crate rayon;
extern crate fnv;
extern crate glob;
extern crate chrono;
#[macro_use(
    slog_o,
    slog_info,
    slog_debug,
    slog_warn,
    slog_crit,
    slog_log,
    slog_record,
    slog_record_static,
    slog_b,
    slog_kv
)]
extern crate slog;
extern crate slog_async;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;

extern crate fluffy_penguin;

mod actors;
mod color_picker;
mod gameboard;
mod gameboard_controller;

use slog::Drain;
use gameboard::State;

fn print_instructions() {
    println!();
    println!("{:*<70}", "");
    println!("* {: ^66} *", "Welcome !");
    println!("* {: <66} *", "How to play:");
    // println!(
    //     "* {: <66} *",
    //     "Left/Right arrow keys rotate your ship, Up/Down thrusts,"
    // );
    // println!(
    //     "* {: <66} *",
    //     "C/V keys to move the left sensor, F/G for the right one,"
    // );
    // println!(
    //     "* {: <66} *",
    //     "Space bar fires and B 'freeze all motor functions',"
    // );
    println!(
        "* {: <66} *",
        "PageUp/PageDown to modify the simulation's speed."
    );
    println!("* {: <66} *", "N to reset the game's speed.");
    println!("* {: <66} *", "E to trigger the evolution process of the A.I. engine");
    println!("* {: <66} *", "W to manually trigger a parametric mutation");
    println!(
        "* {: <66} *",
        " (this only apply on the weights of connection for each ANN)."
    );
    println!("* {: <66} *", "S to manually trigger a structural mutation");
    println!("* {: <66} *", " (mutate the structures of the specimens).");
    println!(
        "* {: <66} *",
        "R to render the ANN of each specimens in dot/svg files"
    );
    println!(
        "* {: <66} *",
        " (default path is '/tmp/', this will be configurable later)"
    );
    println!("{:*<70}", "");
    println!();
}


fn custom_timestamp_local(io: &mut ::std::io::Write) -> ::std::io::Result<()> {
    write!(io, "{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))
}

fn init_log() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator)
        .use_custom_timestamp(custom_timestamp_local)
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(drain, slog_o!())
}


fn main() {
    let c = {
        let mut f = ::std::fs::File::open("resources/conf.toml")
            .expect("conf.toml not found.");
        ggez::conf::Conf::from_toml_file(&mut f)
            .expect("Failed to load Conf from toml file.")
    };
    let ctx =
        &mut ggez::Context::load_from_conf("ggez-generative-art", "awesome_person", c)
            .expect("Failed to buil Context.");

    // We add the CARGO_MANIFEST_DIR/resources to the filesystems paths so
    // we we look in the cargo project for files.
    // Using a ContextBuilder is nice for this because it means that
    // it will look for a conf.toml or icon file or such in
    // this directory when the Context is created.
    if let Ok(manifest_dir) = ::std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = ::std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
        //cb = cb.add_resource_path(path);
        //println!("{:?}", cb);
        //ctx.filesystem.read_config().expect("Fail to read config");
    }

    let state = &mut State::new(ctx)
        .expect("Fail to instantiate the game state.")
        .with_actor_capacity(64)
        .wrap_world(false);

    println!("{:#?}", ctx.conf);
    print_instructions();

    let _guard = slog_scope::set_global_logger(init_log());

    match ggez::event::run(ctx, state) {
        Err(e) => println!("Error encountered during game: {}", e),
        Ok(_) => println!("Game exited cleanly!"),
    }
}
