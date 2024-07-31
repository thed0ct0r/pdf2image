//! # Overview
//! This crate is a simplified port of Python's [`pdf2image`](https://github.com/Belval/pdf2image/) that wraps `pdftoppm` and `pdftocairo` (part of [poppler](https://poppler.freedesktop.org/))
//! to convert PDFs to `image::DynamicImage`s.
//!
//! It requires `poppler` to be installed on your system. You can follow the instructions found in the [README.md file which is most easily viewed on
//! github](https://github.com/styrowolf/pdf2image/blob/main/README.md).
//!
//! ## Quick Start
//!
//! ```rust
//! use pdf2image::{PDF2ImageError, RenderOptionsBuilder, PdfInfo, render_pdf};
//!
//! fn main() -> Result<(), PDF2ImageError> {
//!     let data = std::fs::read("examples/pdfs/ropes.pdf").unwrap();
//!     let pdf_info = PdfInfo::try_from(data.as_slice()).unwrap();
//!     let pages = render_pdf(
//!         &data,
//!         &pdf_info,
//!         pdf2image::Pages::Range(1..=8),
//!         RenderOptionsBuilder::default().build()?,
//!     );
//!     println!("{:?}", pages.unwrap().len());
//!
//!     Ok(())
//! }
//! ```
mod error;
mod pdf;
mod render_options;
mod utils;

pub use error::{PDF2ImageError, Result};
pub use pdf::{render_pdf, Pages, PdfInfo};
pub use render_options::{Crop, Password, RenderOptions, RenderOptionsBuilder, Scale, DPI};

// re-export image crate
pub use image;
