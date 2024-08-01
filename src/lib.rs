//! # Overview
//!
//! This crate is a modified version of https://github.com/styrowolf/pdf2image with some changes that make it easier and cheaper to
//! render single pages
//!
//! It requires `poppler` to be installed on your system. You can follow the instructions found in the [README.md file which is most easily viewed on
//! github](https://github.com/jacobtread/pdf2image/blob/main/README.md).
//!
//! ## Quick Start
//!
//! ```rust
//! use pdf2image_alt::{render_pdf_multi_page, PDF2ImageError, PdfInfo, RenderOptionsBuilder};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), PDF2ImageError> {
//!     let data = std::fs::read("examples/pdfs/ropes.pdf").unwrap();
//!     let pdf_info = PdfInfo::read(data.as_slice()).await.unwrap();
//!     let options = RenderOptionsBuilder::default().pdftocairo(true).build()?;
//!     let pages = render_pdf_multi_page(
//!         &data,
//!         &pdf_info,
//!         pdf2image_alt::Pages::Range(1..=8),
//!         &options,
//!     )
//!     .await
//!     .unwrap();
//!     println!("{:?}", pages.len());
//!
//!     Ok(())
//! }
//! ```

mod error;
mod pdf;
mod render_options;

pub use error::{PDF2ImageError, Result};
pub use pdf::{render_pdf_multi_page, render_pdf_single_page, Pages, PdfInfo};
pub use render_options::{Crop, Password, RenderOptions, RenderOptionsBuilder, Scale, DPI};

// re-export image crate
pub use image;
