use futures::{future::BoxFuture, stream::FuturesOrdered, TryStreamExt};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::error::{PDF2ImageError, Result};
use crate::render_options::RenderOptions;

pub struct PdfInfo {
    /// The page count within the pdf
    page_count: u32,
    /// Whether the PDF is encrypted
    encrypted: bool,
}

impl PdfInfo {
    pub async fn read(data: &[u8]) -> Result<Self> {
        let (page_count, encrypted) = extract_pdf_info(data).await?;

        Ok(Self {
            page_count,
            encrypted,
        })
    }

    /// Returns the number of pages in the PDF.
    pub fn page_count(&self) -> u32 {
        self.page_count
    }

    /// Returns whether the PDF is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.encrypted
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
pub async fn render_pdf_single_page<'data, 'options: 'data>(
    data: &'data [u8],
    info: &'options PdfInfo,
    page: u32,
    options: &'options RenderOptions,
) -> Result<image::DynamicImage> {
    if info.encrypted && options.password.is_none() {
        return Err(PDF2ImageError::NoPasswordForEncryptedPDF);
    }

    let image = render_page(data, page, options).await?;

    Ok(image)
}

/// Renders the PDF to images.
pub async fn render_pdf_multi_page<'data, 'options: 'data>(
    data: &'data [u8],
    info: &'options PdfInfo,
    pages: Pages,
    options: &'options RenderOptions,
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

    pages_range
        .into_iter()
        .map(|page| -> BoxFuture<'data, Result<image::DynamicImage>> {
            Box::pin(render_page(data, page, options))
        })
        .collect::<FuturesOrdered<BoxFuture<'data, Result<image::DynamicImage>>>>()
        .try_collect()
        .await
}

/// Renders a specific page from the pdf file
async fn render_page<'data, 'options: 'data>(
    data: &'data [u8],
    page: u32,
    options: &'options RenderOptions,
) -> Result<image::DynamicImage> {
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
        .spawn()?;

    // UNWRAP SAFETY: The child process is guaranteed to have a stdin as .stdin(Stdio::piped()) was called
    child.stdin.as_mut().unwrap().write_all(data).await?;

    let output = child.wait_with_output().await?;
    let image = image::load_from_memory_with_format(&output.stdout, image::ImageFormat::Jpeg)?;

    Ok(image)
}

/// Extracts the text contents of a pdf file from a single page
pub async fn pdftext_single_page<'data, 'options: 'data>(
    data: &'data [u8],
    info: &'options PdfInfo,
    page: u32,
    options: &'options RenderOptions,
) -> Result<String> {
    if info.encrypted && options.password.is_none() {
        return Err(PDF2ImageError::NoPasswordForEncryptedPDF);
    }

    let image = render_page_text(data, page, options).await?;

    Ok(image)
}

/// Extracts the text contents of a pdf file from multiple page
///
/// If you want all pages as a string it will likely be more performant
/// to use [pdftext_multi_page]
pub async fn pdftext_multi_page<'data, 'options: 'data>(
    data: &'data [u8],
    info: &'options PdfInfo,
    pages: Pages,
    options: &'options RenderOptions,
) -> Result<String> {
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

    pages_range
        .into_iter()
        .map(|page| -> BoxFuture<'data, Result<String>> {
            Box::pin(render_page_text(data, page, options))
        })
        .collect::<FuturesOrdered<BoxFuture<'data, Result<String>>>>()
        .try_collect()
        .await
}

/// Extracts the text contents of a pdf file from all pages as
/// one big string, use [pdftext_multi_page] to get a separate
/// string for each page
pub async fn pdftext_all_pages<'data, 'options: 'data>(
    data: &'data [u8],
    info: &'options PdfInfo,
    pages: Pages,
    options: &'options RenderOptions,
) -> Result<String> {
    if info.encrypted && options.password.is_none() {
        return Err(PDF2ImageError::NoPasswordForEncryptedPDF);
    }

    let valid_range = 0..=info.page_count;

    let pages_range: Vec<u32> = match pages {
        Pages::All => return render_all_pages_text(data, options).await,
        Pages::Range(range) => range // Filter only valid pages
            .filter(|value| valid_range.contains(value))
            .collect(),
        Pages::Specific(pages) => pages // Filter only valid pages
            .into_iter()
            .filter(|value| valid_range.contains(value))
            .collect(),
    };

    pages_range
        .into_iter()
        .map(|page| -> BoxFuture<'data, Result<String>> {
            Box::pin(render_page_text(data, page, options))
        })
        .collect::<FuturesOrdered<BoxFuture<'data, Result<String>>>>()
        .try_collect()
        .await
}

/// Renders a specific page from the pdf file
async fn render_page_text<'data, 'options: 'data>(
    data: &'data [u8],
    page: u32,
    options: &'options RenderOptions,
) -> Result<String> {
    let cli_options = options.to_cli_args();

    let mut child = Command::new("pdftotext")
        // Take input from stdin and provide to stdout
        .args(["-", "-"])
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
        .spawn()?;

    // UNWRAP SAFETY: The child process is guaranteed to have a stdin as .stdin(Stdio::piped()) was called
    child.stdin.as_mut().unwrap().write_all(data).await?;

    let output = child.wait_with_output().await?;
    let value = String::from_utf8_lossy(&output.stdout);

    Ok(value.into_owned())
}
/// Renders a specific page from the pdf file
async fn render_all_pages_text<'data, 'options: 'data>(
    data: &'data [u8],
    options: &'options RenderOptions,
) -> Result<String> {
    let cli_options = options.to_cli_args();

    let mut child = Command::new("pdftotext")
        // Take input from stdin and provide to stdout
        .args(["-", "-"])
        // Add the cli options
        .args(cli_options)
        // Pipe input and output for use
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // UNWRAP SAFETY: The child process is guaranteed to have a stdin as .stdin(Stdio::piped()) was called
    child.stdin.as_mut().unwrap().write_all(data).await?;

    let output = child.wait_with_output().await?;
    let value = String::from_utf8_lossy(&output.stdout);

    Ok(value.into_owned())
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

pub async fn extract_pdf_info(pdf: &[u8]) -> Result<(u32, bool)> {
    let mut child = Command::new(get_executable_path("pdfinfo"))
        .args(["-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // UNWRAP SAFETY: The child process is guaranteed to have a stdin as .stdin(Stdio::piped()) was called
    child.stdin.as_mut().unwrap().write_all(pdf).await?;
    let output = child.wait_with_output().await?;
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
