#![doc = include_str!("../README.md")]

mod error;
mod pdf;
mod render_options;

pub use error::{PDF2ImageError, Result};
pub use pdf::{
    pdftext_all_pages, pdftext_multi_page, pdftext_single_page, render_pdf_multi_page,
    render_pdf_single_page, Pages, PdfInfo,
};
pub use render_options::{Crop, Password, RenderOptions, RenderOptionsBuilder, Scale, DPI};

// re-export image crate
pub use image;
