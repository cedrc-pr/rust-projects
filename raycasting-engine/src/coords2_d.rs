use std::{convert::From, f32::consts::PI};

use crate::Map;

#[derive(Debug, Default, Copy, Clone)]
#[allow(dead_code)]
pub struct Coords2D<T> {
    /// Coord in x
    pub x: T,
    /// Coord in y
    pub y: T,
}

impl From<Coords2D<i32>> for Coords2D<f32> {
    fn from(value: Coords2D<i32>) -> Self {
        Coords2D {
            x: value.x as f32,
            y: value.y as f32,
        }
    }
}

impl From<Coords2D<f32>> for Coords2D<i32> {
    fn from(value: Coords2D<f32>) -> Self {
        Coords2D {
            x: value.x as i32,
            y: value.y as i32,
        }
    }
}

impl Coords2D<f32> {
    #[allow(dead_code)]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[allow(dead_code)]
    pub fn distance(&self, to: Coords2D<f32>) -> f32 {
        let dx = to.x - self.x;
        let dy = to.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }

    #[allow(dead_code)]
    pub fn move_towards(&mut self, angle: f32, distance: f32) {
        self.x += distance * angle.cos();
        self.y -= distance * angle.sin();
    }

    #[allow(dead_code)]
    pub fn convert_to_tile(&self, map: &Map) -> Coords2D<f32> {
        Coords2D {
            x: self.x / map.tile_size_in_px as f32,
            y: self.y / map.tile_size_in_px as f32,
        }
    }
}

#[allow(dead_code)]
/// Function to convert degrees to rad
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}

#[allow(dead_code)]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / PI
}

#[allow(dead_code)]
pub fn move_towards(start: Coords2D<f32>, angle: f32, distance: f32) -> Coords2D<f32> {
    Coords2D {
        x: (start.x + distance * angle.cos()),
        y: (start.y - distance * angle.sin()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let a = Coords2D { x: 0.0, y: 0.0 };
        let b = Coords2D { x: 3.0, y: 4.0 };
        assert_eq!(a.distance(b), 5.0);
    }

    #[test]
    fn test_float_to_integer_conversion() {
        let float_coords = Coords2D { x: 10.9, y: 5.1 };
        let int_coords: Coords2D<i32> = float_coords.into();
        assert_eq!(int_coords.x, 10);
        assert_eq!(int_coords.y, 5);
    }

    #[test]
    fn test_integer_to_float_conversion() {
        let int_coords = Coords2D { x: -25, y: 100 };
        let float_coords: Coords2D<f32> = int_coords.into();
        assert_eq!(float_coords.x, -25.0);
        assert_eq!(float_coords.y, 100.0);
    }

    #[test]
    fn test_move_towards() {
        let start_point: Coords2D<f32> = Coords2D::default();
        let res = move_towards(start_point, deg_to_rad(270 as f32), 5 as f32);
        println!("{:#?}", res);
        // Test with marge of error
        assert!((res.x - 0.0).abs() < 1e-6);
        assert!((res.y - 5.0).abs() < 1e-6);
    }
}
