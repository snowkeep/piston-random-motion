//RANDOM MOTION

extern crate graphics;
extern crate piston;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate shader_version;
extern crate event;
extern crate input;

use std::cmp::{max, min}; //use for edge behav
use std::cell::RefCell;

use sdl2_window::Sdl2Window as Window;
use opengl_graphics::Gl;
use shader_version::opengl::OpenGL::_3_2;

use piston::RenderArgs;

use graphics::{
    Context,
    Ellipse
};

use event::{
    Event,
    Events,
    RenderEvent,
    UpdateEvent,
    PressEvent,
    MouseCursorEvent,
    WindowSettings
};

use input::Button;

use input::keyboard::Key::{
    P,
    C,
    R,
    Space

};

use input::mouse::MouseButton::{
    Left
};



use std::rand;
use std::rand::{Rng, SeedableRng, XorShiftRng};

const GRID_HEIGHT: usize = 100;
const GRID_WIDTH: usize = 100;

const BLOCK_SIZE: usize = 8;

const WINDOW_HEIGHT: usize = GRID_HEIGHT * BLOCK_SIZE;
const WINDOW_WIDTH: usize = GRID_WIDTH * BLOCK_SIZE;

#[derive(PartialEq, Clone)]
struct Loc {
  	pub x: usize,
	  pub y: usize,
    pub color: [f32; 4]
}

struct GameState {
    gl: Gl,
    pub map: [[bool; GRID_HEIGHT]; GRID_WIDTH],
    pub entities: Vec<Loc>,
    pub max_x: usize,
    pub max_y: usize,
    pub rng: XorShiftRng
}

impl GameState {
    pub fn new(gl: Gl, square_side: usize, max_x: usize, max_y: usize) -> GameState {

        let mut map = [[false; GRID_HEIGHT]; GRID_WIDTH];
        let mut new_entities: Vec<Loc> = Vec::with_capacity((square_side*square_side*2));
        //create 2 squares of red and blue particles in opposite corners
//        for x in range(0, square_side){
        for x in (0..square_side){
            for y in (0..square_side){
                map[x][y] = true;
                new_entities.push(
                    Loc { x: x,
                          y: y,
                          color: [1.0, 0.0, 0.0, 1.0]}
                    );

                map[(GRID_WIDTH - x -1)][(GRID_HEIGHT - y - 1)] = true;
                new_entities.push(
                    Loc { x: GRID_WIDTH - x -1,
                          y: GRID_HEIGHT - y - 1,
                          color: [0.0, 0.0, 1.0, 1.0]});
            }
        };
        let rng: rand::XorShiftRng = SeedableRng::from_seed([1, 2, 3, 4]);

        GameState {
            gl: gl,
            map: map,
            entities: new_entities,
            max_x: max_x,
            max_y: max_y,
            rng: rng
        }
    }

    pub fn mov(&self, loc: &Loc, x: isize, y: isize) -> Loc {
        //stopping behavior, to prevent getting out of edges
        let x = min(max( (loc.x as isize) + x, 0), (self.max_x as isize) - 1);
        let y = min(max( (loc.y as isize) + y, 0), (self.max_y as isize) - 1);

        Loc {
            x: x as usize,
            y: y as usize,
            color: loc.color
        }

    }//end mov

    fn random_mov(&mut self, loc: &Loc) -> Loc {
        let r = self.rng.gen::<usize>() % 8; // % trick to get range 0-7
        let new_entity = match r {
            0 => {self.mov(loc ,1, 0)},
            1 => {self.mov(loc, -1, 0)},
            2 => {self.mov(loc, 0, 1)},
            3 => {self.mov(loc, 0, -1)},
            4 => {self.mov(loc ,1, 1)},
            5 => {self.mov(loc, -1, -1)},
            6 => {self.mov(loc, -1, 1)},
            7 => {self.mov(loc, 1, -1)},
            _ => {self.mov(loc, 0, 0)} //should never happen
        };
        new_entity
    }//end random_mov
    
    pub fn render(&mut self, args: &RenderArgs) {
        self.gl.viewport(0, 0, args.width as i32, args.height as i32);
        let c = &Context::abs(args.width as f64, args.height as f64);
        graphics::clear(graphics::color::BLACK, &mut self.gl);
        for entity in self.entities.iter() {
              entity.render(c, &mut self.gl);
        }

    }


    pub fn update(&mut self) {
        //MAIN LOGIC
        for i in (0..self.entities.len()) {
            let ref loc = self.entities[i].clone();
            let new_loc = self.random_mov(loc);
            //calculate opposite loc for bouncing
            let (opp_mov_x, opp_mov_y) = (loc.x - new_loc.x, loc.y - new_loc.y);
            let opposite_new_loc = self.mov(&new_loc, opp_mov_x as isize, opp_mov_y as isize);

            let mut new_free = true;
            let mut oppos_free = true;

            if self.map[new_loc.x][new_loc.y] == true {
                new_free = false };
            if self.map[opposite_new_loc.x][opposite_new_loc.y] == true {
                oppos_free = false };

            if new_free {
                self.map[loc.x][loc.y] = false;
                self.map[new_loc.x][new_loc.y] = true;
                self.entities[i] = new_loc;
            }//try to bounce
            else if oppos_free {
                self.map[loc.x][loc.y] = false;
                self.map[opposite_new_loc.x][opposite_new_loc.y] = true;
                self.entities[i] = opposite_new_loc;
            }// end if
        }//end for i
    }//end update

}//end impl GameState

impl Loc {
    pub fn render(&self, c: &Context, gl: &mut Gl) {
        Ellipse::new(self.color).draw([
            (self.x * BLOCK_SIZE + BLOCK_SIZE/2) as f64,
            (self.y * BLOCK_SIZE + BLOCK_SIZE/2) as f64,
            (BLOCK_SIZE/2) as f64,
            (BLOCK_SIZE/2) as f64
        ], c, gl);
    }
}


fn main() {
    let window = Window::new(
        shader_version::OpenGL::_3_2,
        WindowSettings {
            title: "Random motion of particles".to_string(),
            size: [WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32],
            fullscreen: false,
            exit_on_esc: true,
            samples: 0
        }
    );

    let window = RefCell::new(window);

    let mut game = GameState::new(Gl::new(_3_2), 45, GRID_WIDTH, GRID_HEIGHT);

    let mut update_counter: usize = 0;

    let mut paused = true;

    let mut mouse_x: f64 = 0.0;
    let mut mouse_y: f64 = 0.0;

    for e in Events::new(&window) {
        let e: Event<input::Input> = e;
        e.press(|button| {
            match button {
                Button::Keyboard(key) => {
                    match key {

                        P => {paused = !paused},
                        Space => {game.update()},
                        C => {game = GameState::new(Gl::new(_3_2), 0, GRID_WIDTH, GRID_HEIGHT)}, //clean screan
                        R => {game = GameState::new(Gl::new(_3_2), 45, GRID_WIDTH, GRID_HEIGHT)}, //reset
                        _ => {}
                    }
                }
                Button::Mouse(click) => {
                    match click { 
                        Left => {
                            //translate mouse coord. to grid
                            let loc = Loc {x: (mouse_x/BLOCK_SIZE as f64) as usize,
                                           y: (mouse_y/BLOCK_SIZE as f64) as usize,
                                           color: [0.0, 1.0, 0.0, 1.0]};
                            //if it exists, remove it
                            if game.map[loc.x][loc.y] {
                                game.map[loc.x][loc.y] = false;
                                for i in (0..game.entities.len()){
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
                    }//match click
                }//MousePress
            }//match button
        });//press event
        e.mouse_cursor(|x, y| {
            mouse_x = x; //get mouse coordinates for MousePress
            mouse_y = y;
        });
        e.render(|r| game.render(r));
        e.update(|_| {
            if !paused {
                update_counter += 1;
                if update_counter == 10 {
                    game.update();
                    update_counter = 0;
                }
            } //end if !paused

        });
    }
}
