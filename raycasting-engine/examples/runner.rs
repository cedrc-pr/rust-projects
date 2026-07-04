use std::{fs, path::PathBuf};

use chinchilib::WinitHandler;
use runner::{Map, Player};

use clap::Parser;

#[derive(clap::Parser)]
struct Cli {
    #[arg(short, long)]
    fov: Option<f32>,
    #[arg(short, long)]
    map: Option<std::path::PathBuf>,
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    log::info!("Hello, world!");

    let screen_size: (usize, usize) = (2200, 1200);
    let map_str = fs::read_to_string(cli.map.unwrap_or_else(|| PathBuf::from("levels/map3.txt")));
    let map: Map = Map::new_from_str(
        map_str
            .unwrap_or(String::from("#######   ##   ##   ##   ##   ######"))
            .as_ref(),
    );
    println!("{map}");
    let fov = cli.fov.unwrap_or_else(|| 70.0);
    let player = Box::new(Player::new(
        map.get_spawn_point_in_pixel(),
        map,
        screen_size,
        fov,
    ));
    let mut app = WinitHandler::new(player, screen_size, 60);
    app.set_always_tick(false);
    app.run().unwrap();
}
