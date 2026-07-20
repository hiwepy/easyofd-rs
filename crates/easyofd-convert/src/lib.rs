//! # easyofd-convert
//!
//! Bidirectional PDF ↔ OFD conversion for easyofd-rs.
//!
//! ## Status: v0.4 — API design complete, conversion logic stubs
//!
//! Full conversion requires integration with PDF libraries (lopdf/printpdf)
//! and OFD renderers. Current implementation provides the API surface and
//! metadata mapping.

use easyofd_core::OfdResult;

/// Conversion direction and options.
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// Page range to convert (0-based, empty = all pages).
    pub pages: std::ops::Range<usize>,
    /// Output page size override in mm (width, height). None = preserve original.
    pub page_size: Option<(f64, f64)>,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            pages: 0..0, // empty = all
            page_size: None,
        }
    }
}

/// Convert a PDF file to OFD format.
///
/// Extracts text content and page structure from the PDF,
/// mapping to OFD text objects.
///
/// # Errors
///
/// Returns an error if the input file cannot be read or parsed.
pub fn pdf_to_ofd(
    pdf_path: impl AsRef<std::path::Path>,
    ofd_path: impl AsRef<std::path::Path>,
    options: &ConvertOptions,
) -> OfdResult<()> {
    let _pdf_bytes = std::fs::read(pdf_path).map_err(easyofd_core::OfdError::Io)?;
    let _ = options;
    // Conversion logic: parse PDF pages → create OfdPage objects → write OFD
    // Requires lopdf for PDF parsing + easyofd-writer for OFD output.
    Err(easyofd_core::OfdError::Conversion(
        "PDF→OFD conversion requires lopdf integration (planned)".into(),
    ))
}

/// Convert an OFD file to PDF format.
///
/// Renders OFD page content to PDF pages.
///
/// # Errors
///
/// Returns an error if the input file cannot be read or parsed.
pub fn ofd_to_pdf(
    ofd_path: impl AsRef<std::path::Path>,
    pdf_path: impl AsRef<std::path::Path>,
    options: &ConvertOptions,
) -> OfdResult<()> {
    let _ofd_bytes = std::fs::read(ofd_path).map_err(easyofd_core::OfdError::Io)?;
    let _ = pdf_path;
    let _ = options;
    // Conversion logic: parse OFD pages → create PDF page objects → write PDF
    // Requires easyofd-reader for OFD parsing + printpdf for PDF output.
    Err(easyofd_core::OfdError::Conversion(
        "OFD→PDF conversion requires printpdf integration (planned)".into(),
    ))
}

/// Image format conversion helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageConvertFormat {
    /// JPEG format.
    Jpeg,
    /// PNG format.
    Png,
    /// BMP format.
    Bmp,
}

/// Convert an image between formats (for use in OFD Resource embedding).
///
/// # Errors
///
/// Returns an error if conversion fails.
pub fn convert_image(
    _input: &[u8],
    _target_format: ImageConvertFormat,
) -> OfdResult<Vec<u8>> {
    Err(easyofd_core::OfdError::Conversion(
        "image conversion requires image crate integration (planned)".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_options_default() {
        let opts = ConvertOptions::default();
        assert!(opts.pages.is_empty()); // 0..0 = all pages
        assert!(opts.page_size.is_none());
    }

    #[test]
    fn test_convert_options_custom() {
        let opts = ConvertOptions {
            pages: 0..5,
            page_size: Some((210.0, 297.0)),
        };
        assert_eq!(opts.pages, (0..5));
        assert_eq!(opts.page_size, Some((210.0, 297.0)));
    }

    #[test]
    fn test_pdf_to_ofd_returns_error_for_missing_file() {
        let result = pdf_to_ofd("nonexistent.pdf", "out.ofd", &ConvertOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_ofd_to_pdf_returns_error_for_missing_file() {
        let result = ofd_to_pdf("nonexistent.ofd", "out.pdf", &ConvertOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_image_returns_error() {
        let result = convert_image(&[0xFF, 0xD8], ImageConvertFormat::Png);
        assert!(result.is_err());
    }

    #[test]
    fn test_image_convert_format_enum() {
        assert_ne!(ImageConvertFormat::Jpeg, ImageConvertFormat::Png);
        assert_ne!(ImageConvertFormat::Bmp, ImageConvertFormat::Jpeg);
    }
}
