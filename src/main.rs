use pixels::{Error, Pixels, SurfaceTexture};
use winit::{dpi::LogicalSize, event::Event, event_loop::EventLoop, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

const WIDTH: i32 = 400;
const HEIGHT: i32 = 300;
const SCALE_FACTOR: f64 = 3.0;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

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
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    // Create a grid full of ded cells
    let mut grid = Grid::get_randomized_grid();

    for _ in 0..3 {
        grid.update_cells();
    }

    event_loop.run(move |event, _, _| {
        if let Event::RedrawRequested(_) = event {
            let frame = pixels.frame_mut();
            grid.draw_cell(frame);

            // Draw it to the `SurfaceTexture`
            pixels.render().unwrap(); // todo handle error
        }

        if input.update(&event) {
            grid.update_cells();
            window.request_redraw();
        }
    });
}

#[derive(Clone, Debug)]
struct Cell {
    pub is_alive: bool,
    pub heat: u8,
}

impl Cell {
    fn dead_cell() -> Self {
        Self {
            is_alive: false,
            heat: 0,
        }
    }

    fn process_next_state(mut self, neighbours: i32) -> Self {
        let is_alive_next = match self.is_alive {
            // If the cell is alive, then it stays alive if it has either 2 or 3 live neighbors
            true => (2..=3).contains(&neighbours),

            // If the cell is dead, then it springs to life only in the case that it has 3 live neighbors
            false => neighbours == 3,
        };

        self.is_alive = is_alive_next;
        // if the cell is alive, its heat is 255,
        // otherwise it decays from 1
        self.heat = if is_alive_next {
            255
        } else {
            self.heat.saturating_sub(1)
        };

        self
    }
}

#[derive(Clone, Debug)]
struct Grid {
    pub cells: Vec<Cell>,
    pub next_step_cells: Vec<Cell>,
}

impl Grid {
    fn get_randomized_grid() -> Self {
        let mut rng: randomize::PCG32 = (1_u64, 1_u64).into();

        let cells: Vec<Cell> = (0..(HEIGHT as usize * WIDTH as usize))
            .map(|_| Cell {
                is_alive: randomize::f32_half_open_right(rng.next_u32()) > 0.90,
                heat: 0,
            })
            .collect();

        let next_step_cells: Vec<Cell> = vec![Cell::dead_cell(); HEIGHT as usize * WIDTH as usize];

        Self {
            cells,
            next_step_cells,
        }
    }

    fn draw_cell(&mut self, frame: &mut [u8]) {
        for (cell, pixel) in self.cells.iter().zip(frame.chunks_exact_mut(4)) {
            let color = if cell.is_alive {
                [0, 0xff, 0xff, 0xff]
            } else {
                [0, 0, cell.heat, 0xff]
            };

            pixel.copy_from_slice(&color);
        }
    }

    fn update_cells(&mut self) {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let id = x + y * WIDTH;
                
                // calculate neighbours of that cell
                let neighbours_cell_count: i32 =
                    // From top-left to bottom-right
                    self.cells
                        .get((id - WIDTH - 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false) as i32 +
                    self.cells
                        .get((id - WIDTH) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false) as i32 +
                    self.cells
                        .get((id - WIDTH + 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false) as i32 +
                    self.cells
                        .get((id - 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false) as i32 +
                    self.cells
                        .get((id + 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false) as i32 +
                    self.cells
                        .get((id + WIDTH - 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false) as i32 +
                    self.cells
                        .get((id + WIDTH) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false) as i32 +
                    self.cells
                        .get((id + WIDTH + 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false) as i32
                ;

                let next_state = self.cells[id as usize].clone().process_next_state(neighbours_cell_count);
                self.next_step_cells[id as usize] = next_state;
            }
        }
        std::mem::swap(&mut self.next_step_cells, &mut self.cells);
    }
}

