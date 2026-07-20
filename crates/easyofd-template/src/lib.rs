//! # easyofd-template
//!
//! OFD template engine — replaces `{placeholder}` patterns in OFD XML content.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::collections::HashMap;
//! use easyofd::EasyOfd;
//!
//! let mut data = HashMap::new();
//! data.insert("name".to_string(), "Alice".to_string());
//! data.insert("amount".to_string(), "$1,234.00".to_string());
//!
//! EasyOfd::fill_template("template.ofd", &data)?
//!     .save("output.ofd")?;
//! ```

use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

use easyofd_core::OfdResult;
use zip::write::FileOptions;
use zip::ZipWriter;

/// An OFD template filler.
///
/// Opens a template OFD file, replaces `{key}` placeholders in XML content,
/// and writes the result to a new OFD file.
pub struct OfdTemplateFiller {
    output: Vec<u8>,
}

impl OfdTemplateFiller {
    /// Fill a template OFD with placeholder values.
    ///
    /// Supports `{key}` style placeholders in all XML files within the OFD ZIP.
    ///
    /// # Errors
    ///
    /// Returns an error if the template file cannot be read or is not a valid ZIP.
    pub fn fill(
        template_path: impl AsRef<std::path::Path>,
        data: &HashMap<String, String>,
    ) -> OfdResult<Self> {
        let template_bytes = std::fs::read(template_path).map_err(easyofd_core::OfdError::Io)?;
        Self::fill_bytes(&template_bytes, data)
    }

    /// Fill a template OFD from in-memory bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is not a valid ZIP.
    pub fn fill_bytes(
        template_bytes: &[u8],
        data: &HashMap<String, String>,
    ) -> OfdResult<Self> {
        let cursor = Cursor::new(template_bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;

        let out_buf = Vec::new();
        let out_cursor = Cursor::new(out_buf);
        let mut zip = ZipWriter::new(out_cursor);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            let name = entry.name().to_string();
            let mut content = Vec::new();
            entry.read_to_end(&mut content).map_err(easyofd_core::OfdError::Io)?;

            // Replace placeholders in XML files
            let is_xml = name.ends_with(".xml") || name.ends_with(".XML");
            if is_xml {
                let text = String::from_utf8_lossy(&content).to_string();
                let mut replaced = text;
                for (key, value) in data {
                    let placeholder = format!("{{{key}}}");
                    replaced = replaced.replace(&placeholder, value);
                }
                zip.start_file(name, options)
                    .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
                zip.write_all(replaced.as_bytes())
                    .map_err(easyofd_core::OfdError::Io)?;
            } else {
                // Binary files (images, etc.) — copy as-is
                zip.start_file(name, options)
                    .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
                zip.write_all(&content).map_err(easyofd_core::OfdError::Io)?;
            }
        }

        let cursor = zip
            .finish()
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        let output = cursor.into_inner();

        Ok(Self { output })
    }

    /// Return the filled OFD as bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.output
    }

    /// Save the filled OFD to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if file I/O fails.
    pub fn save(self, path: impl AsRef<std::path::Path>) -> OfdResult<()> {
        std::fs::write(path, self.output).map_err(easyofd_core::OfdError::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};
    use easyofd_writer::OfdWriter;

    fn make_template(placeholders: &[&str]) -> Vec<u8> {
        let mut page = OfdPage::new(210.0, 297.0);
        for p in placeholders {
            page.add_text(TextObject::new(20.0, 30.0, format!("{{{p}}}")));
        }
        let mut writer = OfdWriter::new();
        writer.add_page(page);
        writer.build().unwrap()
    }

    #[test]
    fn test_fill_single_placeholder() {
        let template = make_template(&["name"]);
        let mut data = HashMap::new();
        data.insert("name".into(), "Alice".into());

        let filler = OfdTemplateFiller::fill_bytes(&template, &data).unwrap();
        let output = filler.into_bytes();

        // Verify the output is a valid ZIP
        let cursor = Cursor::new(&output);
        let archive = zip::ZipArchive::new(cursor).unwrap();
        assert!(archive.len() > 0);

        // Read the page XML and check replacement
        let reader = easyofd_reader::OfdReader::from_bytes(&output).unwrap();
        let text = reader.extract_all_text();
        assert!(text.contains("Alice"));
        assert!(!text.contains("{name}"));
    }

    #[test]
    fn test_fill_multiple_placeholders() {
        let template = make_template(&["title", "amount", "date"]);
        let mut data = HashMap::new();
        data.insert("title".into(), "Invoice #001".into());
        data.insert("amount".into(), "$1,234.00".into());
        data.insert("date".into(), "2026-01-15".into());

        let filler = OfdTemplateFiller::fill_bytes(&template, &data).unwrap();
        let output = filler.into_bytes();

        let reader = easyofd_reader::OfdReader::from_bytes(&output).unwrap();
        let text = reader.extract_all_text();
        assert!(text.contains("Invoice #001"));
        assert!(text.contains("$1,234.00"));
        assert!(text.contains("2026-01-15"));
        assert!(!text.contains("{title}"));
    }

    #[test]
    fn test_fill_template_file() {
        let template = make_template(&["greeting"]);
        let dir = std::env::temp_dir().join("easyofd_template");
        std::fs::create_dir_all(&dir).unwrap();
        let tpl_path = dir.join("template.ofd");
        std::fs::write(&tpl_path, &template).unwrap();

        let mut data = HashMap::new();
        data.insert("greeting".into(), "Hello World".into());

        let filler = OfdTemplateFiller::fill(&tpl_path, &data).unwrap();
        let output_path = dir.join("filled.ofd");
        filler.save(&output_path).unwrap();

        let reader = easyofd_reader::OfdReader::open(&output_path).unwrap();
        assert!(reader.extract_all_text().contains("Hello World"));

        let _ = std::fs::remove_file(&tpl_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_fill_missing_key_preserves_placeholder() {
        // Keys not in the data map should remain as-is
        let template = make_template(&["present", "missing"]);
        let mut data = HashMap::new();
        data.insert("present".into(), "value".into());
        // "missing" is not in the map

        let filler = OfdTemplateFiller::fill_bytes(&template, &data).unwrap();
        let output = filler.into_bytes();

        let reader = easyofd_reader::OfdReader::from_bytes(&output).unwrap();
        let text = reader.extract_all_text();
        assert!(text.contains("value"));
        assert!(text.contains("{missing}")); // preserved
    }
}
