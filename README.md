# pdf2image

This crate is a modified version of https://github.com/styrowolf/pdf2image with some changes that make it easier and cheaper to render single pages and using async instead of rayon for rendering multiple pages
and loading pdf data

A simplified port of Python's [`pdf2image`](https://github.com/Belval/pdf2image/) that wraps `pdftoppm`and `pdftocairo` (part of [poppler](https://poppler.freedesktop.org/)) to convert PDFs to `image::DynamicImage`s.

## Installation

Add to your project: `cargo add pdf2image_alt`

`pdf2image` requires `poppler` to be installed.

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