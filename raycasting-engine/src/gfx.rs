use crate::{
    coords2_d::Coords2D, deg_to_rad, get_char_data, send_precise_ray, Player, PreciseRaycastResult,
    BORDER_WIDTH, CELLING_COLOR, FLOOR_COLOR, FLOOR_TWEEN_COLOR, FONT_HEIGHT, FONT_WIDTH,
    WALL_DARK_COLOR, WALL_DARK_TWEEN_COLOR, WALL_LIGHT_COLOR, WALL_LIGHT_TWEEN_COLOR,
};
use chinchilib::{
    put_pixel,
    rgb::{self, FromSlice, RGBA8},
};

pub fn modulo(x: &mut i32, y: &mut i32, screen_size: (usize, usize)) {
    if *x < 0 {
        *x += screen_size.0 as i32;
    } else if *x >= screen_size.0 as i32 {
        *x -= screen_size.0 as i32;
    }
    if *y < 0 {
        *y += screen_size.1 as i32;
    } else if *y >= screen_size.1 as i32 {
        *y -= screen_size.1 as i32;
    }
}

pub fn draw_all(buffer: &mut [u8], width: usize, player: &mut Player) {
    Renderer::new(buffer, width, player).draw_all()
}

struct Renderer<'a> {
    buffer: &'a mut [u8],
    width: usize,
    player: &'a mut Player,
    fov_rad: f32,
    angle_step: f32,
    pos_in_tiles: Coords2D<f32>,
}

impl<'a> Renderer<'a> {
    fn new(buffer: &'a mut [u8], width: usize, player: &'a mut Player) -> Self {
        let fov_rad = deg_to_rad(player.fov);
        let angle_step = fov_rad / player.screen_size.0 as f32;
        let pos_in_tiles = Coords2D {
            x: player.pos.x / player.map.tile_size_in_px as f32,
            y: player.pos.y / player.map.tile_size_in_px as f32,
        };
        Self {
            buffer,
            width,
            player,
            fov_rad,
            angle_step,
            pos_in_tiles,
        }
    }

    pub fn draw_all(&mut self) {
        let mut ray_angle = self.player.angle - (self.fov_rad / 2.0);
        let screen_w = self.player.screen_size.0;
        for x in 0..screen_w {
            match send_precise_ray(&self.player.map, self.pos_in_tiles, ray_angle) {
                Ok(res) => self.draw_wall_column(x, &res),
                Err(_) => self.draw_empty_column(x),
            }
            ray_angle += self.angle_step;
        }
    }

    fn draw_wall_column(&mut self, x: usize, res: &PreciseRaycastResult) {
        let corrected_distance = res.distance;
        let tile_size = self.player.map.tile_size_in_px as f32;

        let offset_in_tile = if res.side {
            res.collide_point_in_px.x % tile_size
        } else {
            res.collide_point_in_px.y % tile_size
        };

        let (current_wall_color, tween_wall_color) =
            if offset_in_tile < BORDER_WIDTH || offset_in_tile > (tile_size - BORDER_WIDTH) {
                (RGBA8::new(10, 10, 10, 255), Some(RGBA8::new(5, 5, 5, 255)))
            } else if res.wall_facing_dir.is_north_east() {
                (WALL_DARK_COLOR, Some(WALL_DARK_TWEEN_COLOR))
            } else {
                (WALL_LIGHT_COLOR, Some(WALL_LIGHT_TWEEN_COLOR))
            };

        let wall_h = if corrected_distance < 0.01 {
            self.player.screen_size.1 as f32
        } else {
            self.player.screen_size.1 as f32 / corrected_distance
        };
        let wall_height = wall_h as usize;

        let screen_h = self.player.screen_size.1;
        let wall_start_y = (screen_h.saturating_sub(wall_height)) / 2;
        let wall_end_y = wall_start_y.saturating_add(wall_height);

        draw_vertical_line(
            self.buffer,
            self.width,
            x,
            0,
            wall_start_y,
            CELLING_COLOR,
            None,
        );
        draw_vertical_line(
            self.buffer,
            self.width,
            x,
            wall_start_y,
            wall_end_y,
            current_wall_color,
            tween_wall_color,
        );
        draw_vertical_line(
            self.buffer,
            self.width,
            x,
            wall_end_y,
            screen_h,
            FLOOR_COLOR,
            Some(FLOOR_TWEEN_COLOR),
        );
    }

    fn draw_empty_column(&mut self, x: usize) {
        let mid_pos = Coords2D {
            x: x as f32,
            y: self.player.screen_size.1 as f32 / 2.0,
        };
        draw_line(
            self.buffer,
            self.width,
            Coords2D {
                x: x as f32,
                y: 0.0,
            },
            mid_pos,
            CELLING_COLOR,
        );
        draw_line(
            self.buffer,
            self.width,
            mid_pos,
            Coords2D {
                x: x as f32,
                y: self.player.screen_size.1 as f32,
            },
            FLOOR_COLOR,
        );
    }
}

#[allow(dead_code)]
pub fn draw_vertical_line(
    buffer: &mut [u8],
    width: usize,
    x: usize,
    y_start: usize,
    y_end: usize,
    start_color: RGBA8,
    end_color: Option<RGBA8>,
) {
    let pixels = buffer.as_rgba_mut();
    if x >= width {
        return;
    }
    let buffer_height = pixels.len() / width;
    let start = y_start.min(buffer_height);
    let end = y_end.min(buffer_height);
    if start >= end {
        return;
    }
    let mut index = start * width + x;
    let stride = width;
    if end_color.is_none() || start_color == end_color.unwrap() {
        for _ in start..end {
            pixels[index] = start_color;
            index += stride;
        }
        return;
    }
    let target = end_color.unwrap();
    let height = (end - start) as f32;
    let mut r = start_color.r as f32;
    let mut g = start_color.g as f32;
    let mut b = start_color.b as f32;
    let r_step = (target.r as f32 - r) / height;
    let g_step = (target.g as f32 - g) / height;
    let b_step = (target.b as f32 - b) / height;
    for _ in start..end {
        pixels[index] = RGBA8::new(r as u8, g as u8, b as u8, 255);
        r += r_step;
        g += g_step;
        b += b_step;
        index += stride;
    }
}

#[allow(dead_code)]
pub fn draw_line(
    buffer: &mut [u8],
    width: usize,
    coord1: Coords2D<f32>,
    coord2: Coords2D<f32>,
    color: rgb::RGBA8,
) {
    let x0 = coord1.x.round() as i32;
    let y0 = coord1.y.round() as i32;
    let x1 = coord2.x.round() as i32;
    let y1 = coord2.y.round() as i32;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;
    let height = (buffer.len() / 4) / width;
    loop {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            put_pixel(buffer, width, x as usize, y as usize, color);
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

#[allow(dead_code)]
pub fn draw_circle(
    buffer: &mut [u8],
    width: usize,
    coord1: Coords2D<f32>,
    beam: f32,
    color: rgb::RGBA8,
    screen_size: (usize, usize),
) {
    let mut angle: f32 = 0.0;
    let angle_step: f32 = 0.01;
    let max_angle = std::f32::consts::PI * 2.0;
    let screen_w = screen_size.0 as i32;
    let screen_h = screen_size.1 as i32;

    while angle <= max_angle {
        let mut x_signed = (coord1.x + beam * angle.cos()).round() as i32;
        let mut y_signed = (coord1.y + beam * angle.sin()).round() as i32;
        if x_signed < 0 {
            x_signed += screen_w;
        } else if x_signed >= screen_w {
            x_signed -= screen_w;
        }
        if y_signed < 0 {
            y_signed += screen_h;
        } else if y_signed >= screen_h {
            y_signed -= screen_h;
        }
        let x_final = x_signed as usize;
        let y_final = y_signed as usize;
        put_pixel(buffer, width, x_final, y_final, color);
        angle += angle_step;
    }
}

#[allow(dead_code)]
pub fn draw_spiral(
    buffer: &mut [u8],
    width: usize,
    coord1: Coords2D<f32>,
    beam: f32,
    color: RGBA8,
    screen_size: (usize, usize),
) {
    let mut angle: f32 = 0.0;
    let mut distance: f32 = 0.0;
    let angle_step: f32 = 0.001;
    let distance_step: f32 = 0.01;

    while distance <= beam {
        let mut x_signed = (coord1.x + distance * angle.cos()).round() as i32;
        let mut y_signed = (coord1.y + distance * angle.sin()).round() as i32;
        if x_signed < 0 {
            x_signed += screen_size.0 as i32;
        } else if x_signed >= screen_size.0 as i32 {
            x_signed -= screen_size.0 as i32;
        }
        if y_signed < 0 {
            y_signed += screen_size.1 as i32;
        } else if y_signed >= screen_size.1 as i32 {
            y_signed -= screen_size.1 as i32;
        }
        put_pixel(buffer, width, x_signed as usize, y_signed as usize, color);
        angle += angle_step;
        distance += distance_step;
    }
}

#[allow(dead_code)]
pub fn draw_rectangle(
    buffer: &mut [u8],
    width: usize,
    coord: Coords2D<f32>,
    rec_width: usize,
    rec_height: usize,
    screen_size: (usize, usize),
    color: RGBA8,
) {
    let mut y: i32 = 0;
    while y < rec_height as i32 {
        let mut x: i32 = 0;
        while x < rec_width as i32 {
            let mut draw_x: i32 = coord.x as i32 + x;
            let mut draw_y: i32 = coord.y as i32 + y;
            modulo(&mut draw_x, &mut draw_y, screen_size);
            put_pixel(buffer, width, draw_x as usize, draw_y as usize, color);
            x += 1;
        }
        y += 1;
    }
}

#[allow(dead_code)]
pub fn draw_rectangle_from(
    buffer: &mut [u8],
    width: usize,
    coord: Coords2D<f32>,
    rec_width: usize,
    rec_height: usize,
    screen_size: (usize, usize),
    color: &[RGBA8],
) {
    let mut y: i32 = 0;
    while y < rec_height as i32 {
        let mut x: i32 = 0;
        while x < rec_width as i32 {
            let mut draw_x: i32 = coord.x as i32 + x;
            let mut draw_y: i32 = coord.y as i32 + y;
            modulo(&mut draw_x, &mut draw_y, screen_size);
            put_pixel(
                buffer,
                width,
                draw_x as usize,
                draw_y as usize,
                color[(y * rec_width as i32 + x) as usize],
            );
            x += 1;
        }
        y += 1;
    }
}

#[allow(dead_code)]
pub fn draw_text(
    buffer: &mut [u8],
    width: usize,
    coord: Coords2D<f32>,
    text: &str,
    color: RGBA8,
    size: usize,
) {
    let mut cursor_x = coord.x as usize;
    let mut start_y = coord.y as usize;
    let buffer_len = buffer.len() / 4;
    let height = buffer_len / width;
    let char_width = (FONT_WIDTH + 1) as usize * size;
    let char_height = (FONT_HEIGHT as usize + 1) * size;
    let color_bytes = [color.r, color.g, color.b, color.a];

    for char in text.chars() {
        if char == '\n' {
            cursor_x = coord.x as usize;
            start_y += char_height;
            continue;
        }
        if start_y >= height {
            break;
        }
        if cursor_x >= width {
            cursor_x += char_width;
            continue;
        }
        let data = get_char_data(char);
        let max_row = (FONT_HEIGHT as usize).min((height - start_y) / size);
        for (row_idx, _) in data.iter().enumerate().take(max_row) {
            let row_byte = data[row_idx];
            if row_byte == 0 {
                continue;
            }
            let base_py = start_y + (row_idx * size);
            let py_end = (base_py + size).min(height);
            for bit_idx in 0..FONT_WIDTH {
                let bit_mask = 1u8 << (FONT_WIDTH - 1 - bit_idx);
                if (row_byte & bit_mask) != 0 {
                    let base_px = cursor_x + (bit_idx as usize * size);
                    if base_px >= width {
                        continue;
                    }
                    let px_end = (base_px + size).min(width);
                    if size == 1 {
                        let pixel_idx = (base_py * width + base_px) * 4;
                        if pixel_idx + 3 < buffer.len() {
                            buffer[pixel_idx..pixel_idx + 4].copy_from_slice(&color_bytes);
                        }
                    } else {
                        for py in base_py..py_end {
                            let row_offset = py * width * 4;
                            for px in base_px..px_end {
                                let pixel_idx = row_offset + px * 4;
                                if pixel_idx + 3 < buffer.len() {
                                    buffer[pixel_idx..pixel_idx + 4].copy_from_slice(&color_bytes);
                                }
                            }
                        }
                    }
                }
            }
        }
        cursor_x += char_width;
    }
}

#[allow(dead_code)]
pub fn draw_minimap(buffer: &mut [u8], width: usize, player: &Player) {
    let mini_map_center = Coords2D {
        x: player.screen_size.0 as f32 - 120.0,
        y: 120.0,
    };
    let cos_a = player.angle.cos();
    let sin_a = player.angle.sin();
    let radius: f32 = 100.0;
    let radius_i32 = radius as i32;
    let zoom: f32 = 0.1;
    for y in -radius_i32..radius_i32 {
        for x in -radius_i32..radius_i32 {
            // Calcul la distance (si on est en dehors de la mini map ça dégage)
            if x * x + y * y > radius_i32 * radius_i32 {
                continue;
            }
            // Tourne autour du joueur
            let rotation = Coords2D {
                x: -x as f32 * sin_a - y as f32 * cos_a,
                y: -x as f32 * cos_a + y as f32 * sin_a,
            };
            let world = Coords2D {
                x: player.pos.x + rotation.x / zoom,
                y: player.pos.y + rotation.y / zoom,
            };
            let tile = Coords2D {
                x: (world.x / player.map.tile_size_in_px as f32).floor() as i32,
                y: (world.y / player.map.tile_size_in_px as f32).floor() as i32,
            };
            let mut pixel_color = chinchilib::rgb::RGBA8::new(0, 0, 0, 0);
            if tile.x >= 0
                && tile.x < player.map.width_in_tiles as i32
                && tile.y >= 0
                && tile.y < player.map.height_in_tiles as i32
            {
                let idx = (tile.y as usize) * player.map.width_in_tiles + (tile.x as usize);

                if player.map.tiles[idx] {
                    // C'est un MUR
                    pixel_color = chinchilib::rgb::RGBA8::new(200, 200, 200, 255);
                } else {
                    // C'est du SOL
                    pixel_color = chinchilib::rgb::RGBA8::new(50, 50, 50, 255);
                }
            }
            let screen = Coords2D {
                x: (mini_map_center.x + x as f32) as usize,
                y: (mini_map_center.y + y as f32) as usize,
            };
            if screen.x < width && screen.y < (buffer.len() / 4) / width {
                chinchilib::put_pixel(buffer, width, screen.x, screen.y, pixel_color);
            }
        }
    }
    draw_circle(
        buffer,
        width,
        mini_map_center,
        8.0,
        RGBA8 {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        },
        player.screen_size,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use chinchilib::rgb;

    // Useful function for calculating the index of the first byte of a pixel (R)
    // Simulates what put_pixel does
    fn get_pixel_index(width: usize, x: usize, y: usize) -> usize {
        // EVery pixel is compose of 4 bytes (R, G, B, A)
        (y * width + x) * 4
    }

    #[test]
    fn test_horizontal_line() {
        let width = 10;
        let height = 1;
        let mut buffer = vec![0u8; width * height * 4];
        let start = Coords2D { x: 2.0, y: 0.0 };
        let end = Coords2D { x: 6.0, y: 0.0 };
        let color = rgb::RGBA8 {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        };
        draw_line(&mut buffer, width, start, end, color);
        let expected_x_coords = vec![2, 3, 4, 5, 6];
        for x in 0..width {
            let index = get_pixel_index(width, x, 0);
            if expected_x_coords.contains(&x) {
                assert_eq!(buffer[index], 255, "Pixel à ({}, 0) devrait être rouge", x);
            } else {
                assert_eq!(buffer[index], 0, "Pixel à ({}, 0) devrait être vide", x);
            }
        }
    }

    #[test]
    fn test_draw_rectangle_wrapping() {
        let width = 10;
        let height = 10;
        let screen_size = (width, height);
        let mut buffer = vec![0u8; width * height * 4];
        let start = Coords2D { x: 9.0, y: 9.0 };
        let rec_w = 2;
        let rec_h = 2;
        let color = rgb::RGBA8 {
            r: 0,
            g: 255,
            b: 0,
            a: 255,
        };

        draw_rectangle(&mut buffer, width, start, rec_w, rec_h, screen_size, color);

        let expected_coords = vec![(9, 9), (0, 9), (9, 0), (0, 0)];

        for y in 0..height {
            for x in 0..width {
                let index = get_pixel_index(width, x, y);
                if expected_coords.contains(&(x, y)) {
                    assert_eq!(
                        buffer[index + 1],
                        255,
                        "Le pixel ({}, {}) devrait être VERT",
                        x,
                        y
                    );
                } else {
                    assert_eq!(
                        buffer[index + 1],
                        0,
                        "Le pixel ({}, {}) devrait être vide",
                        x,
                        y
                    );
                }
            }
        }
    }
}
