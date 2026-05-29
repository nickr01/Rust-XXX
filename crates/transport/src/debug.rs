// needed for traits
use plotters::{backend::RGBPixel, prelude::*};

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

struct BufferWrapper(Vec<u32>);

impl Borrow<[u8]> for BufferWrapper {
    fn borrow(&self) -> &[u8] {
        // Safe for alignment: align_of(u8) <= align_of(u32)
        // Safe for cast: u32 can be thought of as being transparent over [u8; 4]
        unsafe {
            std::slice::from_raw_parts(
                self.0.as_ptr() as *const u8,
                self.0.len() * 4
            )
        }
    }
}

impl BorrowMut<[u8]> for BufferWrapper {
    fn borrow_mut(&mut self) -> &mut [u8] {
        // Safe for alignment: align_of(u8) <= align_of(u32)
        // Safe for cast: u32 can be thought of as being transparent over [u8; 4]
        unsafe {
            std::slice::from_raw_parts_mut(
                self.0.as_mut_ptr() as *mut u8,
                self.0.len() * 4
            )
        }
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

pub fn plot_spectrogram_to_buffer(
    spectrogram: &[waterfall::WflDataType],
    width: usize, height: usize,
) {    
    let mut buf = BufferWrapper(vec![0u32; width * height]);

    let bitmap_backend = BitMapBackend::with_buffer(buf.borrow_mut(), (width as u32, height as u32));
    let drawing_area= bitmap_backend.into_drawing_area();

    let spectrogram_cells = drawing_area.split_evenly((height, width));

    let windows_scaled = spectrogram.iter().map(|i| *i as f32).collect::<Vec<f32>>();
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

    drawing_area.present().expect("Cannot present the drawing area");

}

pub fn _plot_spectrogram_to_file(
	path: &str,
    spectrogram: &[waterfall::WflDataType],
    width: usize,
    height: usize,
) {
	let drawing_area= BitMapBackend::new(path, (width as u32, height as u32)).into_drawing_area();
    let spectrogram_cells = drawing_area.split_evenly((height, width));

    let windows_scaled = spectrogram.iter().map(|i| *i as f32).collect::<Vec<f32>>();
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
    x_min: usize, x_max: usize,
    y_min: f32, y_max: f32,
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
