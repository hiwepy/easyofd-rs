//! Error types for easyofd.

use thiserror::Error;

/// The result type alias for easyofd operations.
pub type OfdResult<T> = std::result::Result<T, OfdError>;

/// Errors that can occur during OFD operations.
#[derive(Debug, Error)]
pub enum OfdError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// XML serialization/deserialization error.
    #[error("XML error: {0}")]
    Xml(String),

    /// ZIP archive error.
    #[error("ZIP error: {0}")]
    Zip(String),

    /// Invalid OFD document structure.
    #[error("invalid OFD document: {0}")]
    InvalidDocument(String),

    /// Invalid page content.
    #[error("invalid page content: {0}")]
    InvalidPage(String),

    /// Conversion error.
    #[error("conversion error: {0}")]
    Conversion(String),

    /// Model error (from `OfdModel` trait).
    #[error("model error: {0}")]
    Model(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file gone");
        let err = OfdError::Io(io_err);
        assert!(format!("{err}").contains("I/O error"));
    }

    #[test]
    fn test_error_display_xml() {
        let err = OfdError::Xml("bad tag".into());
        assert_eq!(format!("{err}"), "XML error: bad tag");
    }

    #[test]
    fn test_error_display_zip() {
        let err = OfdError::Zip("corrupt".into());
        assert_eq!(format!("{err}"), "ZIP error: corrupt");
    }

    #[test]
    fn test_error_display_invalid_document() {
        let err = OfdError::InvalidDocument("missing OFD.xml".into());
        assert_eq!(format!("{err}"), "invalid OFD document: missing OFD.xml");
    }

    #[test]
    fn test_error_display_invalid_page() {
        let err = OfdError::InvalidPage("no content".into());
        assert_eq!(format!("{err}"), "invalid page content: no content");
    }

    #[test]
    fn test_error_display_conversion() {
        let err = OfdError::Conversion("type mismatch".into());
        assert_eq!(format!("{err}"), "conversion error: type mismatch");
    }

    #[test]
    fn test_error_display_model() {
        let err = OfdError::Model("field missing".into());
        assert_eq!(format!("{err}"), "model error: field missing");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "oops");
        let err: OfdError = io_err.into();
        assert!(matches!(err, OfdError::Io(_)));
    }

    #[test]
    fn test_error_debug() {
        let err = OfdError::Xml("x".into());
        let dbg = format!("{err:?}");
        assert!(dbg.contains("Xml"));
    }

    #[test]
    fn test_result_alias() {
        let ok: OfdResult<i32> = Ok(42);
        let err: OfdResult<i32> = Err(OfdError::Model("fail".into()));
        assert!(ok.is_ok());
        assert!(matches!(ok, Ok(42)));
        assert!(err.is_err());
    }
}
