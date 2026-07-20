//! # easyofd-reader
//!
//! OFD file reader that parses GB/T 33190-2016 compliant ZIP archives.
//!
//! ## Architecture
//!
//! ```text
//! input.ofd (ZIP)
//! ├── OFD.xml                    → find DocRoot
//! └── Doc_0/
//!     ├── Document.xml           → read page list
//!     └── Pages/
//!         ├── Page_0.xml         → parse content
//!         └── Page_N.xml
//! ```

use std::io::{BufReader, Cursor, Read};

use easyofd_core::{
    ContentObject, ImageFormat, ImageObject, OfdError, OfdPage, OfdResult, TextObject,
};
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;

/// An OFD document reader.
pub struct OfdReader {
    pages: Vec<OfdPage>,
}

impl OfdReader {
    /// Open and parse an OFD file from a path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or contains invalid OFD data.
    pub fn open(path: impl AsRef<std::path::Path>) -> OfdResult<Self> {
        let data = std::fs::read(path).map_err(OfdError::Io)?;
        Self::from_bytes(&data)
    }

    /// Parse an OFD file from in-memory bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is invalid.
    pub fn from_bytes(data: &[u8]) -> OfdResult<Self> {
        let cursor = Cursor::new(data);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| OfdError::Zip(e.to_string()))?;

        let doc_root = parse_ofd_entry(&mut archive)?;
        let page_refs = parse_document_entry(&mut archive, &doc_root)?;

        let mut pages = Vec::with_capacity(page_refs.len());
        for page_loc in &page_refs {
            let page_path = format!("{doc_root}/{page_loc}");
            let page = parse_page_entry(&mut archive, &page_path)?;
            pages.push(page);
        }

        Ok(Self { pages })
    }

    /// Number of pages in the document.
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// All parsed pages.
    #[must_use]
    pub fn pages(&self) -> &[OfdPage] {
        &self.pages
    }

    /// Extract text from all pages, one `String` per page.
    #[must_use]
    pub fn extract_text(&self) -> Vec<String> {
        self.pages.iter().map(page_text).collect()
    }

    /// Extract all text joined into a single string with page separators.
    #[must_use]
    pub fn extract_all_text(&self) -> String {
        self.extract_text().join("\n---\n")
    }
}

/// Join all text objects on a page into one string.
fn page_text(page: &OfdPage) -> String {
    page.content
        .iter()
        .filter_map(|obj| {
            if let ContentObject::Text(t) = obj {
                Some(t.text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── XML Parsing ─────────────────────────────────────────────────────────────

/// Parse OFD.xml → return the DocRoot directory (e.g. "Doc_0").
fn parse_ofd_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> OfdResult<String> {
    let xml = read_zip_entry(archive, "OFD.xml")?;
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut doc_root = String::new();
    let mut in_target = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"ofd:DocRoot" => {
                in_target = true;
            }
            Ok(Event::Text(ref e)) if in_target => {
                doc_root = e.unescape().unwrap_or_default().to_string();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"ofd:DocRoot" => {
                in_target = false;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OfdError::Xml(format!("OFD.xml: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    if doc_root.is_empty() {
        return Err(OfdError::InvalidDocument("missing DocRoot".into()));
    }

    // Strip "/Document.xml" suffix to get the doc directory
    Ok(doc_root
        .strip_suffix("/Document.xml")
        .unwrap_or(&doc_root)
        .to_string())
}

/// Parse Document.xml → return list of page BaseLoc paths (e.g. "Pages/Page_0.xml").
fn parse_document_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    doc_dir: &str,
) -> OfdResult<Vec<String>> {
    let path = format!("{doc_dir}/Document.xml");
    let xml = read_zip_entry(archive, &path)?;
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut pages = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e))
                if e.name().as_ref() == b"ofd:Page" =>
            {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"BaseLoc" {
                        let val = attr.unescape_value().unwrap_or_default();
                        pages.push(val.to_string());
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OfdError::Xml(format!("Document.xml: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(pages)
}

/// Parse Page_N.xml → return `OfdPage` with dimensions and content objects.
fn parse_page_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    page_path: &str,
) -> OfdResult<OfdPage> {
    let xml = read_zip_entry(archive, page_path)?;
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.trim_text(true);
    let mut buf = Vec::new();

    let mut width = 210.0_f64;
    let mut height = 297.0_f64;
    let mut content = Vec::new();

    let mut current_text: Option<TextObjectBuilder> = None;
    let mut in_text_code = false;
    let mut in_physical_box = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                match e.name().as_ref() {
                    b"ofd:PhysicalBox" => in_physical_box = true,
                    b"ofd:TextObject" => current_text = Some(parse_text_object_attrs(e)?),
                    b"ofd:TextCode" => in_text_code = true,
                    b"ofd:ImageObject" => {
                        let img = parse_image_object_attrs(e)?;
                        content.push(ContentObject::Image(ImageObject::new(
                            img.x, img.y, img.width, img.height,
                            Vec::new(),
                            img.format,
                        )));
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_physical_box {
                    let parts: Vec<f64> = text
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if parts.len() >= 4 {
                        width = parts[2];
                        height = parts[3];
                    }
                }
                if in_text_code {
                    if let Some(ref mut t) = current_text {
                        t.text = text;
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"ofd:PhysicalBox" => in_physical_box = false,
                    b"ofd:TextObject" => {
                        if let Some(t) = current_text.take() {
                            let mut obj = TextObject::new(t.x, t.y, t.text);
                            if let Some(f) = t.font {
                                obj = obj.font(f);
                            }
                            if let Some(s) = t.size {
                                obj = obj.size(s);
                            }
                            content.push(ContentObject::Text(obj));
                        }
                    }
                    b"ofd:TextCode" => in_text_code = false,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OfdError::Xml(format!("{page_path}: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    Ok(OfdPage { width, height, content })
}

// ─── Attribute Parsing Helpers ───────────────────────────────────────────────

struct TextObjectBuilder {
    x: f64,
    y: f64,
    text: String,
    font: Option<String>,
    size: Option<f64>,
}

fn parse_text_object_attrs(e: &quick_xml::events::BytesStart) -> OfdResult<TextObjectBuilder> {
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    let mut font = None;
    let mut size = None;

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"Boundary" => {
                let parts: Vec<f64> = attr
                    .unescape_value()
                    .unwrap_or_default()
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() >= 2 {
                    x = parts[0];
                    y = parts[1];
                }
            }
            b"Font" => font = Some(attr.unescape_value().unwrap_or_default().to_string()),
            b"Size" => size = attr.unescape_value().unwrap_or_default().parse().ok(),
            _ => {}
        }
    }

    Ok(TextObjectBuilder {
        x,
        y,
        text: String::new(),
        font,
        size,
    })
}

struct ImageObjectBuilder {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    format: ImageFormat,
}

fn parse_image_object_attrs(e: &quick_xml::events::BytesStart) -> OfdResult<ImageObjectBuilder> {
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    let mut w = 0.0_f64;
    let mut h = 0.0_f64;

    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"Boundary" {
            let parts: Vec<f64> = attr
                .unescape_value()
                .unwrap_or_default()
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if parts.len() >= 4 {
                x = parts[0];
                y = parts[1];
                w = parts[2];
                h = parts[3];
            }
        }
    }

    Ok(ImageObjectBuilder {
        x,
        y,
        width: w,
        height: h,
        format: ImageFormat::Jpeg,
    })
}

// ─── ZIP Helper ──────────────────────────────────────────────────────────────

fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> OfdResult<Vec<u8>> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| OfdError::Zip(format!("{name}: {e}")))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(OfdError::Io)?;
    Ok(buf)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::OfdPage;
    use easyofd_writer::OfdWriter;

    fn roundtrip(pages: Vec<OfdPage>) -> Vec<u8> {
        let mut writer = OfdWriter::new();
        for page in pages {
            writer.add_page(page);
        }
        writer.build().unwrap()
    }

    #[test]
    fn test_empty_document() {
        let bytes = OfdWriter::new().build().unwrap();
        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 0);
    }

    #[test]
    fn test_single_text_page() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Hello OFD Reader!"));
        let bytes = roundtrip(vec![page]);

        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 1);
        assert_eq!(reader.pages()[0].content.len(), 1);

        let text = reader.extract_text();
        assert_eq!(text.len(), 1);
        assert!(text[0].contains("Hello OFD Reader!"));
    }

    #[test]
    fn test_multiple_pages() {
        let mut pages = Vec::new();
        for i in 1..=3 {
            let mut page = OfdPage::new(210.0, 297.0);
            page.add_text(TextObject::new(10.0, 20.0, format!("Page {i} text")));
            pages.push(page);
        }
        let bytes = roundtrip(pages);

        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 3);
        let text = reader.extract_text();
        assert_eq!(text.len(), 3);
        assert!(text[0].contains("Page 1"));
        assert!(text[2].contains("Page 3"));
    }

    #[test]
    fn test_extract_all_text() {
        let mut p1 = OfdPage::new(210.0, 297.0);
        p1.add_text(TextObject::new(10.0, 20.0, "First"));
        let mut p2 = OfdPage::new(210.0, 297.0);
        p2.add_text(TextObject::new(10.0, 20.0, "Second"));
        let bytes = roundtrip(vec![p1, p2]);

        let reader = OfdReader::from_bytes(&bytes).unwrap();
        let all = reader.extract_all_text();
        assert!(all.contains("First"));
        assert!(all.contains("Second"));
        assert!(all.contains("---"));
    }

    #[test]
    fn test_text_and_image() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Invoice"));
        page.add_image(ImageObject::jpeg(150.0, 30.0, 30.0, 30.0, vec![0xFF, 0xD8]));
        let bytes = roundtrip(vec![page]);

        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.pages()[0].content.len(), 2);
    }

    #[test]
    fn test_from_file() {
        let dir = std::env::temp_dir().join("easyofd_reader");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ofd");

        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "File test"));
        let mut w = OfdWriter::new();
        w.add_page(page);
        w.build_to_file(&path).unwrap();

        let reader = OfdReader::open(&path).unwrap();
        assert_eq!(reader.page_count(), 1);
        assert!(reader.extract_all_text().contains("File test"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_invalid_data() {
        assert!(OfdReader::from_bytes(b"not a zip file").is_err());
    }

    #[test]
    fn test_styled_text() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(
            TextObject::new(10.0, 20.0, "Styled")
                .font("SimHei")
                .size(18.0)
                .bold(),
        );
        let bytes = roundtrip(vec![page]);
        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 1);
        assert!(reader.extract_all_text().contains("Styled"));
    }
}
