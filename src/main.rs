use pixels::{wgpu::Color, Error, Pixels, SurfaceTexture};
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

const WIDTH: i32 = 500;
const HEIGHT: i32 = 300;
const SCALE_FACTOR: f64 = 3.0;
const HEAT_MAP_COLOR_SCHEME: [[u8; 4]; 7] = [
    [0x10, 0x10, 0x10, 0xff],
    [0, 0, 0xff, 0xff],
    [0, 0xff, 0xff, 0xff],
    [0, 0xff, 0, 0xff],
    [0xff, 0xff, 0, 0xff],
    [0xff, 0, 0, 0xff],
    [0xff, 0xff, 0xff, 0xff],
];

#[derive(Clone, Debug)]
struct Cell {
    pub is_alive: bool,
    pub number_of_neighbours: u8,
}

impl Cell {
    fn dead_cell() -> Self {
        Self {
            is_alive: false,
            number_of_neighbours: 0,
        }
    }

    fn process_next_state(&self, neighbours: [bool; 8]) -> Self {
        let n_count = neighbours.into_iter().filter(|b| *b).count();
        let is_alive_next = match self.is_alive {
            // If the cell is alive, then it stays alive if it has either 2 or 3 live neighbors
            true => (2..=3).contains(&n_count),

            // If the cell is dead, then it springs to life only in the case that it has 3 live neighbors
            false => n_count == 3,
        };

        Self {
            is_alive: is_alive_next,
            number_of_neighbours: n_count as u8,
        }
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
                number_of_neighbours: 7,
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
                HEAT_MAP_COLOR_SCHEME[(cell.number_of_neighbours - 1) as usize]
            } else {
                [0, 0, 0, 0] // Black
            };

            pixel.copy_from_slice(&color);
        }
    }

    fn update_cells(&mut self) {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let id = x + y * WIDTH;
                let cell = &self.cells[id as usize];

                // calculate neighbours of that cell
                let neighbours_cell: [bool; 8] = [
                    // From top-left to bottom-right
                    self.cells
                        .get((id - WIDTH - 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false),
                    self.cells
                        .get((id - WIDTH) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false),
                    self.cells
                        .get((id - WIDTH + 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false),
                    self.cells
                        .get((id - 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false),
                    self.cells
                        .get((id + 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false),
                    self.cells
                        .get((id + WIDTH - 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false),
                    self.cells
                        .get((id + WIDTH) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false),
                    self.cells
                        .get((id + WIDTH + 1) as usize)
                        .map(|c| c.is_alive)
                        .unwrap_or(false),
                ];

                let next_state = cell.process_next_state(neighbours_cell);
                self.next_step_cells[id as usize] = next_state;
            }
        }
        std::mem::swap(&mut self.next_step_cells, &mut self.cells);
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
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    // Create a grid full of ded cells
    let mut grid = Grid::get_randomized_grid();

    // Set clear color to red.
    pixels.clear_color(Color::BLACK);

    event_loop.run(move |event, _, control_flow| {
        // Clear the pixel buffer
        let frame = pixels.frame_mut();

        grid.draw_cell(frame);

        // Draw it to the `SurfaceTexture`
        pixels.render().unwrap(); // todo handle error
        window.request_redraw();

        grid.update_cells();
    });
}
