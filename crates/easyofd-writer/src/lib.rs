#![allow(clippy::format_push_string)]

//! # easyofd-writer
//!
//! OFD file writer that produces GB/T 33190-2016 compliant ZIP archives.
//!
//! ## Architecture
//!
//! The writer builds an in-memory ZIP archive with this structure:
//!
//! ```text
//! output.ofd (ZIP)
//! ├── OFD.xml                    ← entry point
//! └── Doc_0/
//!     ├── Document.xml           ← document structure
//!     ├── DocumentRes.xml        ← document resources (images, fonts)
//!     ├── Pages/
//!     │   ├── Page_0.xml         ← page content
//!     │   ├── Page_1.xml
//!     │   └── ...
//!     └── Res/                   ← embedded resources
//!         ├── Image_0.jpeg
//!         └── ...
//! ```

use std::io::{Cursor, Write};

use chrono::Utc;
use easyofd_core::{
    ContentObject, ImageFormat, OfdMetadata, OfdPage, OfdResult,
};
use zip::write::{FileOptions, ZipWriter};

// ─── Public API ──────────────────────────────────────────────────────────────

/// Write options for OFD generation.
#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// Document metadata.
    pub metadata: OfdMetadata,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            metadata: OfdMetadata {
                version: "1.0".to_string(),
                title: Some("EasyOFD Document".to_string()),
                author: Some("easyofd-rs".to_string()),
                creator: Some("easyofd-rs".to_string()),
                creation_date: Some(Utc::now().naive_utc()),
            },
        }
    }
}

/// The OFD writer. Collects pages and writes them to a ZIP archive.
pub struct OfdWriter {
    pages: Vec<OfdPage>,
    options: WriteOptions,
}

impl OfdWriter {
    /// Create a new OFD writer with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            options: WriteOptions::default(),
        }
    }

    /// Create a new OFD writer with custom options.
    #[must_use]
    pub fn with_options(options: WriteOptions) -> Self {
        Self {
            pages: Vec::new(),
            options,
        }
    }

    /// Add a page to the document.
    pub fn add_page(&mut self, page: OfdPage) {
        self.pages.push(page);
    }

    /// Add multiple pages to the document.
    pub fn add_pages(&mut self, pages: Vec<OfdPage>) {
        self.pages.extend(pages);
    }

    /// Build the OFD file and return the raw bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if ZIP creation fails.
    pub fn build(&self) -> OfdResult<Vec<u8>> {
        let cursor = Cursor::new(Vec::with_capacity(4096));
        let mut zip = ZipWriter::new(cursor);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        self.write_zip(&mut zip, &options)?;
        let cursor = zip.finish().map_err(zip_err)?;
        Ok(cursor.into_inner())
    }

    /// Build the OFD file and write it to a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if ZIP creation or file I/O fails.
    pub fn build_to_file(&self, path: impl AsRef<std::path::Path>) -> OfdResult<()> {
        let bytes = self.build()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    fn write_zip<W: Write + std::io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        options: &FileOptions,
    ) -> OfdResult<()> {
        // Collect all image resources across all pages.
        let mut image_resources: Vec<(String, Vec<u8>, ImageFormat)> = Vec::new();

        for page in &self.pages {
            for obj in &page.content {
                if let ContentObject::Image(img) = obj {
                    let ext = match img.format {
                        ImageFormat::Jpeg => "jpeg",
                        ImageFormat::Png => "png",
                        ImageFormat::Bmp => "bmp",
                        ImageFormat::Tiff => "tiff",
                    };
                    let res_name = format!("Doc_0/Res/Image_{}.{}", image_resources.len(), ext);
                    image_resources.push((res_name.clone(), img.data.clone(), img.format));
                }
            }
        }

        // 1. Write OFD.xml
        let ofd_xml = self.build_ofd_xml();
        zip.start_file("OFD.xml", *options)
            .map_err(zip_err)?;
        zip.write_all(ofd_xml.as_bytes()).map_err(io_err)?;

        // 2. Write Document.xml
        let doc_xml = self.build_document_xml(&image_resources);
        zip.start_file("Doc_0/Document.xml", *options)
            .map_err(zip_err)?;
        zip.write_all(doc_xml.as_bytes()).map_err(io_err)?;

        // 3. Write DocumentRes.xml
        let doc_res_xml = self.build_document_res_xml(&image_resources);
        zip.start_file("Doc_0/DocumentRes.xml", *options)
            .map_err(zip_err)?;
        zip.write_all(doc_res_xml.as_bytes()).map_err(io_err)?;

        // 4. Write each page
        for (i, page) in self.pages.iter().enumerate() {
            let page_xml = self.build_page_xml(page, i, &image_resources);
            zip.start_file(format!("Doc_0/Pages/Page_{i}.xml"), *options)
                .map_err(zip_err)?;
            zip.write_all(page_xml.as_bytes()).map_err(io_err)?;
        }

        // 5. Write image resources
        for (res_name, data, _) in &image_resources {
            zip.start_file(res_name, *options).map_err(zip_err)?;
            zip.write_all(data).map_err(io_err)?;
        }

        Ok(())
    }

    // ─── XML Generation ──────────────────────────────────────────────────────

    fn build_ofd_xml(&self) -> String {
        let mut xml = String::with_capacity(512);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&format!(
            r#"<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="{}">"#,
            self.options.metadata.version
        ));
        xml.push('\n');
        xml.push_str(r"  <ofd:DocBody>");
        xml.push('\n');
        xml.push_str(r"    <ofd:DocInfo>");
        xml.push('\n');

        if let Some(ref title) = self.options.metadata.title {
            xml.push_str(&format!("      <ofd:Title>{}</ofd:Title>", xml_escape(title)));
            xml.push('\n');
        }
        if let Some(ref author) = self.options.metadata.author {
            xml.push_str(&format!(
                "      <ofd:Author>{}</ofd:Author>",
                xml_escape(author)
            ));
            xml.push('\n');
        }
        if let Some(ref creator) = self.options.metadata.creator {
            xml.push_str(&format!(
                "      <ofd:Creator>{}</ofd:Creator>",
                xml_escape(creator)
            ));
            xml.push('\n');
        }
        if let Some(dt) = self.options.metadata.creation_date {
            xml.push_str(&format!(
                "      <ofd:CreationDate>{}</ofd:CreationDate>",
                dt.format("%Y-%m-%dT%H:%M:%S")
            ));
            xml.push('\n');
        }

        xml.push_str(r"    </ofd:DocInfo>");
        xml.push('\n');
        xml.push_str(r"    <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>");
        xml.push('\n');
        xml.push_str(r"  </ofd:DocBody>");
        xml.push('\n');
        xml.push_str(r"</ofd:OFD>");
        xml.push('\n');
        xml
    }

    fn build_document_xml(
        &self,
        image_resources: &[(String, Vec<u8>, ImageFormat)],
    ) -> String {
        let mut xml = String::with_capacity(1024);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">"#);
        xml.push('\n');

        // Common Data
        xml.push_str(r"  <ofd:CommonData>");
        xml.push('\n');

        // Page area: use first page dimensions, or A4 default
        let (pw, ph) = self.pages.first().map_or((210.0, 297.0), |p| (p.width, p.height));
        xml.push_str(&format!(
            r"    <ofd:PageArea><ofd:PhysicalBox>0 0 {pw:.2} {ph:.2}</ofd:PhysicalBox></ofd:PageArea>"
        ));
        xml.push('\n');

        // Font declarations
        xml.push_str(r"    <ofd:PublicRes>Doc_0/PublicRes.xml</ofd:PublicRes>");
        xml.push('\n');

        // Document resources
        if !image_resources.is_empty() {
            xml.push_str(
                r"    <ofd:DocumentRes>Doc_0/DocumentRes.xml</ofd:DocumentRes>",
            );
            xml.push('\n');
        }

        xml.push_str(r"  </ofd:CommonData>");
        xml.push('\n');

        // Pages
        xml.push_str(r"  <ofd:Pages>");
        xml.push('\n');
        for i in 0..self.pages.len() {
            xml.push_str(&format!(
                r#"    <ofd:Page ID="{id}" BaseLoc="Pages/Page_{i}.xml"/>"#,
                id = i + 1
            ));
            xml.push('\n');
        }
        xml.push_str(r"  </ofd:Pages>");
        xml.push('\n');

        xml.push_str(r"</ofd:Document>");
        xml.push('\n');
        xml
    }

    #[allow(clippy::unused_self)]
    fn build_document_res_xml(
        &self,
        image_resources: &[(String, Vec<u8>, ImageFormat)],
    ) -> String {
        let mut xml = String::with_capacity(512);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<ofd:DocumentRes xmlns:ofd="http://www.ofdspec.org/2016">"#);
        xml.push('\n');
        xml.push_str(r"  <ofd:MultiMedia>");
        xml.push('\n');

        for (i, (res_name, _, fmt)) in image_resources.iter().enumerate() {
            let type_str = match fmt {
                ImageFormat::Jpeg => "JPEG",
                ImageFormat::Png => "PNG",
                ImageFormat::Bmp => "BMP",
                ImageFormat::Tiff => "TIFF",
            };
            // The BaseLoc is relative to the Doc_0 directory.
            let relative = res_name.strip_prefix("Doc_0/").unwrap_or(res_name);
            xml.push_str(&format!(
                r#"    <ofd:MultiMedia ID="{}" Type="{}"><ofd:MediaFile>{}</ofd:MediaFile></ofd:MultiMedia>"#,
                100 + i,
                type_str,
                relative,
            ));
            xml.push('\n');
        }

        xml.push_str(r"  </ofd:MultiMedia>");
        xml.push('\n');
        xml.push_str(r"</ofd:DocumentRes>");
        xml.push('\n');
        xml
    }

    #[allow(clippy::unused_self)]
    fn build_page_xml(
        &self,
        page: &OfdPage,
        page_index: usize,
        image_resources: &[(String, Vec<u8>, ImageFormat)],
    ) -> String {
        let mut xml = String::with_capacity(2048);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&format!(
            r#"<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016" ID="{}">"#,
            page_index + 1
        ));
        xml.push('\n');

        // Page area
        xml.push_str(&format!(
            r"  <ofd:Area><ofd:PhysicalBox>0 0 {:.2} {:.2}</ofd:PhysicalBox></ofd:Area>",
            page.width, page.height
        ));
        xml.push('\n');

        // Content layer
        xml.push_str(r"  <ofd:Content>");
        xml.push('\n');

        // Collect image indices for this page.
        let mut image_counter = 0usize;

        for obj in &page.content {
            match obj {
                ContentObject::Text(text) => {
                    // mm to OFD units (1 mm = ~3.543307 pixels at 96dpi, but OFD uses mm directly)
                    let x = text.x;
                    let y = text.y;
                    // Estimate text width: ~0.3mm per character for 12pt SimSun (rough heuristic)
                    let est_width = text
                        .width
                        .unwrap_or(f64::from(u32::try_from(text.text.len()).unwrap_or(u32::MAX)) * text.size * 0.06);
                    let est_height = text.height.unwrap_or(text.size * 0.4);

                    xml.push_str(&format!(
                        r#"    <ofd:TextObject ID="t_{page_index}_{idx}" Boundary="{x:.2} {y:.2} {w:.2} {h:.2}" Font="{font}" Size="{size:.1}">"#,
                        idx = page_index * 1000 + image_counter,
                        w = est_width,
                        h = est_height,
                        font = text.font,
                        size = text.size,
                    ));
                    xml.push('\n');

                    // TextCode
                    xml.push_str(&format!(
                        r#"      <ofd:TextCode X="0" Y="{y:.2}">{text}</ofd:TextCode>"#,
                        y = text.size * 0.8,
                        text = xml_escape(&text.text),
                    ));
                    xml.push('\n');

                    xml.push_str(r"    </ofd:TextObject>");
                    xml.push('\n');
                }
                ContentObject::Image(img) => {
                    // Find the resource ID for this image.
                    let img_path = format!("Doc_0/Res/Image_{}.{}", image_counter,
                        match img.format {
                            ImageFormat::Jpeg => "jpeg",
                            ImageFormat::Png => "png",
                            ImageFormat::Bmp => "bmp",
                            ImageFormat::Tiff => "tiff",
                        }
                    );
                    let res_id = image_resources
                        .iter()
                        .position(|(name, _, _)| *name == img_path)
                        .map_or(100, |i| 100 + i);

                    xml.push_str(&format!(
                        r#"    <ofd:ImageObject ID="i_{page_index}_{idx}" Boundary="{x:.2} {y:.2} {w:.2} {h:.2}" ResourceID="{res_id}"/>"#,
                        idx = page_index * 1000 + image_counter,
                        x = img.x,
                        y = img.y,
                        w = img.width,
                        h = img.height,
                    ));
                    xml.push('\n');
                    image_counter += 1;
                }
                ContentObject::Path(path) => {
                    let stroke = format!("{:06X}", path.stroke_color);
                    xml.push_str(&format!(
                        r#"    <ofd:PathObject ID="p_{page_index}_{idx}" Boundary="{x:.2} {y:.2} 0 0" StrokeColor="{stroke}" LineWidth="{lw:.2}">"#,
                        idx = page_index * 1000 + image_counter,
                        x = path.x,
                        y = path.y,
                        lw = path.stroke_width,
                    ));
                    xml.push('\n');
                    xml.push_str(&format!(
                        r"      <ofd:AbbreviatedData>{}</ofd:AbbreviatedData>",
                        xml_escape(&path.path_data),
                    ));
                    xml.push('\n');
                    xml.push_str(r"    </ofd:PathObject>");
                    xml.push('\n');
                }
            }
        }

        xml.push_str(r"  </ofd:Content>");
        xml.push('\n');
        xml.push_str(r"</ofd:Page>");
        xml.push('\n');
        xml
    }
}

impl Default for OfdWriter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

#[allow(clippy::needless_pass_by_value)]
fn zip_err(e: zip::result::ZipError) -> easyofd_core::OfdError {
    easyofd_core::OfdError::Zip(format!("{e}"))
}

fn io_err(e: std::io::Error) -> easyofd_core::OfdError {
    easyofd_core::OfdError::Io(e)
}

/// Escape special XML characters.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{ImageObject, PathObject, TextObject};

    // ── WriteOptions ──────────────────────────────────────────────────────────

    #[test]
    fn test_write_options_default() {
        let opts = WriteOptions::default();
        assert_eq!(opts.metadata.version, "1.0");
        assert_eq!(opts.metadata.title.as_deref(), Some("EasyOFD Document"));
        assert_eq!(opts.metadata.author.as_deref(), Some("easyofd-rs"));
        assert_eq!(opts.metadata.creator.as_deref(), Some("easyofd-rs"));
        assert!(opts.metadata.creation_date.is_some());
    }

    #[test]
    fn test_write_options_clone_debug() {
        let opts = WriteOptions::default();
        let opts2 = opts.clone();
        assert_eq!(opts2.metadata.version, "1.0");
        assert!(format!("{opts:?}").contains("WriteOptions"));
    }

    // ── OfdWriter constructors ────────────────────────────────────────────────

    #[test]
    fn test_ofd_writer_new() {
        let w = OfdWriter::new();
        assert!(w.pages.is_empty());
        assert_eq!(w.options.metadata.version, "1.0");
    }

    #[test]
    fn test_ofd_writer_default() {
        let w = OfdWriter::default();
        assert!(w.pages.is_empty());
    }

    #[test]
    fn test_ofd_writer_with_options() {
        let mut opts = WriteOptions::default();
        opts.metadata.title = Some("Custom".into());
        let w = OfdWriter::with_options(opts);
        assert_eq!(w.options.metadata.title.as_deref(), Some("Custom"));
    }

    // ── add_page / add_pages ──────────────────────────────────────────────────

    #[test]
    fn test_add_page() {
        let mut w = OfdWriter::new();
        w.add_page(OfdPage::new(210.0, 297.0));
        assert_eq!(w.pages.len(), 1);
    }

    #[test]
    fn test_add_pages() {
        let mut w = OfdWriter::new();
        w.add_pages(vec![
            OfdPage::new(210.0, 297.0),
            OfdPage::new(297.0, 210.0),
        ]);
        assert_eq!(w.pages.len(), 2);
    }

    // ── build ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_build_empty() {
        let bytes = OfdWriter::new().build().unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_build_single_text_page() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Hello, OFD!"));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"OFD.xml".to_string()));
        assert!(names.contains(&"Doc_0/Document.xml".to_string()));
        assert!(names.contains(&"Doc_0/Pages/Page_0.xml".to_string()));
    }

    #[test]
    fn test_build_multi_page() {
        let mut w = OfdWriter::new();
        for i in 0..3 {
            let mut page = OfdPage::new(210.0, 297.0);
            page.add_text(TextObject::new(20.0, 30.0, format!("Page {i}")));
            w.add_page(page);
        }
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Pages/Page_0.xml".to_string()));
        assert!(names.contains(&"Doc_0/Pages/Page_1.xml".to_string()));
        assert!(names.contains(&"Doc_0/Pages/Page_2.xml".to_string()));
    }

    // ── build_to_file ─────────────────────────────────────────────────────────

    #[test]
    fn test_build_to_file() {
        let dir = std::env::temp_dir().join("easyofd_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_build_to_file.ofd");

        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "file test"));
        w.add_page(page);
        w.build_to_file(&path).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
        let _ = std::fs::remove_file(&path);
    }

    // ── Metadata variations ───────────────────────────────────────────────────

    #[test]
    fn test_ofd_xml_all_metadata() {
        let mut opts = WriteOptions::default();
        opts.metadata.title = Some("T".into());
        opts.metadata.author = Some("A".into());
        opts.metadata.creator = Some("C".into());
        let w = OfdWriter::with_options(opts);
        let xml = w.build_ofd_xml();
        assert!(xml.contains("<ofd:Title>T</ofd:Title>"));
        assert!(xml.contains("<ofd:Author>A</ofd:Author>"));
        assert!(xml.contains("<ofd:Creator>C</ofd:Creator>"));
        assert!(xml.contains("<ofd:CreationDate>"));
    }

    #[test]
    fn test_ofd_xml_no_optional_metadata() {
        let mut opts = WriteOptions::default();
        opts.metadata.title = None;
        opts.metadata.author = None;
        opts.metadata.creator = None;
        opts.metadata.creation_date = None;
        let w = OfdWriter::with_options(opts);
        let xml = w.build_ofd_xml();
        assert!(!xml.contains("<ofd:T"));
        assert!(!xml.contains("<ofd:Author>"));
        assert!(!xml.contains("<ofd:Creator>"));
        assert!(!xml.contains("<ofd:CreationDate>"));
    }

    // ── XML special chars in metadata ─────────────────────────────────────────

    #[test]
    fn test_ofd_xml_special_chars_in_title() {
        let mut opts = WriteOptions::default();
        opts.metadata.title = Some("A<B&C\"D'E".into());
        let w = OfdWriter::with_options(opts);
        let xml = w.build_ofd_xml();
        assert!(xml.contains("&lt;"));
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&quot;"));
        assert!(xml.contains("&apos;"));
    }

    // ── Document.xml variations ───────────────────────────────────────────────

    #[test]
    fn test_document_xml_with_images() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::jpeg(0.0, 0.0, 10.0, 10.0, vec![0xFF]));
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(bytes.windows(6).any(|w| w == b"Image_"));
    }

    #[test]
    fn test_document_xml_without_images() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "no images"));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        // No image resource files in the ZIP
        assert!(!names.iter().any(|n| n.contains("Image_")));
        // DocumentRes.xml is always present but has no MultiMedia entries
        assert!(names.contains(&"Doc_0/DocumentRes.xml".to_string()));
    }

    // ── DocumentRes.xml with all image formats ────────────────────────────────

    #[test]
    fn test_document_res_png() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::png(0.0, 0.0, 10.0, 10.0, vec![0x89]));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.png".to_string()));
    }

    #[test]
    fn test_document_res_bmp() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::new(0.0, 0.0, 10.0, 10.0, vec![0x42], ImageFormat::Bmp));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.bmp".to_string()));
    }

    #[test]
    fn test_document_res_tiff() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::new(0.0, 0.0, 10.0, 10.0, vec![0x49], ImageFormat::Tiff));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.tiff".to_string()));
    }

    // ── Page content: Text ────────────────────────────────────────────────────

    #[test]
    fn test_page_text_with_custom_size() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(
            TextObject::new(10.0, 20.0, "styled")
                .font("SimHei")
                .size(24.0)
                .bold()
                .italic()
                .color(0xFF_0000),
        );
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_page_text_with_explicit_width_height() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        let mut t = TextObject::new(10.0, 20.0, "sized");
        t.width = Some(100.0);
        t.height = Some(20.0);
        page.add_text(t);
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_page_text_special_chars() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "a<b&c\"d'e"));
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    // ── Page content: Image ───────────────────────────────────────────────────

    #[test]
    fn test_page_image_jpeg() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::jpeg(50.0, 50.0, 30.0, 30.0, vec![0xFF, 0xD8]));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.jpeg".to_string()));
    }

    #[test]
    fn test_page_multiple_images() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::jpeg(0.0, 0.0, 10.0, 10.0, vec![0xFF]));
        page.add_image(ImageObject::png(20.0, 0.0, 10.0, 10.0, vec![0x89]));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.jpeg".to_string()));
        assert!(names.contains(&"Doc_0/Res/Image_1.png".to_string()));
    }

    // ── Page content: Path ────────────────────────────────────────────────────

    #[test]
    fn test_page_path_hline() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(PathObject::hline(20.0, 40.0, 190.0));
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_page_path_rect_with_fill() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(
            PathObject::rect(10.0, 10.0, 100.0, 50.0)
                .stroke_color(0xFF_0000)
                .stroke_width(1.0)
                .fill_color(0x00_FF00),
        );
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_page_path_special_chars_in_data() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(PathObject::new(0.0, 0.0, "M0&0<10"));
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    // ── Mixed content on one page ─────────────────────────────────────────────

    #[test]
    fn test_page_mixed_content() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Invoice"));
        page.add_text(TextObject::new(20.0, 50.0, "$100.00"));
        page.add_image(ImageObject::jpeg(150.0, 30.0, 30.0, 30.0, vec![0xFF]));
        page.add_path(PathObject::hline(20.0, 45.0, 190.0));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.jpeg".to_string()));
        assert!(names.contains(&"Doc_0/Pages/Page_0.xml".to_string()));
    }

    // ── Page dimensions ───────────────────────────────────────────────────────

    #[test]
    fn test_custom_page_dimensions() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(297.0, 420.0); // A3
        page.add_text(TextObject::new(0.0, 0.0, "A3 page"));
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    // ── xml_escape ────────────────────────────────────────────────────────────

    #[test]
    fn test_xml_escape_empty() {
        assert_eq!(xml_escape(""), "");
    }

    #[test]
    fn test_xml_escape_no_special() {
        assert_eq!(xml_escape("hello world"), "hello world");
    }

    #[test]
    fn test_xml_escape_all_special() {
        assert_eq!(
            xml_escape("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }

    #[test]
    fn test_xml_escape_only_ampersand() {
        assert_eq!(xml_escape("&"), "&amp;");
    }

    #[test]
    fn test_xml_escape_only_lt() {
        assert_eq!(xml_escape("<"), "&lt;");
    }

    #[test]
    fn test_xml_escape_only_gt() {
        assert_eq!(xml_escape(">"), "&gt;");
    }

    #[test]
    fn test_xml_escape_only_quote() {
        assert_eq!(xml_escape("\""), "&quot;");
    }

    #[test]
    fn test_xml_escape_only_apos() {
        assert_eq!(xml_escape("'"), "&apos;");
    }

    // ── Error helpers ─────────────────────────────────────────────────────────

    #[test]
    fn test_zip_err() {
        let zip_err = zip::result::ZipError::FileNotFound;
        let err = super::zip_err(zip_err);
        assert!(format!("{err}").contains("ZIP error"));
    }

    #[test]
    fn test_io_err() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let err = super::io_err(io_err);
        assert!(format!("{err}").contains("I/O error"));
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Extract all entry names from a ZIP byte slice.
    fn zip_entry_names(bytes: &[u8]) -> Vec<String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect()
    }
}
