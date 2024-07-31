use image::DynamicImage;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::error::{PDF2ImageError, Result};
use crate::render_options::RenderOptions;
use crate::utils::{extract_pdf_info, get_executable_path};

pub struct PdfInfo {
    /// The page count within the pdf
    page_count: u32,
    /// Whether the PDF is encrypted
    encrypted: bool,
}

impl PdfInfo {
    /// Returns the number of pages in the PDF.
    pub fn page_count(&self) -> u32 {
        self.page_count
    }

    /// Returns whether the PDF is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.encrypted
    }
}

impl TryFrom<&[u8]> for PdfInfo {
    type Error = PDF2ImageError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let (page_count, encrypted) = extract_pdf_info(value)?;

        Ok(Self {
            page_count,
            encrypted,
        })
    }
}

#[derive(Debug, Clone)]
/// Specifies which pages to render
pub enum Pages {
    All,
    Range(std::ops::RangeInclusive<u32>),
    Single(u32),
}

/// Renders the PDF to images.
pub fn render_pdf(
    data: &[u8],
    info: &PdfInfo,
    pages: Pages,
    options: impl Into<Option<RenderOptions>>,
) -> Result<Vec<image::DynamicImage>> {
    let pages_range: Vec<_> = match pages {
        Pages::Range(range) => range
            .filter(|page| {
                if *page > info.page_count || *page < 1 {
                    //eprintln!("Page {} does not exist in the PDF.", page);
                    false
                } else {
                    true
                }
            })
            .collect(),
        Pages::All => (0..=info.page_count).collect(),
        Pages::Single(page) => (page..page + 1).collect(),
    };

    let options: RenderOptions = options.into().unwrap_or_default();

    if info.encrypted && options.password.is_none() {
        return Err(PDF2ImageError::NoPasswordForEncryptedPDF);
    }

    let cli_options = options.to_cli_args();

    let executable = get_executable_path(if options.pdftocairo {
        "pdftocairo"
    } else {
        "pdftoppm"
    });

    let poppler_args: &[&str] = if options.pdftocairo {
        &["-", "-", "-jpeg", "-singlefile"]
    } else {
        &["-jpeg", "-singlefile"]
    };

    let images_results: Vec<Result<DynamicImage>> = pages_range
        .par_iter()
        .map(|page| render_page(data, *page, &executable, poppler_args, &cli_options))
        .collect();

    let mut images = Vec::with_capacity(images_results.len());

    for image in images_results {
        match image {
            Ok(image) => images.push(image),
            Err(e) => return Err(e),
        }
    }

    Ok(images)
}

/// Renders a specific page from the pdf file
fn render_page(
    data: &[u8],
    page: u32,
    executable: &str,
    poppler_args: &[&str],
    cli_options: &[String],
) -> Result<image::DynamicImage> {
    let mut child = Command::new(executable)
        // Add the poppler args
        .args(poppler_args)
        // Add the page args
        .args([
            "-f".to_string(),
            format!("{page}"),
            "-l".to_string(),
            format!("{page}"),
        ])
        // Add the cli options
        .args(cli_options)
        // Pipe input and output for use
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    // UNWRAP SAFETY: The child process is guaranteed to have a stdin as .stdin(Stdio::piped()) was called
    child.stdin.as_mut().unwrap().write_all(data)?;

    let output = child.wait_with_output()?;
    let image = image::load_from_memory_with_format(&output.stdout, image::ImageFormat::Jpeg)?;

    Ok(image)
}
