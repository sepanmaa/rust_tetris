/*

    Tetris clone in Rust
    Copyright (C) 2016  Juho Sepänmaa

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

extern crate sdl2;
extern crate rand;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::rect::{Rect, Point};
use std::time::{Duration, Instant};


#[derive(Clone, Copy)]
enum Tile {
    Empty, Red, Yellow, Blue, Green, Cyan, Magenta, Grey, 
}


struct Block<'a> {
    x: i32,
    y: i32,
    rotation: usize,
    v: &'a [(i32,i32)],
    tile: Tile,
}

static L_BLOCK: [(i32, i32); 16] = [(0,0), (1,0), (1,1), (1,2),
                                    (0,1), (1,1), (2,1), (2,0),
                                    (1,0), (1,1), (1,2), (2,2),
                                    (0,1), (0,2), (1,1), (2,1)];

static J_BLOCK: [(i32, i32); 16] = [(0,2), (1,2), (1,1), (1,0),
                                    (0,0), (0,1), (1,1), (2,1),
                                    (1,0), (2,0), (1,1), (1,2),
                                    (0,1), (1,1), (2,1), (2,2)];

static T_BLOCK: [(i32, i32); 16] = [(1,0), (0,1), (1,1), (2,1),
                                    (1,0), (1,1), (2,1), (1,2),
                                    (0,1), (1,1), (2,1), (1,2),
                                    (0,1), (1,1), (1,2), (1,0)];

static I_BLOCK: [(i32, i32); 8] = [(0,1), (1,1), (2,1), (3,1),
                                   (1,0), (1,1), (1,2), (1,3)];

static Z_BLOCK: [(i32, i32); 8] = [(0,0), (1,0), (1,1), (2,1),
                                   (1,1), (1,2), (2,1), (2,0)];

static N_BLOCK: [(i32, i32); 8] = [(0,0), (0,1), (1,1), (1,2),
                                   (0,1), (1,1), (1,0), (2,0)];

static O_BLOCK: [(i32, i32); 4] = [(0,0), (1,0), (0,1), (1,1)];


const WIDTH: i32 = 10;

const HEIGHT: i32 = 20;

const TILE_SIZE: i32 = 30;


macro_rules! move_block {
    ($time:ident, $block:ident, $grid:ident,
     $set_block:expr, $cancel_move:expr, $delay_millis:expr) => {
        if $time.elapsed().subsec_nanos() / 1_000_000 > $delay_millis {
            $set_block;
            if collision(&$block, &$grid) {
                $cancel_move;
            } else {
                $time = Instant::now();
            }
        }
    };
}


macro_rules! block {
    ($arr:ident, $tile_type:ident) => {
        Block { x: 5, y: 0, rotation: 0, v: &$arr, tile: Tile::$tile_type }        
    }
}


macro_rules! next_block {
    ($block:ident, $preview:ident) => {
        $block = $preview;
        $block.x = 5;
        $block.y = 0;
        $preview = random_block();
        $preview.x = 20;
        $preview.y = 2;
    }
}


fn collision(block: &Block, grid: &Vec<Tile>) -> bool {
    let block_slice = &block.v[block.rotation*4 .. (block.rotation+1)*4];
    for &(x, y) in block_slice.iter() {
        let i = ((block.y+y)*WIDTH+(x+block.x)) as usize;
        if block.y+y >= HEIGHT || block.x+x >= WIDTH
            || block.y+y < 0 || block.x+x < 0 {
            return true;
        }
        match grid[i] {
            Tile::Empty => {},
            _ => return true,
        }
    }
    return false;
}


fn insert_block(block: &Block, grid: &mut Vec<Tile>) {
    let block_slice = &block.v[block.rotation*4 .. (block.rotation+1)*4];
    for &(x, y) in block_slice.iter() {
        let i = ((block.y+y)*WIDTH+(x+block.x)) as usize;
        if i >= (WIDTH*HEIGHT) as usize { continue; }
        grid[i] = block.tile;            
    }
}


fn clear_rows(grid: &mut Vec<Tile>) -> i32 {
    let mut full_rows = vec![];
    let mut score = 0;
    for (row, chunk) in grid.chunks(WIDTH as usize).enumerate() {
        if chunk.iter().all(|tile| match tile {
            &Tile::Empty => false,
            _ => true,
        }) {
            full_rows.push(row);
            score += 1;
        }
    }
    for row in full_rows {
        for i in (0..row+1).rev() {
            for j in 0..WIDTH {
                let index = i*(WIDTH as usize)+j as usize;
                match i {
                    0 => grid[index] = Tile::Empty,
                    _ => grid[index] = grid[index-WIDTH as usize],
                }
            }
        }
    }
    return score;
}


fn draw_tile(renderer: &mut sdl2::render::Renderer, x: i32, y: i32, tile: Tile) {
    let color = match tile {
        Tile::Yellow => Color::RGB(255, 255, 0),
        Tile::Red => Color::RGB(255, 0, 0),
        Tile::Blue => Color::RGB(0, 0, 255),
        Tile::Cyan => Color::RGB(0, 255, 255),
        Tile::Magenta => Color::RGB(255, 0, 255),
        Tile::Green => Color::RGB(0, 255, 0),
        Tile::Grey => Color::RGB(127, 127, 127),
        _ => Color::RGB(0, 0, 0),
    };
    let (r, g, b) = color.rgb();
    let dark_color = Color::RGB(r / 4, g / 4, b / 4);
    let medium_color = Color::RGB(r / 2, g / 2, b / 2);

    renderer.set_draw_color(dark_color);
    renderer.fill_rect(Rect::new(x*TILE_SIZE, y*TILE_SIZE,
                                 TILE_SIZE as u32, TILE_SIZE as u32)).ok();

    renderer.set_draw_color(medium_color);
    renderer.fill_rect(Rect::new(x*TILE_SIZE+2, y*TILE_SIZE+2,
                                 TILE_SIZE as u32 - 4, TILE_SIZE as u32 - 4)).ok();

    renderer.set_draw_color(color);
    let point = Point::new(x*TILE_SIZE, y*TILE_SIZE);
    renderer.draw_line(point, point.offset(0, TILE_SIZE-2)).ok();
    renderer.draw_line(point.offset(1, 1), point.offset(1, TILE_SIZE-3)).ok();
    renderer.draw_line(point, point.offset(TILE_SIZE-2, 0)).ok();
    renderer.draw_line(point.offset(0, 1), point.offset(TILE_SIZE-3, 1)).ok();    
}


fn draw_block(renderer: &mut sdl2::render::Renderer, block: &Block) {
    let block_slice = &block.v[block.rotation*4 .. (block.rotation+1)*4];
    for &(x, y) in block_slice.iter() {
        draw_tile(renderer, block.x+x, block.y+y, block.tile);
    }
}


fn draw_grid(renderer: &mut sdl2::render::Renderer, grid: &Vec<Tile>) {
    renderer.set_draw_color(Color::RGB(0, 0, 0));
    renderer.fill_rect(Rect::new(0, 0, 10*TILE_SIZE as u32, 20*TILE_SIZE as u32)).ok();
    for (i, &tile) in grid.iter().enumerate() {
        match tile {
            Tile::Empty => continue,
            _ => {},
        }
        let y = i as i32 / WIDTH;
        let x = i as i32 % WIDTH;
        draw_tile(renderer, x, y, tile);
    }
}



fn random_block<'a>() -> Block<'a> {
    let r = rand::random::<u8>() % 7;
    match r {
        0 => block!(L_BLOCK, Yellow),
        1 => block!(T_BLOCK, Grey),
        2 => block!(J_BLOCK, Magenta),
        3 => block!(O_BLOCK, Blue),
        4 => block!(I_BLOCK, Red),
        5 => block!(N_BLOCK, Green),
        _ => block!(Z_BLOCK, Cyan),
    }
}

fn draw_text(renderer: &mut sdl2::render::Renderer, font: &sdl2::ttf::Font,
             text: &str, x: i32, y: i32) {
    let surface = font.render(text).blended(Color::RGB(255, 255, 255)).ok().unwrap();
    let tex = renderer.create_texture_from_surface(&surface).ok().unwrap();
    let src = Rect::new(0, 0, surface.width(), surface.height());
    let dst = Rect::new(x, y, surface.width(), surface.height());
    renderer.copy(&tex, Some(src), Some(dst)).ok();
}


fn main() {
    let context = sdl2::init().unwrap();
    let ttf = sdl2::ttf::init().unwrap();
    let font_path = std::path::Path::new("arial.ttf");
    let font = ttf.load_font(font_path, 24).expect("Font not found.");

    let video = context.video().unwrap();
    let window = video.window("Tetris", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut grid: Vec<Tile> = vec![Tile::Empty; (WIDTH*HEIGHT) as usize];
    let mut renderer = window.renderer().build().unwrap();
    let mut block;
    let mut preview = random_block();
    let mut event_pump = context.event_pump().unwrap();
    let mut drop_time = Instant::now();
    let mut rotate_time = Instant::now();
    let mut move_time = Instant::now();
    let mut fall_time = Instant::now();
    let mut score = 0;

    next_block!(block, preview);
    
    'running: loop {
        let frame_time = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'running,
                _ => {}
            }
        }
        for scancode in event_pump.keyboard_state().pressed_scancodes() {
            match scancode {
                Scancode::Escape =>
                    break 'running,
                Scancode::Up => {
                    let len = block.v.len() / 4;
                    move_block!(rotate_time, block, grid, block.rotation = (block.rotation + 1) % len,
                                block.rotation = ((block.rotation + len) - 1) % len, 150);
                },
                Scancode::Left =>
                    move_block!(move_time, block, grid, block.x -= 1, block.x += 1, 50),
                Scancode::Right =>
                    move_block!(move_time, block, grid, block.x += 1, block.x -= 1, 50),
                Scancode::Down =>
                    move_block!(move_time, block, grid, block.y += 1, block.y -= 1, 50),
                Scancode::Space => {
                    let t = drop_time.elapsed().subsec_nanos() / 1_000_000; 
                    if t > 200 {
                        drop_time = Instant::now();
                        while block.y < HEIGHT {
                            block.y += 1;
                            if collision(&block, &grid) {
                                block.y -= 1;
                                insert_block(&block, &mut grid);
                                next_block!(block, preview);
                                break;
                            }                            
                        }
                    }                    
                },
                _ => {}
            }
        }
        if fall_time.elapsed().subsec_nanos() / 1_000_000 > 250 {
            block.y += 1;
            if collision(&block, &grid) {
                block.y -= 1;
                if block.y == 0 {
                    break;
                }
                insert_block(&block, &mut grid);
                next_block!(block, preview);
            }
            fall_time = Instant::now();
        }

        score += clear_rows(&mut grid);
        renderer.set_draw_color(Color::RGB(64, 64, 64));
        renderer.clear();
        draw_grid(&mut renderer, &grid);
        draw_block(&mut renderer, &block);
        draw_block(&mut renderer, &preview);
        let score_str = format!("Score: {}", score);
        draw_text(&mut renderer, &font, &score_str, 600, 400);
        renderer.present();

        let delta = frame_time.elapsed().subsec_nanos() / 1_000_000;
        if delta < 16 {
            std::thread::sleep(Duration::from_millis(16-delta as u64));
        }
    }
    println!("Score: {}", score);
}
