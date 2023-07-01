use pixels::{Error, Pixels, SurfaceTexture, wgpu::Color};
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;
const SCALE_FACTOR: f64 = 10.0;

struct Cell {
    pub is_alive: bool,
}

impl Cell {
    fn toggle_state(&mut self) {
        self.is_alive = !self.is_alive;
    }

    fn set_is_alive(&mut self, is_alive: bool) {
        self.is_alive = is_alive;
    }

    fn process_next_state(&mut self, neighbours: Vec<bool>) {
        let n_count = neighbours.into_iter().filter(|b| *b).count();
        let is_alive_next = match self.is_alive {
            // If the cell is alive, then it stays alive if it has either 2 or 3 live neighbors
            true => (2..=3).contains(&n_count),

            // If the cell is dead, then it springs to life only in the case that it has 3 live neighbors
            false => n_count == 3,   
        };
        
        self.set_is_alive(is_alive_next);
    }
}

struct Grid {
    pub cells: Vec<Cell>,
}

impl Grid {
    fn get_randomized_grid() -> Grid {
        let mut rng: randomize::PCG32 = (1_u64, 1_u64).into();
        
        let cells: Vec<Cell> = (0..(HEIGHT as usize * WIDTH as usize)).map(|_| Cell {
            is_alive: randomize::f32_half_open_right(rng.next_u32()) > 0.9,
        }).collect();
        
        Grid {
            cells,
        }
    }

    fn draw_cell(&mut self, frame: &mut [u8]) {
        for (cell, pixel) in self.cells.iter().zip(frame.chunks_exact_mut(4)) {
            let color = if cell.is_alive {
                [0xff, 0xff, 0xff, 0xff] // White
            } else {
                [0, 0, 0, 0] // Black
            };
            
            pixel.copy_from_slice(&color);
        }
    }

    fn update_cells(&mut self) {
        self.cells.iter_mut().for_each(|c| {
            c.process_next_state(neighbours)
        })
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();

    // Creates the window that holds the game
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size =
            LogicalSize::new(WIDTH as f64 * SCALE_FACTOR, HEIGHT as f64 * SCALE_FACTOR);

        WindowBuilder::new()
            .with_title("Conway's Game of Life")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    // A 2D pixels buffer
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    // Create a grid full of ded cells
    let mut grid = Grid::get_randomized_grid();

    // Set clear color to red.
    pixels.clear_color(Color::BLACK);
    
    event_loop.run(move |event, _, control_flow| {
        // Clear the pixel buffer
        let frame = pixels.frame_mut();

        grid.draw_cell(frame);
        grid.update_cells();

        // Draw it to the `SurfaceTexture`
        pixels.render().unwrap(); // todo handle error
        window.request_redraw();
    });
}
