// needed for traits
// use plotters::backend::RGBPixel;
use plotters::backend::BGRXPixel;
use plotters::prelude::*;
// use plotters_bitmap::bitmap_pixel::BGRXPixel;
// use plotters_bitmap::BitMapBackend;

use crate::waterfall;

// use as crate::ddbg!(xxx);
// alternative to dbg! which does not print on production
// https://medium.com/@trivajay259/til-dbg-prints-the-variable-name-and-more-stop-typing-println-x-x-like-a-caveman-c60faa840af8

#[macro_export]
macro_rules! ddbg {
    ($($expr:expr),+ $(,)?) => {
        {
            #[cfg(debug_assertions)]
            {
                // Expand to dbg! in debug mode
                dbg!($($expr),+)
            }
            #[cfg(not(debug_assertions))]
            {
                // In release: evaluate and return without printing
                // Return last value or a tuple like dbg! does
                ($($expr),+)
            }
        }
    };
}

use std::borrow::{Borrow, BorrowMut};

pub struct BufferWrapper(Vec<u32>);

impl Borrow<[u8]> for BufferWrapper {
    fn borrow(&self) -> &[u8] {
        // Safe for alignment: align_of(u8) <= align_of(u32)
        // Safe for cast: u32 can be thought of as being transparent over [u8; 4]
        unsafe { std::slice::from_raw_parts(self.0.as_ptr() as *const u8, self.0.len() * 4) }
    }
}

impl BorrowMut<[u8]> for BufferWrapper {
    fn borrow_mut(&mut self) -> &mut [u8] {
        // Safe for alignment: align_of(u8) <= align_of(u32)
        // Safe for cast: u32 can be thought of as being transparent over [u8; 4]
        unsafe { std::slice::from_raw_parts_mut(self.0.as_mut_ptr() as *mut u8, self.0.len() * 4) }
    }
}

impl Borrow<[u32]> for BufferWrapper {
    fn borrow(&self) -> &[u32] {
        self.0.as_slice()
    }
}

impl BorrowMut<[u32]> for BufferWrapper {
    fn borrow_mut(&mut self) -> &mut [u32] {
        self.0.as_mut_slice()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DrawSize {
    pub width: usize,
    pub height: usize,
}

pub struct DebugWindow {
    window: minifb::Window,
    pub draw_size: DrawSize,
}

impl DebugWindow {
    pub fn new(draw_size: DrawSize) -> DebugWindow {
        dbg!(&draw_size);
        let mut window = minifb::Window::new(
            "RusXXX - ESC to exit",
            draw_size.width,
            draw_size.height,
            minifb::WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
        window.set_target_fps(20);
        DebugWindow { window, draw_size }
    }
    pub fn continue_run(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(minifb::Key::Escape)
    }
}
pub struct DebugPortal {
    debug_window: DebugWindow,
    buf: BufferWrapper,
}

impl DebugPortal {
    pub fn new(debug_window: DebugWindow) -> DebugPortal {
        let bufsize = debug_window.draw_size.width * debug_window.draw_size.height;
        dbg!(bufsize);

        let buf = BufferWrapper(vec![0u32; bufsize]);

        DebugPortal { debug_window, buf }
    }

    pub fn continue_run(&self) -> bool {
        self.debug_window.continue_run()
    }

    pub fn update(&mut self) {
        // dbg!("update debug portal");
        self.debug_window
            .window
            .update_with_buffer(
                self.buf.borrow(),
                self.debug_window.draw_size.width,
                self.debug_window.draw_size.height,
            )
            .expect("Cannot update debug window");
    }

    pub fn _buf(&self) -> &BufferWrapper {
        &self.buf
    }

    pub fn buf_as_mut(&mut self) -> &mut BufferWrapper {
        &mut self.buf
    }
}

pub fn plot_spectrogram_to_buffer(
    buf: &mut BufferWrapper,
    spectrogram: &[waterfall::WflDataType],
    draw_size: DrawSize,
) {
    // dbg!(&draw_size);
    let bitmap_backend = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
        buf.borrow_mut(),
        (draw_size.width as u32, draw_size.height as u32),
    )
    .expect("Cannot set up BitMap backend");
    let drawing_area = bitmap_backend.into_drawing_area();

    let spectrogram_cells = drawing_area.split_evenly((draw_size.height, draw_size.width));

    let windows_scaled = spectrogram.to_vec(); // iter().copied().collect::<Vec<f32>>();
    let highest_spectral_density = windows_scaled
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .expect("Cannot calc spectrogram density");
    let color_scale = colorous::MAGMA;

    for (cell, spectral_density) in spectrogram_cells.iter().zip(windows_scaled.iter()) {
        let spectral_density_scaled = spectral_density / highest_spectral_density;
        let color = color_scale.eval_continuous(spectral_density_scaled as f64);
        cell.fill(&RGBColor(color.r, color.g, color.b))
            .expect("Cannot plot spectrogram");
    }

    drawing_area
        .present()
        .expect("Cannot present the drawing area");
}

pub fn _plot_spectrogram_to_file(
    path: &str,
    spectrogram: &[waterfall::WflDataType],
    width: usize,
    height: usize,
) {
    let drawing_area = BitMapBackend::new(path, (width as u32, height as u32)).into_drawing_area();
    let spectrogram_cells = drawing_area.split_evenly((height, width));

    let windows_scaled = spectrogram.to_vec(); // iter().copied().collect::<Vec<f32>>();
    let highest_spectral_density = windows_scaled
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .expect("Cannot calc spectrogram density");
    let color_scale = colorous::MAGMA;

    for (cell, spectral_density) in spectrogram_cells.iter().zip(windows_scaled.iter()) {
        let spectral_density_scaled = spectral_density / highest_spectral_density;
        let color = color_scale.eval_continuous(spectral_density_scaled as f64);
        cell.fill(&RGBColor(color.r, color.g, color.b))
            .expect("Cannot plot spectrogram");
    }
    drawing_area.present().unwrap(); // added - necessary?
}

pub fn _plot_graph(
    path: &str,
    caption: &str,
    values: &[f32],
    x_min: usize,
    x_max: usize,
    y_min: f32,
    y_max: f32,
) {
    let root = BitMapBackend::new(path, (1024, 1000)).into_drawing_area();

    root.fill(&WHITE).unwrap();

    let font = ("sans-serif", 20);

    let mut chart = ChartBuilder::on(&root)
        .caption(caption, font.into_font())
        .margin(10)
        .x_label_area_size(20)
        .y_label_area_size(20)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .expect("Cannot plot chart");

    chart.configure_mesh().draw().unwrap();
    let line_series = LineSeries::new((0..).zip(values.iter()).map(|(idx, y)| (idx, *y)), &RED);
    chart.draw_series(line_series).unwrap();
}
