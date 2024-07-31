use pdf2image::{render_pdf, PDF2ImageError, PdfInfo, RenderOptionsBuilder};

fn main() -> Result<(), PDF2ImageError> {
    let data = std::fs::read("examples/pdfs/ropes.pdf").unwrap();
    let pdf_info = PdfInfo::try_from(data.as_slice()).unwrap();
    let pages = render_pdf(
        &data,
        &pdf_info,
        pdf2image::Pages::Range(1..=8),
        RenderOptionsBuilder::default().pdftocairo(true).build()?,
    );
    println!("{:?}", pages.unwrap().len());

    Ok(())
}
