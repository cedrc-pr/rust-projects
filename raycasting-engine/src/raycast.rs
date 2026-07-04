use chinchilib::{put_pixel, rgb::RGBA8};

use crate::{Coords2D, Map};

#[derive(Debug)]
pub enum RaycastError {
    OutsideOfMap,
    NoTargetFound,
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct RaycastResult {
    /// The coord of the collision in map scale
    pub collide_point: Coords2D<f32>,
    /// The coord of the collision in pixel scale
    pub collide_point_in_px: Coords2D<f32>,
    /// Distance between the collision point and the origin point in map scale
    pub distance: f32,
}

#[derive(Default, Debug)]
pub enum CardinalDirection {
    #[default]
    North,
    South,
    East,
    West,
}

impl CardinalDirection {
    pub fn is_north_east(&self) -> bool {
        matches!(self, Self::North | Self::East)
    }
}

#[derive(Default, Debug)]
pub struct PreciseRaycastResult {
    /// Distance between the collision point and the origin point in pixel scale
    pub distance: f32,
    /// The coord of the collision in map scale
    pub collide_tile: Coords2D<usize>,
    /// The coord of the collision in pixel scale
    pub collide_point_in_px: Coords2D<f32>,
    /// The direction the wall hit is facing
    pub wall_facing_dir: CardinalDirection,
    /// x hit -> false & y hit -> true
    pub side: bool,
}

struct PreciseRayCast {
    map: Coords2D<i32>,
    ray_dir: Coords2D<f32>,
    delta_dist: Coords2D<f32>,
    step: Coords2D<i32>,
    side_dist: Coords2D<f32>,
    hit: bool,
    /// x hit -> false & y hit -> true
    side: bool,
    steps: usize,
    max_steps: usize,
}

impl PreciseRayCast {
    fn init(map: &Map, start_pos: Coords2D<f32>, ray_angle: f32) -> Self {
        let map_x = start_pos.x.floor() as i32;
        let map_y = start_pos.y.floor() as i32;
        let ray_dir_x = ray_angle.cos();
        let ray_dir_y = -ray_angle.sin();
        let delta_dist_x = (1.0 / ray_dir_x).abs();
        let delta_dist_y = (1.0 / ray_dir_y).abs();
        let step_x: i32;
        let step_y: i32;
        let side_dist_x: f32;
        let side_dist_y: f32;
        if ray_dir_x < 0.0 {
            step_x = -1;
            side_dist_x = (start_pos.x - map_x as f32) * delta_dist_x;
        } else {
            step_x = 1;
            side_dist_x = (map_x as f32 + 1.0 - start_pos.x) * delta_dist_x;
        }
        if ray_dir_y < 0.0 {
            step_y = -1;
            side_dist_y = (start_pos.y - map_y as f32) * delta_dist_y;
        } else {
            step_y = 1;
            side_dist_y = (map_y as f32 + 1.0 - start_pos.y) * delta_dist_y;
        }
        PreciseRayCast {
            map: Coords2D { x: map_x, y: map_y },
            ray_dir: Coords2D {
                x: ray_dir_x,
                y: ray_dir_y,
            },
            delta_dist: Coords2D {
                x: delta_dist_x,
                y: delta_dist_y,
            },
            step: Coords2D {
                x: step_x,
                y: step_y,
            },
            side_dist: Coords2D {
                x: side_dist_x,
                y: side_dist_y,
            },
            hit: false,
            side: false,
            steps: 0,
            max_steps: map.width_in_tiles.max(map.height_in_tiles) * 2,
        }
    }

    fn run_raycast(&mut self, map: &Map) -> Result<(), RaycastError> {
        while !self.hit && self.steps < self.max_steps {
            if self.side_dist.x < self.side_dist.y {
                self.side_dist.x += self.delta_dist.x;
                self.map.x += self.step.x;
                self.side = false;
            } else {
                self.side_dist.y += self.delta_dist.y;
                self.map.y += self.step.y;
                self.side = true;
            }
            if self.map.x < 0
                || self.map.x >= map.width_in_tiles as i32
                || self.map.y < 0
                || self.map.y >= map.height_in_tiles as i32
            {
                return Err(RaycastError::OutsideOfMap);
            }
            if map.tiles[(self.map.y as usize) * map.width_in_tiles + (self.map.x as usize)] {
                self.hit = true;
            }
            self.steps += 1;
        }
        Ok(())
    }

    fn handle_hit(&self, start_pos: Coords2D<f32>, map: &Map) -> PreciseRaycastResult {
        let dist = if self.side {
            self.side_dist.y - self.delta_dist.y
        } else {
            self.side_dist.x - self.delta_dist.x
        };

        let hit_x = start_pos.x + dist * self.ray_dir.x;
        let hit_y = start_pos.y + dist * self.ray_dir.y;
        let hit_x_px = hit_x * map.tile_size_in_px as f32;
        let hit_y_px = hit_y * map.tile_size_in_px as f32;

        let wall_facing_dir = if !self.side {
            if self.step.x == 1 {
                CardinalDirection::West
            } else {
                CardinalDirection::East
            }
        } else if self.step.y == 1 {
            CardinalDirection::North
        } else {
            CardinalDirection::South
        };

        PreciseRaycastResult {
            side: self.side,
            distance: dist,
            wall_facing_dir,
            collide_tile: Coords2D {
                x: self.map.x as usize,
                y: self.map.y as usize,
            },
            collide_point_in_px: Coords2D {
                x: hit_x_px,
                y: hit_y_px,
            },
        }
    }
}

// Algorithm: https://en.wikipedia.org/wiki/Digital_differential_analyzer_(graphics_algorithm)
pub fn send_precise_ray(
    map: &Map,
    start_pos: Coords2D<f32>,
    ray_angle: f32,
) -> Result<PreciseRaycastResult, RaycastError> {
    let mut rc = PreciseRayCast::init(map, start_pos, ray_angle);
    rc.run_raycast(map)?;
    if rc.hit {
        return Ok(rc.handle_hit(start_pos, map));
    }

    Err(RaycastError::NoTargetFound)
}

pub fn draw_impact(
    map: &Map,
    buffer: &mut [u8],
    width: usize,
    start_pos: Coords2D<f32>,
    angle: f32,
    color: RGBA8,
) {
    let resize_pos: Coords2D<f32> = Coords2D {
        x: start_pos.x / map.tile_size_in_px as f32, // (start_pos.x.round() as usize / map.tile_size_in_px) as f32
        y: start_pos.y / map.tile_size_in_px as f32,
    };
    let ray_res = send_ray(map, resize_pos, angle);
    if let Ok(res) = ray_res {
        let impact_x: usize =
            (res.collide_point_in_px.x * map.tile_size_in_px as f32).floor() as usize;
        let impact_y: usize =
            (res.collide_point_in_px.y * map.tile_size_in_px as f32).floor() as usize;
        if impact_x < width && impact_y < (buffer.len() / 4) / width {
            // Divide by 4 because buffer as a size of 4 octet
            put_pixel(buffer, width, impact_x, impact_y, color);
        }
    }
}

/// deprecated
pub fn send_ray(
    map: &Map,
    start_point: Coords2D<f32>,
    angle: f32,
) -> Result<RaycastResult, RaycastError> {
    let mut distance: f32 = 0.0;
    let step = 0.1;
    while distance <= (map.width_in_tiles.max(map.height_in_tiles)) as f32 {
        let x_px = start_point.x + distance * angle.cos();
        let y_px = start_point.y - distance * angle.sin(); // Y is reverse
        let x = x_px.floor() as usize;
        let y = y_px.floor() as usize;
        if x > map.width_in_tiles || y >= map.height_in_tiles {
            return Err(RaycastError::OutsideOfMap);
        }
        if map.tiles[y * map.width_in_tiles + x] {
            return Ok(RaycastResult {
                collide_point: Coords2D {
                    x: x as f32,
                    y: y as f32,
                },
                collide_point_in_px: Coords2D { x: x_px, y: y_px },
                distance,
            });
        } else {
            distance += step;
        }
    }
    Err(RaycastError::NoTargetFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords2_d::deg_to_rad;

    // Util function to compare floats with merge
    fn approx_eq(a: f32, b: f32, margin: f32) -> bool {
        (a - b).abs() < margin
    }

    #[test]
    fn test_send_ray() {
        let map = Map::default();
        let start_point: Coords2D<f32> = Coords2D { x: 5.0, y: 2.0 };
        let res = send_ray(&map, start_point, deg_to_rad(270.0)).unwrap();
        let expected_x = 5.0;
        let expected_y = 4.0;
        let expected_dist = 2.0;
        assert!(
            approx_eq(res.collide_point.x, expected_x, 1e-5),
            "X incorrect"
        );
        assert!(
            approx_eq(res.collide_point.y, expected_y, 1e-5),
            "Y incorrect"
        );
        assert!(
            approx_eq(res.distance, expected_dist, 0.15),
            "Distance incorrecte: obtenu {}, attendu env. {}",
            res.distance,
            expected_dist
        );
    }

    #[test]
    fn test2_send_ray() {
        let map = Map::default();
        let start_point: Coords2D<f32> = Coords2D { x: 5.0, y: 1.0 };
        let res = send_ray(&map, start_point, deg_to_rad(0.0)).unwrap();
        let expected_x = 9.0;
        let expected_y = 1.0;
        let expected_dist = 4.0;
        assert!(
            approx_eq(res.collide_point.x, expected_x, 1e-5),
            "X incorrect"
        );
        assert!(
            approx_eq(res.collide_point.y, expected_y, 1e-5),
            "Y incorrect"
        );
        assert!(
            approx_eq(res.distance, expected_dist, 0.15),
            "Distance incorrecte: obtenu {}, attendu env. {}",
            res.distance,
            expected_dist
        );
    }
}
