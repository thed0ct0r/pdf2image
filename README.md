# pdf2image


Provides functions for rendering a single page and one for rendering multiple pages

A simplified port of Python's [`pdf2image`](https://github.com/Belval/pdf2image/) that wraps `pdftoppm` and `pdftocairo` (part of [poppler](https://poppler.freedesktop.org/)) to convert PDFs to `image::DynamicImage`s.

This library is a fork of https://github.com/styrowolf/pdf2image that replaces the usages of blocking multithreaded (rayon) with tokio async rendering. Which itself is a port of the python [pdf2image](https://github.com/Belval/pdf2image/) library.

It wraps `pdftoppm` and `pdftocairo` (part of [Poppler](https://poppler.freedesktop.org/)) under the hood, uses the "pdfinfo" from poppler to determine basic info about the pdf (number of pages and whether its encrypted)

This fork uses async rendering instead and allows the rendering of a single page or multiple pages with separate functions.

> [!INFO]
> You must have poppler installed on your system in order to use 
> this program it depends on the pdfinfo and 

## Installation

`pdf2image` requires `poppler` to be installed.

```sh
cargo add pdf2image_alt
```

### Windows

Windows users will have to build or download `poppler` for Windows. Python's `pdf2image` maintainer recommends [@oschwartz10612 version](https://github.com/oschwartz10612/poppler-windows/releases/). You will then have to add the `bin/` folder to [PATH](https://www.architectryan.com/2018/03/17/add-to-the-path-on-windows-10/) or use the environment variable `PDF2IMAGE_POPPLER_PATH`.

### macOS

using [homebrew](https://brew.sh):

`brew install poppler`

### Linux

Most distros ship with `pdftoppm` and `pdftocairo`. If they are not installed, refer to your package manager to install `poppler-utils`

### Platform-independent (Using `conda`)

1. Install `poppler`: `conda install -c conda-forge poppler`
2. Install `pdf2image`: `pip install pdf2image`

## Quick Start

```rust
use pdf2image_alt::{render_pdf_multi_page, PDF2ImageError, PdfInfo, RenderOptionsBuilder};

#[tokio::main]
async fn main() -> Result<(), PDF2ImageError> {
    let data = std::fs::read("examples/pdfs/ropes.pdf").unwrap();
    let pdf_info = PdfInfo::read(data.as_slice()).await.unwrap();
    let options = RenderOptionsBuilder::default().pdftocairo(true).build()?;
    let pages = render_pdf_multi_page(
        &data,
        &pdf_info,
        pdf2image_alt::Pages::Range(1..=8),
        &options,
    )
    .await
    .unwrap();
    println!("{:?}", pages.len());

    Ok(())
}
```

## License

`pdf2image` includes code derived from [Edouard Belval](https://github.com/Belval/)'s [`pdf2image`](https://github.com/Belval/pdf2image) Python module, which is MIT licensed. Similarly, `pdf2image` is also licensed under the MIT License.