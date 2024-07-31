use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::error::{PDF2ImageError, Result};
use crate::render_options::RenderOptions;

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
    Specific(Vec<u32>),
}

/// Renders the PDF to images.
pub fn render_pdf_single_page(
    data: &[u8],
    info: &PdfInfo,
    page: u32,
    options: &RenderOptions,
) -> Result<image::DynamicImage> {
    if info.encrypted && options.password.is_none() {
        return Err(PDF2ImageError::NoPasswordForEncryptedPDF);
    }

    let cli_options = options.to_cli_args();
    let image = render_page(data, page, &cli_options, options)?;

    Ok(image)
}

/// Renders the PDF to images.
pub fn render_pdf_multi_page(
    data: &[u8],
    info: &PdfInfo,
    pages: Pages,
    options: &RenderOptions,
) -> Result<Vec<image::DynamicImage>> {
    if info.encrypted && options.password.is_none() {
        return Err(PDF2ImageError::NoPasswordForEncryptedPDF);
    }

    let valid_range = 0..=info.page_count;

    let pages_range: Vec<u32> = match pages {
        Pages::All => valid_range.collect(),
        Pages::Range(range) => range // Filter only valid pages
            .filter(|value| valid_range.contains(value))
            .collect(),
        Pages::Specific(pages) => pages // Filter only valid pages
            .into_iter()
            .filter(|value| valid_range.contains(value))
            .collect(),
    };

    let cli_options = options.to_cli_args();

    pages_range
        .par_iter()
        .map(|page| render_page(data, *page, &cli_options, options))
        .collect()
}

/// Renders a specific page from the pdf file
fn render_page(
    data: &[u8],
    page: u32,
    cli_options: &[String],
    options: &RenderOptions,
) -> Result<image::DynamicImage> {
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

    let mut child = Command::new(&executable)
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

/// Determines the executable path for the provided command
pub fn get_executable_path(command: &str) -> String {
    if let Ok(poppler_path) = std::env::var("PDF2IMAGE_POPPLER_PATH") {
        #[cfg(target_os = "windows")]
        return format!("{}\\{}.exe", poppler_path, command);
        #[cfg(not(target_os = "windows"))]
        return format!("{}/{}", poppler_path, command);
    }

    #[cfg(target_os = "windows")]
    return format!("{}.exe", command);

    #[cfg(not(target_os = "windows"))]
    return command.to_string();
}

pub fn extract_pdf_info(pdf: &[u8]) -> Result<(u32, bool)> {
    let mut child = Command::new(get_executable_path("pdfinfo"))
        .args(["-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // UNWRAP SAFETY: The child process is guaranteed to have a stdin as .stdin(Stdio::piped()) was called
    child.stdin.as_mut().unwrap().write_all(pdf)?;
    let output = child.wait_with_output()?;
    let mut splits = output.stdout.split(|&x| x == b'\n');

    let page_count: u32 = splits
        .clone()
        .find(|line| line.starts_with(b"Pages:"))
        .map(|line| {
            let line = std::str::from_utf8(line)?;
            let pg_str = line
                .split_whitespace()
                .last()
                .ok_or(PDF2ImageError::UnableToExtractPageCount)?;
            pg_str
                .parse::<u32>()
                .map_err(|_| PDF2ImageError::UnableToExtractPageCount)
        })
        .ok_or(PDF2ImageError::UnableToExtractPageCount)??;

    let encrypted = splits
        .find(|line| line.starts_with(b"Encrypted:"))
        .map(|line| {
            let line = std::str::from_utf8(line)?;
            Ok(
                match line
                    .split_whitespace()
                    .last()
                    .ok_or(PDF2ImageError::UnableToExtractEncryptionStatus)?
                {
                    "yes" => true,
                    "no" => false,
                    _ => return Err(PDF2ImageError::UnableToExtractEncryptionStatus),
                },
            )
        })
        .ok_or(PDF2ImageError::UnableToExtractEncryptionStatus)??;

    Ok((page_count, encrypted))
}
