//! The `OfdModel` trait — the core abstraction for OFD document mapping.

use crate::error::OfdResult;
use crate::model::OfdPage;

/// Metadata about a column/field in an OFD model, used for declarative mapping.
#[derive(Debug, Clone)]
pub struct OfdField {
    /// Field name (Rust field name).
    pub name: &'static str,
    /// Display position (x, y) in mm.
    pub position: (f64, f64),
    /// Font family.
    pub font: &'static str,
    /// Font size in pt.
    pub size: f64,
    /// Font weight (400 = normal, 700 = bold).
    pub weight: u32,
    /// Whether italic.
    pub italic: bool,
    /// Text color as RGB hex.
    pub color: u32,
    /// Field kind for rendering.
    pub kind: OfdFieldKind,
}

/// The kind of an OFD field — determines how it is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfdFieldKind {
    /// Render as text.
    Text,
    /// Render as an image (field value must be `Vec<u8>`).
    Image,
}

/// The `OfdModel` trait defines the contract for Rust types that can be
/// mapped to OFD page content.
///
/// This is the OFD equivalent of `ExcelRow` in easyexcel-rs.
/// Derive it with `#[derive(OfdModel)]` for automatic implementation.
pub trait OfdModel: Sized {
    /// Returns the field schema for this model.
    fn schema() -> &'static [OfdField];

    /// Returns the page size (width, height) in mm.
    fn page_size() -> (f64, f64) {
        (210.0, 297.0) // A4 default
    }

    /// Convert this model instance into an OFD page.
    ///
    /// # Errors
    ///
    /// Returns an error if a field cannot be converted to a page content object.
    fn to_page(&self) -> OfdResult<OfdPage>;

    /// Convert a slice of model instances into a vec of OFD pages.
    ///
    /// # Errors
    ///
    /// Returns an error if any item fails conversion via [`to_page`](OfdModel::to_page).
    fn to_pages(items: &[Self]) -> OfdResult<Vec<OfdPage>> {
        items.iter().map(Self::to_page).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::OfdError;
    use crate::model::{ContentObject, TextObject};

    /// A minimal test model implementing `OfdModel` manually.
    struct TestModel {
        text: String,
    }

    impl OfdModel for TestModel {
        fn schema() -> &'static [OfdField] {
            &[OfdField {
                name: "text",
                position: (10.0, 20.0),
                font: "SimSun",
                size: 12.0,
                weight: 400,
                italic: false,
                color: 0,
                kind: OfdFieldKind::Text,
            }]
        }

        fn to_page(&self) -> OfdResult<OfdPage> {
            let mut page = OfdPage::new(210.0, 297.0);
            page.add_text(TextObject::new(10.0, 20.0, &self.text));
            Ok(page)
        }
    }

    /// A model whose `to_page` always fails.
    struct FailingModel;

    impl OfdModel for FailingModel {
        fn schema() -> &'static [OfdField] {
            &[]
        }
        fn to_page(&self) -> OfdResult<OfdPage> {
            Err(OfdError::Model("deliberate failure".into()))
        }
    }

    #[test]
    fn test_ofd_field_clone_debug() {
        let f = OfdField {
            name: "x",
            position: (1.0, 2.0),
            font: "f",
            size: 10.0,
            weight: 400,
            italic: false,
            color: 0,
            kind: OfdFieldKind::Text,
        };
        let f2 = f.clone();
        assert_eq!(f2.name, "x");
        assert!(format!("{f:?}").contains("OfdField"));
    }

    #[test]
    fn test_ofd_field_kind_clone_copy_debug() {
        let k = OfdFieldKind::Image;
        let k2 = k;
        assert_eq!(k2, OfdFieldKind::Image);
        assert_ne!(OfdFieldKind::Text, OfdFieldKind::Image);
        assert!(format!("{k:?}").contains("Image"));
    }

    #[test]
    fn test_ofd_model_default_page_size() {
        // TestModel doesn't override page_size, so it gets the default A4.
        assert_eq!(TestModel::page_size(), (210.0, 297.0));
    }

    #[test]
    fn test_ofd_model_to_page() {
        let m = TestModel { text: "hello".into() };
        let page = m.to_page().unwrap();
        assert!((page.width - 210.0).abs() < f64::EPSILON);
        assert!((page.height - 297.0).abs() < f64::EPSILON);
        assert_eq!(page.content.len(), 1);
        assert!(matches!(&page.content[0], ContentObject::Text(t) if t.text == "hello"));
    }

    #[test]
    fn test_ofd_model_to_pages() {
        let items = vec![
            TestModel { text: "a".into() },
            TestModel { text: "b".into() },
        ];
        let pages = TestModel::to_pages(&items).unwrap();
        assert_eq!(pages.len(), 2);
    }

    #[test]
    fn test_ofd_model_to_pages_empty() {
        let items: Vec<TestModel> = vec![];
        let pages = TestModel::to_pages(&items).unwrap();
        assert!(pages.is_empty());
    }

    #[test]
    fn test_ofd_model_to_pages_error() {
        let items = vec![FailingModel];
        let result = FailingModel::to_pages(&items);
        assert!(result.is_err());
    }

    #[test]
    fn test_ofd_model_schema() {
        let schema = TestModel::schema();
        assert_eq!(schema.len(), 1);
        assert_eq!(schema[0].name, "text");
    }

    #[test]
    fn test_failing_model_schema() {
        let schema = FailingModel::schema();
        assert!(schema.is_empty());
    }
}
