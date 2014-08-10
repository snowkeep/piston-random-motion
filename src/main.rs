//RANDOM MOTION
#![feature(globs)] //can use foo::*;

extern crate graphics;
extern crate piston;
extern crate sdl2_game_window;
extern crate opengl_graphics;

use std::cmp::{max, min}; //use for edge behav

use opengl_graphics::{
    Gl,
};
use sdl2_game_window::GameWindowSDL2;
use graphics::*;
use piston::{
    GameIterator,
    GameIteratorSettings,
    GameWindowSettings,
    KeyPress,
    MousePress,
    MouseMove,
    Render,
    Update,
};

use std::rand;
use std::rand::{Rng, SeedableRng, XorShiftRng};

pub static GRID_HEIGHT: uint = 100;
pub static GRID_WIDTH: uint = 100;

pub static BLOCK_SIZE: uint = 8;

pub static WINDOW_HEIGHT: uint = GRID_HEIGHT * BLOCK_SIZE;
pub static WINDOW_WIDTH: uint = GRID_WIDTH * BLOCK_SIZE;

#[deriving(PartialEq, Clone)]
struct Loc {
	pub x: uint,
	pub y: uint,
    pub color: (f32, f32, f32)
}

struct GameState {
    pub map: [[bool, ..GRID_HEIGHT], ..GRID_WIDTH],
    pub entities: Vec<Loc>,
    pub max_x: uint,
    pub max_y: uint,
    pub rng: XorShiftRng
}

impl GameState {
    pub fn new(square_side: uint, max_x: uint, max_y: uint) -> GameState {

        let mut map = [[false, ..GRID_HEIGHT], ..GRID_WIDTH];
        let mut new_entities: Vec<Loc> = Vec::with_capacity((square_side*square_side*2));
        //create 2 squares of red and blue particles in opposite corners
        for x in range(0, square_side){
            for y in range(0, square_side){
                map[x][y] = true;
                new_entities.push(
                    Loc { x: x,
                          y: y,
                          color: (1.0, 0.0, 0.0)}
                    );

                map[(GRID_WIDTH - x -1)][(GRID_HEIGHT - y - 1)] = true;
                new_entities.push(
                    Loc { x: GRID_WIDTH - x -1,
                          y: GRID_HEIGHT - y - 1,
                          color: (0.0, 0.0, 1.0)});
            }
        };
        let rng: rand::XorShiftRng = SeedableRng::from_seed([1, 2, 3, 4]);

        GameState {
            map: map,
            entities: new_entities,
            max_x: max_x,
            max_y: max_y,
            rng: rng
        }
    }

    pub fn mov(&mut self, loc: Loc, x: int, y: int) -> Loc {
        //stopping behavior, to prevent getting out of edges
        let x = min(max( (loc.x as int) + x, 0), (self.max_x as int) - 1);
        let y = min(max( (loc.y as int) + y, 0), (self.max_y as int) - 1);

        Loc {
            x: x as uint,
            y: y as uint,
            color: loc.color
        }

    }//end mov

    pub fn random_mov(&mut self, loc: Loc) -> Loc {
        let r = self.rng.gen::<uint>() % 8; // % trick to get range 0-7
        let new_entity = match r {
            0 => {self.mov(loc ,1, 0)},
            1 => {self.mov(loc, -1, 0)},
            2 => {self.mov(loc, 0, 1)},
            3 => {self.mov(loc, 0, -1)},
            4 => {self.mov(loc ,1, 1)},
            5 => {self.mov(loc, -1, -1)},
            6 => {self.mov(loc, -1, 1)},
            7 => {self.mov(loc, 1, -1)},
            _ => {loc} //should never happen
        };
        new_entity
    }//end random_mov


    pub fn update(&mut self) {
        //MAIN LOGIC
        for i in range(0, self.entities.len()) {
            let loc = self.entities[i];
            let new_loc = self.random_mov(loc);
            //calculate opposite loc for bouncing
            let (opp_mov_x, opp_mov_y) = (loc.x - new_loc.x, loc.y - new_loc.y);
            let opposite_new_loc = self.mov(new_loc, opp_mov_x as int, opp_mov_y as int);

            let mut new_free = true;
            let mut oppos_free = true;

            if self.map[new_loc.x][new_loc.y] == true {
                new_free = false };
            if self.map[opposite_new_loc.x][opposite_new_loc.y] == true {
                oppos_free = false };

            if new_free {
                self.map[loc.x][loc.y] = false;
                self.map[new_loc.x][new_loc.y] = true;
                *self.entities.get_mut(i) = new_loc; //self.entities[i] = new_loc; not working yet
            }//try to bounce
            else if oppos_free {
                self.map[loc.x][loc.y] = false;
                self.map[opposite_new_loc.x][opposite_new_loc.y] = true;
                *self.entities.get_mut(i) = opposite_new_loc;
            }// end if
        }//end for i
    }//end update

}//end impl GameState

fn main() {
    let mut window = GameWindowSDL2::new(
        GameWindowSettings {
            title: "Random motion of particles".to_string(),
            size: [WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32],
            fullscreen: false,
            exit_on_esc: true,
        }
    );

    let game_iter_settings = GameIteratorSettings {
            updates_per_second: 1000,
            max_frames_per_second: 60,
        };

    let ref mut gl = Gl::new();

    let mut game = GameState::new(45, GRID_WIDTH, GRID_HEIGHT);

    let mut update_counter: uint = 0;

    let mut paused = true;

    let mut mouse_x: f64 = 0.0;
    let mut mouse_y: f64 = 0.0;

    for event in GameIterator::new(&mut window, &game_iter_settings) {
        match event {
            Render(args) => {
                gl.viewport(0, 0, args.width as i32, args.height as i32);
                let c = Context::abs(args.width as f64, args.height as f64);
                c.rgb(0.0, 0.0, 0.0).draw(gl);
                for entity in game.entities.iter() {
                    c.circle(
                            (entity.x * BLOCK_SIZE + BLOCK_SIZE/2) as f64,
                            (entity.y * BLOCK_SIZE + BLOCK_SIZE/2) as f64,
                            (BLOCK_SIZE/2) as f64
                        )
                        .rgb(entity.color.val0(), entity.color.val1(), entity.color.val2())
                        .draw(gl);
                }
            },

            KeyPress(args) => {
                match args.key {
                    piston::keyboard::P => {paused = !paused},
                    piston::keyboard::Space => {game.update()},
                    piston::keyboard::C => {game = GameState::new(0, GRID_WIDTH, GRID_HEIGHT)}, //clean screan
                    piston::keyboard::R => {game = GameState::new(45, GRID_WIDTH, GRID_HEIGHT)}, //reset
                    _ => {}
                }
            }

            MouseMove(args) => {
                mouse_x = args.x; //get mouse coordinates for MousePress
                mouse_y = args.y;
            }

            MousePress(args) => {
                match args.button {
                    piston::mouse::Left => {
                        //translate mouse coord. to grid
                        let loc = Loc {x: (mouse_x/BLOCK_SIZE as f64) as uint,
                                       y: (mouse_y/BLOCK_SIZE as f64) as uint,
                                       color: (0.0, 1.0, 0.0)};
                        //if it exists, remove it
                        if game.map[loc.x][loc.y] {
                            game.map[loc.x][loc.y] = false;
                            for i in range(0, game.entities.len()){
                                if game.entities[i].x == loc.x && game.entities[i].y == loc.y {
                                    game.entities.swap_remove(i); //always O(1), doesn't preserve order
                                    break;
                                };
                            }
                        } else { //if it doesnt exist, add it
                            game.map[loc.x][loc.y] = true;
                            game.entities.push(loc);
                        };
                    },//Left
                    _ => {}
                }//match
            }//MousePress

            Update(_) => {
                if !paused {
                    update_counter += 1;
                    if update_counter == 10 {
                        game.update();
                        update_counter = 0;
                    }
                } //end if !paused
            }//end Update()
            _ => {}

        }
    }
}
