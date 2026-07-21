//! Core OFD data model types.
//!
//! These types map directly to the GB/T 33190-2016 XML elements.

use chrono::NaiveDateTime;

// ─── Document Metadata ───────────────────────────────────────────────────────

/// Metadata for an OFD document (OFD.xml level).
#[derive(Debug, Clone)]
pub struct OfdMetadata {
    /// Document version (default: "1.0").
    pub version: String,
    /// Document title.
    pub title: Option<String>,
    /// Document author.
    pub author: Option<String>,
    /// Creator application name.
    pub creator: Option<String>,
    /// Creation date.
    pub creation_date: Option<NaiveDateTime>,
}

impl Default for OfdMetadata {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            title: None,
            author: None,
            creator: None,
            creation_date: None,
        }
    }
}

// ─── Page Definition ─────────────────────────────────────────────────────────

/// A single page in an OFD document.
#[derive(Debug, Clone)]
pub struct OfdPage {
    /// Page width in mm.
    pub width: f64,
    /// Page height in mm.
    pub height: f64,
    /// Content blocks on this page.
    pub content: Vec<ContentObject>,
}

impl OfdPage {
    /// Create a new page with the given dimensions.
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            content: Vec::new(),
        }
    }

    /// Add a text object to this page.
    pub fn add_text(&mut self, text: TextObject) {
        self.content.push(ContentObject::Text(text));
    }

    /// Add an image object to this page.
    pub fn add_image(&mut self, image: ImageObject) {
        self.content.push(ContentObject::Image(image));
    }

    /// Add a path object to this page.
    pub fn add_path(&mut self, path: PathObject) {
        self.content.push(ContentObject::Path(path));
    }
}

// ─── Page Sizes ──────────────────────────────────────────────────────────────

/// Common page sizes in mm (width × height).
pub mod page_size {
    /// A4 portrait: 210 × 297 mm
    pub const A4: (f64, f64) = (210.0, 297.0);
    /// A4 landscape: 297 × 210 mm
    pub const A4_LANDSCAPE: (f64, f64) = (297.0, 210.0);
    /// A3 portrait: 297 × 420 mm
    pub const A3: (f64, f64) = (297.0, 420.0);
    /// Letter: 215.9 × 279.4 mm
    pub const LETTER: (f64, f64) = (215.9, 279.4);
}

// ─── Content Objects ─────────────────────────────────────────────────────────

/// A content object on an OFD page.
#[derive(Debug, Clone)]
pub enum ContentObject {
    /// A text block.
    Text(TextObject),
    /// An image.
    Image(ImageObject),
    /// A vector path.
    Path(PathObject),
}

/// A text object with position, font, and content.
#[derive(Debug, Clone)]
pub struct TextObject {
    /// X position in mm from left edge.
    pub x: f64,
    /// Y position in mm from top edge.
    pub y: f64,
    /// Font family name (e.g. "`SimSun`", "`SimHei`").
    pub font: String,
    /// Font size in pt.
    pub size: f64,
    /// Font weight: 400 = normal, 700 = bold.
    pub weight: u32,
    /// Whether the text is italic.
    pub italic: bool,
    /// Text color as RGB hex (e.g. 0x000000 for black).
    pub color: u32,
    /// The actual text content.
    pub text: String,
    /// Optional text width override in mm.
    /// If None, the writer will estimate based on character count.
    pub width: Option<f64>,
    /// Optional text height override in mm.
    /// If None, the writer will use font size.
    pub height: Option<f64>,
}

impl TextObject {
    /// Create a new text object with default styling.
    #[must_use]
    pub fn new(x: f64, y: f64, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            font: "SimSun".to_string(),
            size: 12.0,
            weight: 400,
            italic: false,
            color: 0x000_000,
            text: text.into(),
            width: None,
            height: None,
        }
    }

    /// Set the font family.
    #[must_use]
    pub fn font(mut self, font: impl Into<String>) -> Self {
        self.font = font.into();
        self
    }

    /// Set the font size in pt.
    #[must_use]
    pub fn size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    /// Set bold text.
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.weight = 700;
        self
    }

    /// Set italic text.
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Set text color as RGB hex.
    #[must_use]
    pub fn color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }
}

/// An image object with position and dimensions.
#[derive(Debug, Clone)]
pub struct ImageObject {
    /// X position in mm from left edge.
    pub x: f64,
    /// Y position in mm from top edge.
    pub y: f64,
    /// Width in mm.
    pub width: f64,
    /// Height in mm.
    pub height: f64,
    /// Image data (raw bytes).
    pub data: Vec<u8>,
    /// Image format.
    pub format: ImageFormat,
}

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG image.
    Jpeg,
    /// PNG image.
    Png,
    /// BMP image.
    Bmp,
    /// TIFF image.
    Tiff,
}

impl ImageObject {
    /// Create a new image object.
    #[must_use]
    pub fn new(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        data: Vec<u8>,
        format: ImageFormat,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            data,
            format,
        }
    }

    /// Create a JPEG image object.
    #[must_use]
    pub fn jpeg(x: f64, y: f64, width: f64, height: f64, data: Vec<u8>) -> Self {
        Self::new(x, y, width, height, data, ImageFormat::Jpeg)
    }

    /// Create a PNG image object.
    #[must_use]
    pub fn png(x: f64, y: f64, width: f64, height: f64, data: Vec<u8>) -> Self {
        Self::new(x, y, width, height, data, ImageFormat::Png)
    }

    /// Create an image object from a file path, auto-detecting the format.
    ///
    /// Reads the file and detects the format from:
    /// - File extension (.jpg/.jpeg → Jpeg, .png → Png, .bmp → Bmp, .tiff/.tif → Tiff)
    /// - Magic bytes (fallback if extension is ambiguous)
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the format is unsupported.
    pub fn from_file(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        path: impl AsRef<std::path::Path>,
    ) -> crate::OfdResult<Self> {
        let data = std::fs::read(path.as_ref()).map_err(crate::OfdError::Io)?;
        let format = detect_image_format(path.as_ref(), &data);
        Ok(Self::new(x, y, width, height, data, format))
    }
}

/// Detect image format from file extension and/or magic bytes.
fn detect_image_format(path: &std::path::Path, data: &[u8]) -> ImageFormat {
    // Check extension first
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => return ImageFormat::Jpeg,
            "png" => return ImageFormat::Png,
            "bmp" => return ImageFormat::Bmp,
            "tiff" | "tif" => return ImageFormat::Tiff,
            _ => {}
        }
    }
    // Fallback: magic bytes
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
        return ImageFormat::Jpeg;
    }
    if data.len() >= 4 && data[0] == 0x89 && data[1] == b'P' && data[2] == b'N' && data[3] == b'G' {
        return ImageFormat::Png;
    }
    if data.len() >= 2 && data[0] == b'B' && data[1] == b'M' {
        return ImageFormat::Bmp;
    }
    if data.len() >= 4
        && ((data[0] == b'I' && data[1] == b'I') || (data[0] == b'M' && data[1] == b'M'))
        && data[2] == 0x00 && data[3] == 0x2A
    {
        return ImageFormat::Tiff;
    }
    // Default to JPEG
    ImageFormat::Jpeg
}

/// A vector path object (lines, rectangles, curves).
#[derive(Debug, Clone)]
pub struct PathObject {
    /// X position in mm from left edge.
    pub x: f64,
    /// Y position in mm from top edge.
    pub y: f64,
    /// Stroke color as RGB hex.
    pub stroke_color: u32,
    /// Stroke width in mm.
    pub stroke_width: f64,
    /// Fill color as RGB hex (optional).
    pub fill_color: Option<u32>,
    /// SVG-style path data string.
    pub path_data: String,
}

impl PathObject {
    /// Create a new path object.
    #[must_use]
    pub fn new(x: f64, y: f64, path_data: impl Into<String>) -> Self {
        Self {
            x,
            y,
            stroke_color: 0x000_000,
            stroke_width: 0.35,
            fill_color: None,
            path_data: path_data.into(),
        }
    }

    /// Create a horizontal line.
    #[must_use]
    pub fn hline(x1: f64, y: f64, x2: f64) -> Self {
        Self::new(x1, y, format!("M{x1} {y}L{x2} {y}"))
    }

    /// Create a vertical line.
    #[must_use]
    pub fn vline(x: f64, y1: f64, y2: f64) -> Self {
        Self::new(x, y1, format!("M{x} {y1}L{x} {y2}"))
    }

    /// Create a rectangle outline.
    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub fn rect(x: f64, y: f64, w: f64, h: f64) -> Self {
        let d = format!("M{x} {y}L{} {y}L{} {}L{x} {}Z", x + w, x + w, y + h, y + h);
        Self::new(x, y, d)
    }

    /// Set stroke color.
    #[must_use]
    pub fn stroke_color(mut self, color: u32) -> Self {
        self.stroke_color = color;
        self
    }

    /// Set stroke width.
    #[must_use]
    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// Set fill color.
    #[must_use]
    pub fn fill_color(mut self, color: u32) -> Self {
        self.fill_color = Some(color);
        self
    }
}

// ─── Watermark / Annotation ──────────────────────────────────────────────────

/// A watermark or annotation that can be placed on OFD pages.
///
/// Inspired by hutool's `OfdWriter.add(int page, Annotation)`.
#[derive(Debug, Clone)]
pub struct Watermark {
    /// Watermark text (for text watermarks).
    pub text: Option<String>,
    /// Watermark image data (for image watermarks).
    pub image: Option<Vec<u8>>,
    /// Position (x, y) in mm.
    pub position: (f64, f64),
    /// Font size for text watermarks.
    pub font_size: f64,
    /// Font family for text watermarks.
    pub font: String,
    /// Text color as RGB hex.
    pub color: u32,
    /// Opacity 0.0 (transparent) – 1.0 (opaque). Default: 0.3.
    pub opacity: f64,
    /// Rotation angle in degrees. Default: 45.0 (diagonal).
    pub rotation: f64,
    /// Target page number (0-based). None = all pages.
    pub page: Option<usize>,
}

impl Default for Watermark {
    fn default() -> Self {
        Self {
            text: None,
            image: None,
            position: (0.0, 0.0),
            font_size: 48.0,
            font: "SimSun".to_string(),
            color: 0xCC_CC_CC,
            opacity: 0.3,
            rotation: 45.0,
            page: None,
        }
    }
}

impl Watermark {
    /// Create a text watermark.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }

    /// Create an image watermark.
    #[must_use]
    pub fn image(data: Vec<u8>) -> Self {
        Self {
            image: Some(data),
            ..Default::default()
        }
    }

    /// Set position.
    #[must_use]
    pub fn position(mut self, x: f64, y: f64) -> Self {
        self.position = (x, y);
        self
    }

    /// Set font size for text watermarks.
    #[must_use]
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// Set opacity.
    #[must_use]
    pub fn opacity(mut self, val: f64) -> Self {
        self.opacity = val;
        self
    }

    /// Set rotation in degrees.
    #[must_use]
    pub fn rotation(mut self, degrees: f64) -> Self {
        self.rotation = degrees;
        self
    }

    /// Apply to a specific page.
    #[must_use]
    pub fn on_page(mut self, page: usize) -> Self {
        self.page = Some(page);
        self
    }

    /// Apply to all pages.
    #[must_use]
    pub fn on_all_pages(mut self) -> Self {
        self.page = None;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── OfdMetadata ──────────────────────────────────────────────────────────

    #[test]
    fn test_ofd_metadata_default() {
        let meta = OfdMetadata::default();
        assert_eq!(meta.version, "1.0");
        assert!(meta.title.is_none());
        assert!(meta.author.is_none());
        assert!(meta.creator.is_none());
        assert!(meta.creation_date.is_none());
    }

    #[test]
    fn test_ofd_metadata_clone_debug() {
        let meta = OfdMetadata {
            title: Some("t".into()),
            ..Default::default()
        };
        let meta2 = meta.clone();
        assert_eq!(meta2.title.unwrap(), "t");
        let dbg = format!("{meta:?}");
        assert!(dbg.contains("OfdMetadata"));
    }

    // ─── OfdPage ──────────────────────────────────────────────────────────────

    #[test]
    fn test_ofd_page_new() {
        let page = OfdPage::new(210.0, 297.0);
        assert!((page.width - 210.0).abs() < f64::EPSILON);
        assert!((page.height - 297.0).abs() < f64::EPSILON);
        assert!(page.content.is_empty());
    }

    #[test]
    fn test_ofd_page_add_text() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "hello"));
        assert_eq!(page.content.len(), 1);
        assert!(matches!(&page.content[0], ContentObject::Text(_)));
    }

    #[test]
    fn test_ofd_page_add_image() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::jpeg(0.0, 0.0, 10.0, 10.0, vec![0xFF]));
        assert_eq!(page.content.len(), 1);
        assert!(matches!(&page.content[0], ContentObject::Image(_)));
    }

    #[test]
    fn test_ofd_page_add_path() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(PathObject::hline(0.0, 10.0, 100.0));
        assert_eq!(page.content.len(), 1);
        assert!(matches!(&page.content[0], ContentObject::Path(_)));
    }

    #[test]
    fn test_ofd_page_mixed_content() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "t"));
        page.add_image(ImageObject::png(0.0, 0.0, 1.0, 1.0, vec![0x89]));
        page.add_path(PathObject::vline(5.0, 0.0, 10.0));
        assert_eq!(page.content.len(), 3);
    }

    #[test]
    fn test_ofd_page_clone_debug() {
        let page = OfdPage::new(100.0, 200.0);
        let page2 = page.clone();
        assert!((page2.width - 100.0).abs() < f64::EPSILON);
        let dbg = format!("{page:?}");
        assert!(dbg.contains("OfdPage"));
    }

    // ─── page_size constants ──────────────────────────────────────────────────

    #[test]
    fn test_page_size_a4() {
        assert_eq!(page_size::A4, (210.0, 297.0));
    }

    #[test]
    fn test_page_size_a4_landscape() {
        assert_eq!(page_size::A4_LANDSCAPE, (297.0, 210.0));
    }

    #[test]
    fn test_page_size_a3() {
        assert_eq!(page_size::A3, (297.0, 420.0));
    }

    #[test]
    fn test_page_size_letter() {
        assert_eq!(page_size::LETTER, (215.9, 279.4));
    }

    // ─── TextObject ───────────────────────────────────────────────────────────

    #[test]
    fn test_text_object_new() {
        let t = TextObject::new(10.0, 20.0, "hello");
        assert!((t.x - 10.0).abs() < f64::EPSILON);
        assert!((t.y - 20.0).abs() < f64::EPSILON);
        assert_eq!(t.text, "hello");
        assert_eq!(t.font, "SimSun");
        assert!((t.size - 12.0).abs() < f64::EPSILON);
        assert_eq!(t.weight, 400);
        assert!(!t.italic);
        assert_eq!(t.color, 0);
        assert!(t.width.is_none());
        assert!(t.height.is_none());
    }

    #[test]
    fn test_text_object_from_string() {
        let s = String::from("owned");
        let t = TextObject::new(0.0, 0.0, s);
        assert_eq!(t.text, "owned");
    }

    #[test]
    fn test_text_object_builder_font() {
        let t = TextObject::new(0.0, 0.0, "x").font("SimHei");
        assert_eq!(t.font, "SimHei");
    }

    #[test]
    fn test_text_object_builder_size() {
        let t = TextObject::new(0.0, 0.0, "x").size(24.0);
        assert!((t.size - 24.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_text_object_builder_bold() {
        let t = TextObject::new(0.0, 0.0, "x").bold();
        assert_eq!(t.weight, 700);
    }

    #[test]
    fn test_text_object_builder_italic() {
        let t = TextObject::new(0.0, 0.0, "x").italic();
        assert!(t.italic);
    }

    #[test]
    fn test_text_object_builder_color() {
        let t = TextObject::new(0.0, 0.0, "x").color(0xFF_0000);
        assert_eq!(t.color, 0xFF_0000);
    }

    #[test]
    fn test_text_object_builder_chaining() {
        let t = TextObject::new(1.0, 2.0, "x")
            .font("Arial")
            .size(16.0)
            .bold()
            .italic()
            .color(0x00_FF00);
        assert_eq!(t.font, "Arial");
        assert!((t.size - 16.0).abs() < f64::EPSILON);
        assert_eq!(t.weight, 700);
        assert!(t.italic);
        assert_eq!(t.color, 0x00_FF00);
    }

    #[test]
    fn test_text_object_clone_debug() {
        let t = TextObject::new(0.0, 0.0, "x");
        let t2 = t.clone();
        assert_eq!(t2.text, "x");
        let dbg = format!("{t:?}");
        assert!(dbg.contains("TextObject"));
    }

    // ─── ImageFormat ──────────────────────────────────────────────────────────

    #[test]
    fn test_image_format_variants() {
        assert_ne!(ImageFormat::Jpeg, ImageFormat::Png);
        assert_ne!(ImageFormat::Bmp, ImageFormat::Tiff);
        assert_eq!(ImageFormat::Jpeg, ImageFormat::Jpeg);
    }

    #[test]
    fn test_image_format_clone_copy_debug() {
        let f = ImageFormat::Png;
        let f2 = f;
        assert_eq!(f2, ImageFormat::Png);
        let dbg = format!("{f:?}");
        assert!(dbg.contains("Png"));
    }

    // ─── ImageObject ──────────────────────────────────────────────────────────

    #[test]
    fn test_image_object_new() {
        let img = ImageObject::new(10.0, 20.0, 30.0, 40.0, vec![1, 2], ImageFormat::Png);
        assert!((img.x - 10.0).abs() < f64::EPSILON);
        assert!((img.y - 20.0).abs() < f64::EPSILON);
        assert!((img.width - 30.0).abs() < f64::EPSILON);
        assert!((img.height - 40.0).abs() < f64::EPSILON);
        assert_eq!(img.data, vec![1, 2]);
        assert_eq!(img.format, ImageFormat::Png);
    }

    #[test]
    fn test_image_object_jpeg() {
        let img = ImageObject::jpeg(0.0, 0.0, 10.0, 10.0, vec![0xFF]);
        assert_eq!(img.format, ImageFormat::Jpeg);
    }

    #[test]
    fn test_image_object_png() {
        let img = ImageObject::png(0.0, 0.0, 10.0, 10.0, vec![0x89]);
        assert_eq!(img.format, ImageFormat::Png);
    }

    #[test]
    fn test_image_object_clone_debug() {
        let img = ImageObject::jpeg(0.0, 0.0, 1.0, 1.0, vec![0]);
        let img2 = img.clone();
        assert_eq!(img2.data, vec![0]);
        let dbg = format!("{img:?}");
        assert!(dbg.contains("ImageObject"));
    }

    // ─── PathObject ───────────────────────────────────────────────────────────

    #[test]
    fn test_path_object_new() {
        let p = PathObject::new(5.0, 10.0, "M0 0L10 10");
        assert!((p.x - 5.0).abs() < f64::EPSILON);
        assert!((p.y - 10.0).abs() < f64::EPSILON);
        assert_eq!(p.path_data, "M0 0L10 10");
        assert_eq!(p.stroke_color, 0);
        assert!((p.stroke_width - 0.35).abs() < f64::EPSILON);
        assert!(p.fill_color.is_none());
    }

    #[test]
    fn test_path_object_hline() {
        let p = PathObject::hline(10.0, 20.0, 100.0);
        assert!((p.x - 10.0).abs() < f64::EPSILON);
        assert!((p.y - 20.0).abs() < f64::EPSILON);
        assert!(p.path_data.contains("M10"));
        assert!(p.path_data.contains("L100"));
    }

    #[test]
    fn test_path_object_vline() {
        let p = PathObject::vline(5.0, 0.0, 50.0);
        assert!((p.x - 5.0).abs() < f64::EPSILON);
        assert!(p.path_data.contains("M5"));
    }

    #[test]
    fn test_path_object_rect() {
        let p = PathObject::rect(0.0, 0.0, 100.0, 50.0);
        assert!(p.path_data.starts_with('M'));
        assert!(p.path_data.ends_with('Z'));
        assert!(p.path_data.contains("L100"));
    }

    #[test]
    fn test_path_object_builder_stroke_color() {
        let p = PathObject::new(0.0, 0.0, "M0 0").stroke_color(0xFF_0000);
        assert_eq!(p.stroke_color, 0xFF_0000);
    }

    #[test]
    fn test_path_object_builder_stroke_width() {
        let p = PathObject::new(0.0, 0.0, "M0 0").stroke_width(1.5);
        assert!((p.stroke_width - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_path_object_builder_fill_color() {
        let p = PathObject::new(0.0, 0.0, "M0 0").fill_color(0x00_FF00);
        assert_eq!(p.fill_color, Some(0x00_FF00));
    }

    #[test]
    fn test_path_object_builder_chaining() {
        let p = PathObject::new(0.0, 0.0, "M0 0")
            .stroke_color(0xFF)
            .stroke_width(2.0)
            .fill_color(0xFF_FF00);
        assert_eq!(p.stroke_color, 0xFF);
        assert!((p.stroke_width - 2.0).abs() < f64::EPSILON);
        assert_eq!(p.fill_color, Some(0xFF_FF00));
    }

    #[test]
    fn test_path_object_clone_debug() {
        let p = PathObject::new(0.0, 0.0, "M0 0");
        let p2 = p.clone();
        assert_eq!(p2.path_data, "M0 0");
        let dbg = format!("{p:?}");
        assert!(dbg.contains("PathObject"));
    }

    // ─── ContentObject ────────────────────────────────────────────────────────

    #[test]
    fn test_content_object_variants_debug() {
        let text = ContentObject::Text(TextObject::new(0.0, 0.0, "x"));
        let img = ContentObject::Image(ImageObject::jpeg(0.0, 0.0, 1.0, 1.0, vec![0]));
        let path = ContentObject::Path(PathObject::new(0.0, 0.0, "M0 0"));
        assert!(format!("{text:?}").contains("Text"));
        assert!(format!("{img:?}").contains("Image"));
        assert!(format!("{path:?}").contains("Path"));
    }

    // ─── ImageObject::from_file ─────────────────────────────────────────────

    #[test]
    fn test_image_from_file_png() {
        // Create a minimal PNG file
        let dir = std::env::temp_dir().join("easyofd_img_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.png");
        // Minimal valid PNG: 8-byte signature + IHDR chunk (13 bytes) + IEND
        let png: Vec<u8> = vec![
            0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, // signature
        ];
        std::fs::write(&path, &png).unwrap();

        let img = ImageObject::from_file(0.0, 0.0, 10.0, 10.0, &path).unwrap();
        assert_eq!(img.format, ImageFormat::Png);
        assert!(!img.data.is_empty());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_image_from_file_jpg_by_ext() {
        let dir = std::env::temp_dir().join("easyofd_img_test2");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.jpg");
        std::fs::write(&path, b"not-real-jpg").unwrap();

        let img = ImageObject::from_file(0.0, 0.0, 10.0, 10.0, &path).unwrap();
        assert_eq!(img.format, ImageFormat::Jpeg);
        let _ = std::fs::remove_file(&path);
    }

    // ─── Watermark ──────────────────────────────────────────────────────────

    #[test]
    fn test_watermark_text_defaults() {
        let wm = Watermark::text("CONFIDENTIAL");
        assert_eq!(wm.text.as_deref(), Some("CONFIDENTIAL"));
        assert_eq!(wm.font_size, 48.0);
        assert_eq!(wm.opacity, 0.3);
        assert_eq!(wm.rotation, 45.0);
    }

    #[test]
    fn test_watermark_builder() {
        let wm = Watermark::text("DRAFT")
            .position(50.0, 100.0)
            .font_size(36.0)
            .opacity(0.2)
            .rotation(0.0)
            .on_page(0);
        assert_eq!(wm.position, (50.0, 100.0));
        assert_eq!(wm.page, Some(0));
    }

    #[test]
    fn test_watermark_image() {
        let wm = Watermark::image(vec![0xFF, 0xD8]);
        assert!(wm.image.is_some());
        assert!(wm.text.is_none());
    }

    #[test]
    fn test_watermark_on_all_pages() {
        let wm = Watermark::text("DRAFT").on_all_pages();
        assert!(wm.page.is_none());
    }
}
