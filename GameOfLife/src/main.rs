use eframe::egui;
use rand::Rng;
use std::time::Instant;

const GRID_WIDTH: usize  = 150;
const GRID_HEIGHT: usize = 105;

const INITIAL_CELL_SIZE: f32 =  4.0;

// representa la cuadrícula del juego - cada celda puede estar viva (1) o muerta (0)
type CellGrid = [[u32; GRID_HEIGHT]; GRID_WIDTH];

trait GridOps {
    fn get_cell(&self, x: i32, y: i32) -> Option<u32> ;
    fn set_cell(&mut self, x: i32, y: i32, val: u32);
    fn update_generation(&mut self) ;
    fn copy_from(&mut self, source: &CellGrid);
}

// ---------------------------------------------------------------------------------

fn empty_grid() -> CellGrid {
    [[0; GRID_HEIGHT]; GRID_WIDTH]
}

fn random_grid() -> CellGrid{
    let mut rng = rand::thread_rng() ;
    let mut grid = empty_grid();
   
    for x in 0..GRID_WIDTH{
        for y in 0..GRID_HEIGHT{
            grid[x][y] =  rng.gen::<u32>() & 1;
        }
    }

    grid
}

// -----------------------------------------------------------------------------------

impl GridOps for CellGrid {

    fn get_cell(&self, x: i32, y: i32) -> Option<u32>{
        let wrapped_x = ((x % GRID_WIDTH as i32 + GRID_WIDTH as i32) % GRID_WIDTH as i32) as usize;
        let wrapped_y = ((y % GRID_HEIGHT as i32 + GRID_HEIGHT as i32) % GRID_HEIGHT as i32) as usize ;
        
        Some(self[wrapped_x][wrapped_y])
    }

    fn set_cell(&mut self, x: i32, y: i32, val: u32){
        let wrapped_x = ((x % GRID_WIDTH as i32 + GRID_WIDTH as i32) % GRID_WIDTH as i32) as usize;
        let wrapped_y = ((y % GRID_HEIGHT as i32 + GRID_HEIGHT as i32) % GRID_HEIGHT as i32) as usize ;
        
        self[wrapped_x][wrapped_y] = val;
    }

    // implementa las reglas de GOL de Conway y actualiza el estado de la cuadrícula

    fn update_generation(&mut self){
        
         let mut neighbor_count = empty_grid();

        for x in 0..GRID_WIDTH as i32 {
            for y in 0..GRID_HEIGHT as i32{
                for dx in x - 1..=x + 1{
                    for dy in y - 1..=y + 1 {
                        if let Some(val) = self.get_cell(dx, dy) {
                            neighbor_count[x as usize][y as usize] += val ;
                        }
                    }
                }

                neighbor_count[x as usize][y as usize] -= self.get_cell(x, y).unwrap();
            }
        }

        for x in 0..GRID_WIDTH {

            for y in 0..GRID_HEIGHT{

                if self[x][y] == 1{
                    if neighbor_count[x][y] < 2 || neighbor_count[x][y] > 3{
                        self[x][y] = 0 ;
                    }
                } else if neighbor_count[x][y] == 3{
                    self[x][y] = 1;
                }
            }
        }
    }

    fn copy_from(&mut self, source: &CellGrid){

        for x in 0..GRID_WIDTH {

            for y in 0..GRID_HEIGHT {
                self[x][y] = source[x][y] ;
            }
        }
    }
}

// --------------------------------------------------------------------------------
fn main() -> Result<(), eframe::Error> {
    let win_options = eframe::NativeOptions::default() ;
     eframe::run_native("Game Of Life" , win_options,  Box::new(|_cc| Box::<GameOfLife>::default()))
}

struct GameOfLife {
    grid: CellGrid,
    running: bool ,
     last_update: Option<Instant>,
    cell_size: f32,
}

impl eframe::App for GameOfLife{

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame){

        let Self {
            grid ,
            running,
            last_update,
            cell_size ,
        } =self;

        if *running {
            if let Some(instant) = last_update {
                if instant.elapsed().as_secs_f32() > 0.1 {
                    grid.update_generation();
                    *last_update = Some(Instant::now()) ;
                }
                
            } else{ *last_update = Some(Instant::now());
            }
        }

        ctx.style_mut(|style|{

            style.spacing.button_padding = egui::vec2(7.0, 7.0);
            style.text_styles.insert(

                 egui::TextStyle::Button,
                 egui::FontId::new(15.0, egui::FontFamily::Proportional) ,
            );
        });

        egui::CentralPanel::default().show(ctx, |ui|{
            ui.horizontal(|ui| {

                if ui.button(if *running { "Pause" } else { "Start" }).clicked(){
                    *running = !*running;
                    *last_update = if *running { Some(Instant::now()) } else { None };
                }

                if ui.button("Randomize").clicked(){
                    *grid = random_grid();
                }

                if ui.button("Pew Pew").clicked(){
                    load_gosper_gun(grid, 10, 10);
                }

                if ui.button("Clear").clicked() {
                    *grid = empty_grid();
                }
            });


            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui|{

                let (response, painter) = ui.allocate_painter(
                    egui::Vec2 {
                        x: GRID_WIDTH as f32 * *cell_size,
                        y: GRID_HEIGHT as f32 * *cell_size,
                    },

                    egui::Sense::click_and_drag(),
                );

                let rect = response.rect;

                if response.dragged() || response.clicked(){

                    if let Some(pos) = response.interact_pointer_pos(){

                        let x = ((pos.x - rect.min.x) / *cell_size) as i32;
                        let y = ((pos.y - rect.min.y) / *cell_size) as i32;

                        let value = if response.clicked_by(egui::PointerButton::Primary)
                            || response.dragged_by(egui::PointerButton::Primary)
                        {
                            1
                        } else{ 0
                        };

                        grid.set_cell(x, y, value);
                    }
                }

                let background = ui.visuals().extreme_bg_color ;
                painter.rect_filled(rect, 0.0, background);
                let mut mesh = egui::Mesh::default() ;

                for x in 0..GRID_WIDTH{

                    for y in 0..GRID_HEIGHT{
                        let color = get_color(grid[x][y]);
                        if color != egui::Color32::BLACK {

                            let cell_rect = egui::Rect::from_min_max(
                                rect.min + egui::Vec2::new(x as f32, y as f32) * *cell_size,
                                rect.min + egui::Vec2::new((x + 1) as f32, (y + 1) as f32) * *cell_size,
                            );
                            mesh.add_colored_rect(cell_rect, color);
                        }
                    }
                }

                painter.add(egui::Shape::mesh(mesh));
            });
        });
    }
}

impl Default for GameOfLife{

    fn default() -> Self{
        Self {
            grid: empty_grid(),
            running: false,
            last_update: None,
            cell_size: INITIAL_CELL_SIZE,
        }
    }
}


// pone un patrón predefinido → el cañón de deslizadores de Gosper
// generación automática

fn load_gosper_gun(grid: &mut CellGrid, offset_x: usize, offset_y: usize) {
    let cells = [
        (1, 5), (1, 6), (2, 5), (2, 6),
        (11, 5), (11, 6), (11, 7),
        (12, 4), (12, 8),
        (13, 3), (13, 9),
        (14, 3), (14, 9),
        (15, 6),
        (16, 4), (16, 8),
        (17, 5), (17, 6), (17, 7),
        (18, 6),
        (21, 3), (21, 4), (21, 5),
        (22, 3), (22, 4), (22, 5),
        (23, 2), (23, 6),
        (25, 1), (25, 2), (25, 6), (25, 7),
        (35, 3), (35, 4),
        (36, 3), (36, 4),
    ];

    for (dx, dy) in cells.iter() {
        let x = offset_x + dx;
        let y = offset_y + dy;
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            grid[x][y] = 1;
        }
    }
}

fn get_color(val: u32) -> egui::Color32 {

    match val {
        1 => egui::Color32::from_rgb(0, 255, 0),
        _ => egui::Color32::BLACK ,
    }
}
