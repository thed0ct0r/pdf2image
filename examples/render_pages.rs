use pdf2image_alt::{render_pdf_multi_page, PDF2ImageError, PdfInfo, RenderOptionsBuilder};

fn main() -> Result<(), PDF2ImageError> {
    let data = std::fs::read("examples/pdfs/ropes.pdf").unwrap();
    let pdf_info = PdfInfo::try_from(data.as_slice()).unwrap();
    let options = RenderOptionsBuilder::default().pdftocairo(true).build()?;
    let pages = render_pdf_multi_page(
        &data,
        &pdf_info,
        pdf2image_alt::Pages::Range(1..=8),
        &options,
    );
    println!("{:?}", pages.unwrap().len());

    Ok(())
}
