//! # `EasyOFD`
//!
//! A Rust library for easy OFD (Open Fixed-layout Document) operations,
//! inspired by [EasyExcel](https://github.com/alibaba/easyexcel).
//!
//! ## Quick Start
//!
//! ### One-liner write with derive macro
//!
//! ```rust,ignore
//! use easyofd::{EasyOfd, OfdModel};
//!
//! #[derive(OfdModel)]
//! #[ofd(page_width = 210.0, page_height = 297.0)]
//! struct Invoice {
//!     #[ofd(x = 20.0, y = 30.0, size = 18.0, bold)]
//!     title: String,
//!     #[ofd(x = 20.0, y = 50.0)]
//!     amount: String,
//! }
//!
//! let data = vec![
//!     Invoice { title: "Invoice #001".into(), amount: "$100.00".into() },
//!     Invoice { title: "Invoice #002".into(), amount: "$200.00".into() },
//! ];
//!
//! EasyOfd::write::<Invoice>("output.ofd").do_write(&data)?;
//! ```
//!
//! ### Manual page construction
//!
//! ```rust,ignore
//! use easyofd::{EasyOfd, TextObject, OfdPage};
//!
//! let mut page = OfdPage::new(210.0, 297.0);
//! page.add_text(TextObject::new(20.0, 30.0, "Hello OFD!"));
//!
//! EasyOfd::write_pages("output.ofd")
//!     .metadata_title("My Document")
//!     .do_write(vec![page])?;
//! ```

// Re-export core types for convenience.
pub use easyofd_core::{
    ContentObject, ImageFormat, ImageObject, OfdError, OfdField, OfdFieldKind, OfdMetadata,
    OfdModel, OfdPage, OfdResult, PathObject, TextObject, page_size,
};

// Re-export derive macro.
pub use easyofd_derive::OfdModel;

// Re-export writer internals for advanced usage.
pub use easyofd_writer::{OfdWriter, WriteOptions, EmbeddedFont, FontFormat};

// Re-export reader for advanced usage.
pub use easyofd_reader::OfdReader;

/// Re-export template for advanced usage.
pub use easyofd_template::OfdTemplateFiller;

// Re-export signature types for advanced usage.
pub use easyofd_signature::{ElectronicSeal, OfdSignatureBuilder, SignatureAlgorithm, SignedOfd};

// Re-export convert functions for advanced usage.
pub use easyofd_convert::{pdf_to_ofd, ofd_to_pdf, convert_image, ConvertOptions, ImageConvertFormat};

// ─── EasyOfd Static Factory ──────────────────────────────────────────────────

/// The main entry point for easyofd operations.
///
/// Mirrors the `EasyExcel` / `EasyExcelFactory` pattern from the Java library.
/// All methods are static and return builders.
pub struct EasyOfd;

impl EasyOfd {
    /// Start a typed write operation using an `OfdModel` type.
    ///
    /// Returns an [`OfdWriterBuilder`] for fluent configuration.
    pub fn write<T: OfdModel>(path: impl Into<String>) -> OfdWriterBuilder<T> {
        OfdWriterBuilder {
            path: path.into(),
            _phantom: std::marker::PhantomData,
            metadata: OfdMetadata::default(),
        }
    }

    /// Start a page-based write operation (no model type required).
    ///
    /// Returns a [`PageWriterBuilder`] for fluent configuration.
    pub fn write_pages(path: impl Into<String>) -> PageWriterBuilder {
        PageWriterBuilder {
            path: path.into(),
            metadata: OfdMetadata::default(),
        }
    }

    /// Write pages directly to a file in one call.
    ///
    /// # Errors
    ///
    /// Returns an error if ZIP creation or file I/O fails.
    pub fn write_pages_to(
        path: impl AsRef<std::path::Path>,
        pages: Vec<OfdPage>,
    ) -> OfdResult<()> {
        let mut writer = OfdWriter::new();
        writer.add_pages(pages);
        writer.build_to_file(path)
    }

    /// Write pages directly to bytes in one call.
    ///
    /// # Errors
    ///
    /// Returns an error if ZIP creation fails.
    pub fn write_pages_to_bytes(pages: Vec<OfdPage>) -> OfdResult<Vec<u8>> {
        let mut writer = OfdWriter::new();
        writer.add_pages(pages);
        writer.build()
    }

    /// Open and parse an OFD file for reading.
    ///
    /// Returns an [`OfdReader`] that provides access to pages and text content.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or is not a valid OFD document.
    pub fn read(path: impl AsRef<std::path::Path>) -> OfdResult<OfdReader> {
        OfdReader::open(path)
    }

    /// Parse an OFD file from in-memory bytes.
    ///
    /// Returns an [`OfdReader`] for extracting content.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is not a valid OFD document.
    pub fn read_from_bytes(data: &[u8]) -> OfdResult<OfdReader> {
        OfdReader::from_bytes(data)
    }

    /// Fill an OFD template with placeholder values.
    ///
    /// Replaces `{key}` patterns in XML content with values from the data map.
    ///
    /// # Errors
    ///
    /// Returns an error if the template file cannot be read.
    pub fn fill_template(
        template_path: impl AsRef<std::path::Path>,
        data: &std::collections::HashMap<String, String>,
    ) -> OfdResult<OfdTemplateFiller> {
        OfdTemplateFiller::fill(template_path, data)
    }
}

// ─── Typed Writer Builder ────────────────────────────────────────────────────

/// Builder for typed OFD write operations using `OfdModel`.
///
/// Created by [`EasyOfd::write::<T>(path)`](EasyOfd::write).
pub struct OfdWriterBuilder<T: OfdModel> {
    path: String,
    _phantom: std::marker::PhantomData<T>,
    metadata: OfdMetadata,
}

impl<T: OfdModel> OfdWriterBuilder<T> {
    /// Set the document title.
    #[must_use]
    pub fn metadata_title(mut self, title: impl Into<String>) -> Self {
        self.metadata.title = Some(title.into());
        self
    }

    /// Set the document author.
    #[must_use]
    pub fn metadata_author(mut self, author: impl Into<String>) -> Self {
        self.metadata.author = Some(author.into());
        self
    }

    /// Set the document creator.
    #[must_use]
    pub fn metadata_creator(mut self, creator: impl Into<String>) -> Self {
        self.metadata.creator = Some(creator.into());
        self
    }

    /// Execute the write operation.
    ///
    /// Each item in `data` becomes one page in the OFD document.
    ///
    /// # Errors
    ///
    /// Returns an error if model conversion, ZIP creation, or file I/O fails.
    pub fn do_write(&self, data: &[T]) -> OfdResult<()> {
        let pages = T::to_pages(data)?;
        let options = WriteOptions {
            metadata: self.metadata.clone(),
        };
        let mut writer = OfdWriter::with_options(options);
        writer.add_pages(pages);
        writer.build_to_file(&self.path)
    }

    /// Execute the write operation and return the OFD bytes (no file I/O).
    ///
    /// # Errors
    ///
    /// Returns an error if model conversion or ZIP creation fails.
    pub fn do_write_to_bytes(&self, data: &[T]) -> OfdResult<Vec<u8>> {
        let pages = T::to_pages(data)?;
        let options = WriteOptions {
            metadata: self.metadata.clone(),
        };
        let mut writer = OfdWriter::with_options(options);
        writer.add_pages(pages);
        writer.build()
    }
}

// ─── Page Writer Builder ─────────────────────────────────────────────────────

/// Builder for page-based OFD write operations (no model type).
///
/// Created by [`EasyOfd::write_pages(path)`](EasyOfd::write_pages).
pub struct PageWriterBuilder {
    path: String,
    metadata: OfdMetadata,
}

impl PageWriterBuilder {
    /// Set the document title.
    #[must_use]
    pub fn metadata_title(mut self, title: impl Into<String>) -> Self {
        self.metadata.title = Some(title.into());
        self
    }

    /// Set the document author.
    #[must_use]
    pub fn metadata_author(mut self, author: impl Into<String>) -> Self {
        self.metadata.author = Some(author.into());
        self
    }

    /// Set the document creator.
    #[must_use]
    pub fn metadata_creator(mut self, creator: impl Into<String>) -> Self {
        self.metadata.creator = Some(creator.into());
        self
    }

    /// Execute the write operation.
    ///
    /// # Errors
    ///
    /// Returns an error if ZIP creation or file I/O fails.
    pub fn do_write(&self, pages: Vec<OfdPage>) -> OfdResult<()> {
        let options = WriteOptions {
            metadata: self.metadata.clone(),
        };
        let mut writer = OfdWriter::with_options(options);
        writer.add_pages(pages);
        writer.build_to_file(&self.path)
    }

    /// Execute the write operation and return the OFD bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if ZIP creation fails.
    pub fn do_write_to_bytes(&self, pages: Vec<OfdPage>) -> OfdResult<Vec<u8>> {
        let options = WriteOptions {
            metadata: self.metadata.clone(),
        };
        let mut writer = OfdWriter::with_options(options);
        writer.add_pages(pages);
        writer.build()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── EasyOfd static methods ────────────────────────────────────────────────

    #[test]
    fn test_write_pages_builder() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Hello from EasyOFD!"));

        let bytes = EasyOfd::write_pages("test.ofd")
            .metadata_title("Test Document")
            .do_write_to_bytes(vec![page])
            .unwrap();

        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_write_pages_to_bytes_static() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Direct bytes"));

        let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_write_pages_to_file() {
        let dir = std::env::temp_dir().join("easyofd_unit_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("unit_test.ofd");

        let page = OfdPage::new(210.0, 297.0);
        EasyOfd::write_pages_to(&path, vec![page]).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_with_images() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Invoice"));
        page.add_image(ImageObject::jpeg(
            150.0, 30.0, 30.0, 30.0, vec![0xFF, 0xD8, 0xFF, 0xE0],
        ));

        let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.jpeg".to_string()));
    }

    #[test]
    fn test_with_paths() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "With lines"));
        page.add_path(PathObject::hline(20.0, 40.0, 190.0));
        page.add_path(PathObject::rect(20.0, 50.0, 170.0, 100.0));

        let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
        assert!(!bytes.is_empty());
    }

    // ── PageWriterBuilder all methods ─────────────────────────────────────────

    #[test]
    fn test_page_writer_builder_all_metadata() {
        let page = OfdPage::new(210.0, 297.0);
        let bytes = EasyOfd::write_pages("x.ofd")
            .metadata_title("T")
            .metadata_author("A")
            .metadata_creator("C")
            .do_write_to_bytes(vec![page])
            .unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_page_writer_builder_do_write() {
        let dir = std::env::temp_dir().join("easyofd_unit_test2");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("page_builder.ofd");

        let page = OfdPage::new(210.0, 297.0);
        EasyOfd::write_pages(path.to_string_lossy().into_owned())
            .do_write(vec![page])
            .unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
        let _ = std::fs::remove_file(&path);
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn test_empty_pages() {
        let bytes = EasyOfd::write_pages_to_bytes(vec![]).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_empty_content_page() {
        let page = OfdPage::new(210.0, 297.0);
        let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_special_xml_chars() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "<>&\"'"));
        let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
        assert!(!bytes.is_empty());
    }

    // ── Re-export verification ────────────────────────────────────────────────

    #[test]
    fn test_re_exports() {
        // Verify all re-exported types are accessible
        let _: OfdPage = OfdPage::new(1.0, 1.0);
        let _: TextObject = TextObject::new(0.0, 0.0, "x");
        let _: ImageObject = ImageObject::jpeg(0.0, 0.0, 1.0, 1.0, vec![0]);
        let _: PathObject = PathObject::new(0.0, 0.0, "M0 0");
        let _: ImageFormat = ImageFormat::Jpeg;
        let _: OfdFieldKind = OfdFieldKind::Text;
        let _: OfdMetadata = OfdMetadata::default();
        let _: WriteOptions = WriteOptions::default();
    }

    fn zip_entry_names(bytes: &[u8]) -> Vec<String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect()
    }
}
