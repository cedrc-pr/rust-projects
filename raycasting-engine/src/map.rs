use log::log;

use crate::Coords2D;

const DEFAULT_TILE_SIZE_IN_PIXEL: usize = 150;

fn create_tiles(
    width: usize,
    height: usize,
    map_in_str: &str,
) -> (Vec<bool>, Option<Coords2D<f32>>, Vec<Entity>) {
    let mut tiles = Vec::with_capacity(width * height);
    let mut spawn_point = None;
    let mut entities: Vec<Entity> = Vec::new();
    for line in map_in_str.lines().enumerate() {
        for c in line.1.chars().enumerate() {
            tiles.push(c.1 == '#');
            if c.1 == 'S' {
                log!(log::Level::Info, "Spawn find");
                spawn_point = Some(Coords2D::new(c.0 as f32, line.0 as f32));
            } else if c.1 == 'c' {
                log!(log::Level::Info, "Coin found");
                let coin = Entity {
                    pos: Coords2D {
                        x: c.0 as f32 + 0.5,
                        y: line.0 as f32 + 0.5,
                    },
                    kind: EntityType::Coin,
                    interact: false,
                };
                entities.push(coin);
            }
        }
    }
    (tiles, spawn_point, entities)
}

#[derive(Clone, Debug)]
pub struct Entity {
    /// Position in map coord
    pub pos: Coords2D<f32>,
    /// Type of the entity
    pub kind: EntityType,
    /// If the entity has interact with the player, like coins is collected.
    pub interact: bool,
}

#[derive(Clone, Debug)]
pub enum EntityType {
    Coin,
}

/// A tilled map, where every tile can either be empty (false) of full (true) signaling the
/// presence of a wall.
pub struct Map {
    /// For rendering onto a screen, every tile should be represented as a square with a side of
    /// this many pixels.
    pub tile_size_in_px: usize,
    /// Number of collumns in .tiles.
    pub width_in_tiles: usize,
    /// Number of lines in .tiles.
    pub height_in_tiles: usize,
    /// Fixed size array for the state of each tile.
    pub tiles: Box<[bool]>,
    /// Spawn Position
    pub spawn_position: Option<Coords2D<f32>>,
    /// Entities, Coin, Monsters ...
    pub entities: Vec<Entity>,
}

impl Default for Map {
    /// Create a walled map of 10 by 5 tiles.
    fn default() -> Self {
        const WIDTH: usize = 10;
        const HEIGHT: usize = 5;
        Self::new(WIDTH, HEIGHT)
    }
}

impl std::fmt::Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, char) in self
            .tiles
            .iter()
            .map(|val| if *val { '#' } else { ' ' })
            .enumerate()
        {
            if idx != 0 && idx % self.width_in_tiles == 0 {
                writeln!(f)?;
            }
            write!(f, "{char}")?;
        }
        writeln!(f)?;
        Ok(())
    }
}

impl Map {
    /// Create a new map with these dimensions. The map will have walls on its 4 sides.
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = Vec::new();
        tiles.resize(width * height, false);
        let mut map = Self {
            tile_size_in_px: DEFAULT_TILE_SIZE_IN_PIXEL,
            width_in_tiles: width,
            height_in_tiles: height,
            tiles: tiles.into_boxed_slice(),
            spawn_position: None,
            entities: Vec::new(),
        };
        (0..map.width_in_tiles).for_each(|idx| map.tiles[idx] = true);
        (1..(map.height_in_tiles - 1)).for_each(|idx| map.tiles[idx * map.width_in_tiles] = true);
        (1..(map.height_in_tiles - 1))
            .for_each(|idx| map.tiles[idx * map.width_in_tiles + map.width_in_tiles - 1] = true);
        (0..map.width_in_tiles)
            .for_each(|idx| map.tiles[idx + map.width_in_tiles * (map.height_in_tiles - 1)] = true);
        map
    }

    pub fn new_from_str(map_in_str: &str) -> Self {
        let width = map_in_str.lines().next().unwrap().chars().count(); // TODO handle unwrap
        let height = map_in_str.lines().count();
        let (tiles, spawn_point, entities) = create_tiles(width, height, map_in_str);
        Self {
            tile_size_in_px: DEFAULT_TILE_SIZE_IN_PIXEL,
            width_in_tiles: width,
            height_in_tiles: height,
            tiles: tiles.into_boxed_slice(),
            spawn_position: spawn_point,
            entities,
        }
    }

    pub fn get_spawn_point(&self) -> Coords2D<f32> {
        if self.spawn_position.is_some() {
            self.spawn_position.unwrap()
        } else {
            Coords2D {
                x: self.width_in_tiles as f32 / 2.0,
                y: self.height_in_tiles as f32 / 2.0,
            }
        }
    }

    pub fn get_spawn_point_in_pixel(&self) -> Coords2D<f32> {
        let size = self.tile_size_in_px as f32;
        if self.spawn_position.is_some() {
            let cord = self.spawn_position.unwrap();
            Coords2D {
                x: cord.x * size + size / 2.0,
                y: cord.y * size + size / 2.0,
            }
        } else {
            Coords2D {
                x: self.width_in_tiles as f32 / 2.0 * size + size / 2.0,
                y: self.height_in_tiles as f32 / 2.0 * size + size / 2.0,
            }
        }
    }

    pub fn load_map(&mut self, map_in_str: &str) {
        let new_width = map_in_str.lines().next().unwrap_or("").chars().count();
        let new_height = map_in_str.lines().count();
        let (new_tiles, new_spawn_pos, entities) = create_tiles(new_width, new_height, map_in_str);
        self.width_in_tiles = new_width;
        self.height_in_tiles = new_height;
        self.tiles = new_tiles.into_boxed_slice();
        self.spawn_position = new_spawn_pos;
        self.entities = entities
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn four_by_four() {
        use std::io::Write;
        let map = super::Map::new(4, 4);
        let mut buf: Vec<u8> = Vec::new();
        write!(buf, "{map}").unwrap();
        let string = String::from_utf8(buf).unwrap();
        assert_eq!(string, "####\n#  #\n#  #\n####\n");
    }

    #[test]
    fn two_by_four() {
        use std::io::Write;
        let map = super::Map::new(2, 4);
        let mut buf: Vec<u8> = Vec::new();
        write!(buf, "{map}").unwrap();
        let string = String::from_utf8(buf).unwrap();
        assert_eq!(string, "##\n##\n##\n##\n");
    }
}
