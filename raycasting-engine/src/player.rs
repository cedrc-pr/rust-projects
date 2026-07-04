use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use chinchilib::{
    pixels::Pixels,
    rgb::{self, RGBA8},
    GfxApp, Key,
};

use crate::{
    deg_to_rad, draw_all, draw_minimap, draw_text, rad_to_deg, send_precise_ray, Coords2D,
    EasingDirection, EasingStyle, Map, Tween, TweenInfo, TweenService,
};

const INPUT_COOLDOWN: f32 = 0.2;

pub struct Player {
    /// Position of the player
    pub pos: Coords2D<f32>,
    /// Player movements speed
    pub speed: f32,
    /// If the player is sprinting
    pub sprint: bool,
    /// If the player put the game in pause
    pub pause: bool,
    /// Cooldown for keys that needed
    pub input_cooldown: f32,
    /// Angle of the player in radiant
    pub angle: f32,
    /// Direction where the player is moving in radiant
    pub direction_angle: f32,
    /// Field of View of the player
    pub fov: f32,
    /// Field of View but it's the tween value
    pub fov_tween: Option<Tween<f32>>,
    /// The representation of the map
    pub map: Map,
    /// The windows size (width, height)
    pub screen_size: (usize, usize),
    /// TCP server that receive commands and apply the given command
    pub command_receiver: Option<Receiver<String>>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pos: Coords2D::default(),
            speed: 2.0,
            sprint: false,
            pause: true,
            input_cooldown: 0.0,
            angle: 0.0,
            direction_angle: 0.0,
            fov: 70.0,
            fov_tween: None,
            map: Map::default(),
            screen_size: (500, 500),
            command_receiver: None,
        }
    }
}

impl Player {
    pub fn new(coord: Coords2D<f32>, map: Map, screen_size: (usize, usize), fov: f32) -> Self {
        let (tx, rx): (Sender<String>, Receiver<String>) = channel();

        thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:4242").expect("Impossible de bind le port");
            println!("SERVER: Listening on 127.0.0.1:4242");

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let tx_clone = tx.clone();
                        thread::spawn(move || {
                            let mut reader = BufReader::new(stream);
                            let mut line = String::new();
                            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                                let cmd = line.trim().to_string();
                                if !cmd.is_empty() {
                                    let _ = tx_clone.send(cmd);
                                }
                                line.clear();
                            }
                        });
                    }
                    Err(e) => println!("Connection failed: {}", e),
                }
            }
        });

        Self {
            pos: coord,
            speed: 4.0,
            sprint: false,
            pause: true,
            input_cooldown: 0.0,
            angle: 0.0,
            direction_angle: 0.0,
            fov,
            fov_tween: None,
            map,
            screen_size,
            command_receiver: Some(rx),
        }
    }

    pub fn move_player(&mut self, angle: f32) {
        if !self.pause {
            self.direction_angle = self.angle + angle;
            let ray_res = send_precise_ray(
                &self.map,
                self.pos.convert_to_tile(&self.map),
                self.direction_angle,
            );
            let can_move = ray_res.map_or(true, |res| res.distance > 0.1);
            if can_move {
                self.pos.move_towards(self.direction_angle, self.speed);
            }
        }
    }

    pub fn pivote_player(&mut self, angle: f32) {
        self.angle += angle;
        if rad_to_deg(self.angle) >= 360.0 {
            self.angle -= deg_to_rad(360.0)
        } else if rad_to_deg(self.angle) <= -360.0 {
            self.angle += deg_to_rad(360.0);
        }
    }

    fn handle_commands(&mut self, needs_redraw: &mut bool) {
        if let Some(rx) = &self.command_receiver {
            while let Ok(command) = rx.try_recv() {
                println!("COMMANDE REÇUE: {}", command);
                let parts: Vec<&str> = command.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                match parts[0] {
                    "FOV" => {
                        if let Ok(val) = parts.get(1).unwrap_or(&"70").parse::<f32>() {
                            self.fov = val;
                            self.fov_tween =
                                Some(TweenService::create(self.fov, val, TweenInfo::default()));
                            *needs_redraw = true;
                        }
                    }
                    "SPEED" => {
                        if let Ok(val) = parts.get(1).unwrap_or(&"2").parse::<f32>() {
                            self.speed = val;
                        }
                    }
                    "TP" => {
                        if parts.len() >= 3 {
                            let x = parts[1].parse::<f32>().unwrap_or(1.0);
                            let y = parts[2].parse::<f32>().unwrap_or(1.0);
                            self.pos.x = (x + 0.5) * self.map.tile_size_in_px as f32;
                            self.pos.y = (y + 0.5) * self.map.tile_size_in_px as f32;
                            *needs_redraw = true;
                        }
                    }
                    "PAUSE" => {
                        self.pause = !self.pause;
                        *needs_redraw = true;
                    }
                    _ => println!("Commande inconnue: {}", parts[0]),
                }
            }
        }
    }

    fn handle_pressed_keys(
        &mut self,
        pressed_keys: &std::collections::HashSet<Key>,
        needs_redraw: &mut bool,
    ) {
        for key in pressed_keys {
            match key {
                Key::KeyQ => {
                    self.move_player(deg_to_rad(-90.0));
                    *needs_redraw = true;
                }
                Key::KeyD => {
                    self.move_player(deg_to_rad(90.0));
                    *needs_redraw = true;
                }
                Key::KeyZ => {
                    self.move_player(deg_to_rad(0.0));
                    *needs_redraw = true;
                }
                Key::KeyS => {
                    self.move_player(deg_to_rad(180.0));
                    *needs_redraw = true;
                }
                Key::KeyE => {
                    self.pivote_player(0.04);
                    *needs_redraw = true;
                }
                Key::KeyA => {
                    self.pivote_player(-0.04);
                    *needs_redraw = true;
                }
                Key::Up => {
                    if self.input_cooldown <= 0.0 {
                        self.sprint = !self.sprint;
                        let target_fov_val = if self.sprint {
                            self.fov - 10.0
                        } else {
                            self.fov + 10.0
                        };
                        self.speed = if self.sprint { 8.0 } else { 4.0 };
                        self.fov_tween = Some(TweenService::create(
                            self.fov,
                            target_fov_val,
                            TweenInfo {
                                time: 0.25,
                                style: EasingStyle::Sine,
                                direction: EasingDirection::Out,
                            },
                        ));
                        self.input_cooldown = INPUT_COOLDOWN;
                    }
                    *needs_redraw = true;
                }
                Key::Down => {
                    if self.input_cooldown <= 0.0 {
                        self.pause = true;
                        self.input_cooldown = INPUT_COOLDOWN;
                        *needs_redraw = true;
                    }
                }
                _ => {}
            }
        }
    }
}

const RED: rgb::RGBA8 = rgb::RGBA8 {
    r: u8::MAX,
    g: 0,
    b: 0,
    a: u8::MAX,
};

impl GfxApp for Player {
    fn on_tick(&mut self, pressed_keys: &std::collections::HashSet<Key>) -> bool {
        let mut needs_redraw = false;
        self.handle_commands(&mut needs_redraw);

        if self.input_cooldown > 0.0 {
            self.input_cooldown -= 1.0 / 60.0;
        }

        if self.pause {
            if !pressed_keys.is_empty()
                && !pressed_keys.contains(&Key::Down)
                && self.input_cooldown <= 0.0
            {
                self.pause = false;
                self.input_cooldown = INPUT_COOLDOWN;
                needs_redraw = true;
            }
            return needs_redraw;
        }

        self.handle_pressed_keys(pressed_keys, &mut needs_redraw);
        if let Some(tween) = &mut self.fov_tween {
            self.fov = tween.update(1.0 / 60.0);
            needs_redraw = true;
            if !tween.active {
                self.fov_tween = None;
            }
        }

        needs_redraw
    }

    fn draw(&mut self, pixels: &mut Pixels, width: usize) {
        self.screen_size.0 = width;
        self.screen_size.1 = pixels.frame().len() / (width * 4);
        if !self.pause {
            draw_all(pixels.frame_mut(), width, self);
            draw_text(
                pixels.frame_mut(),
                width,
                Coords2D::default(),
                format!(
                    "x: {:.0}\ny: {:.0}\nangle: {:.0}\n{:.0}",
                    self.pos.x,
                    self.pos.y,
                    rad_to_deg(self.angle),
                    self.speed,
                )
                .as_ref(),
                RED,
                3,
            );
            draw_minimap(pixels.frame_mut(), width, self);
        } else {
            pixels.frame_mut().fill(0);
            draw_text(
                pixels.frame_mut(),
                width,
                Coords2D {
                    x: (self.screen_size.0 as f32 / 2.0) - 100.0,
                    y: self.screen_size.1 as f32 / 2.0,
                },
                "PRESS ANY KEY\n(EXCEPT DOWN)",
                RGBA8::new(255, 255, 255, 255),
                4,
            );
        }
    }

    fn done(&self) -> chinchilib::DoneStatus {
        chinchilib::DoneStatus::NotDone
    }
}
