//! Integration tests for the `OfdModel` derive macro and end-to-end write flow.
#![allow(clippy::float_cmp)]

use easyofd::{EasyOfd, ImageObject, OfdModel, OfdPage, PathObject, TextObject};

// ═══════════════════════════════════════════════════════════════════════════════
// Derive macro: basic struct with text fields
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(OfdModel)]
#[ofd(page_width = 210.0, page_height = 297.0)]
struct Invoice {
    #[ofd(x = 20.0, y = 30.0, size = 18.0, bold)]
    title: String,
    #[ofd(x = 20.0, y = 50.0)]
    amount: String,
    #[ofd(x = 20.0, y = 70.0)]
    note: String,
}

#[test]
fn test_derive_schema() {
    let schema = Invoice::schema();
    assert_eq!(schema.len(), 3);
    assert_eq!(schema[0].name, "title");
    assert_eq!(schema[0].position, (20.0, 30.0));
    assert_eq!(schema[0].size, 18.0);
    assert_eq!(schema[0].weight, 700);
    assert!(!schema[0].italic);
    assert_eq!(schema[0].color, 0);
    assert_eq!(schema[1].name, "amount");
    assert_eq!(schema[1].position, (20.0, 50.0));
    assert_eq!(schema[1].size, 12.0); // default
    assert_eq!(schema[1].weight, 400); // default
    assert_eq!(schema[2].name, "note");
}

#[test]
fn test_derive_page_size() {
    assert_eq!(Invoice::page_size(), (210.0, 297.0));
}

#[test]
fn test_derive_to_page() {
    let invoice = Invoice {
        title: "Invoice #001".into(),
        amount: "$1,234.56".into(),
        note: "Thank you".into(),
    };
    let page = invoice.to_page().unwrap();
    assert_eq!(page.width, 210.0);
    assert_eq!(page.height, 297.0);
    assert_eq!(page.content.len(), 3);
}

#[test]
fn test_derive_to_pages() {
    let invoices = vec![
        Invoice {
            title: "Invoice #001".into(),
            amount: "$100.00".into(),
            note: "First".into(),
        },
        Invoice {
            title: "Invoice #002".into(),
            amount: "$200.00".into(),
            note: "Second".into(),
        },
    ];
    let pages = Invoice::to_pages(&invoices).unwrap();
    assert_eq!(pages.len(), 2);
}

#[test]
fn test_derive_to_pages_empty() {
    let items: Vec<Invoice> = vec![];
    let pages = Invoice::to_pages(&items).unwrap();
    assert!(pages.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derive macro: all attribute combinations
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(OfdModel)]
#[ofd(page_width = 297.0, page_height = 210.0)]
struct FullAttrs {
    #[ofd(x = 10.0, y = 20.0, font = "SimHei", size = 24.0, weight = 700, italic, color = 16711680)]
    header: String,
    #[ofd(x = 10.0, y = 50.0, font = "Arial", size = 14.0)]
    body: String,
    #[ofd(x = 10.0, y = 80.0, bold, italic)]
    emphasized: String,
    #[ofd(ignore)]
    _skipped: String,
}

#[test]
fn test_full_attrs_schema_excludes_ignored() {
    let schema = FullAttrs::schema();
    assert_eq!(schema.len(), 3); // _skipped is ignored
    assert_eq!(schema[0].name, "header");
    assert_eq!(schema[0].font, "SimHei");
    assert_eq!(schema[0].size, 24.0);
    assert_eq!(schema[0].weight, 700);
    assert!(schema[0].italic);
    assert_eq!(schema[0].color, 16_711_680);
}

#[test]
fn test_full_attrs_page_size() {
    assert_eq!(FullAttrs::page_size(), (297.0, 210.0));
}

#[test]
fn test_full_attrs_to_page_skips_ignored() {
    let item = FullAttrs {
        header: "H".into(),
        body: "B".into(),
        emphasized: "E".into(),
        _skipped: "should not appear".into(),
    };
    let page = item.to_page().unwrap();
    assert_eq!(page.content.len(), 3); // _skipped excluded
    assert!((page.width - 297.0).abs() < f64::EPSILON);
    assert!((page.height - 210.0).abs() < f64::EPSILON);
}

#[test]
fn test_full_attrs_italic_bold_combined() {
    let schema = FullAttrs::schema();
    // emphasized has both bold and italic
    assert_eq!(schema[2].name, "emphasized");
    assert_eq!(schema[2].weight, 700);
    assert!(schema[2].italic);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derive macro: image field
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(OfdModel)]
#[ofd(page_width = 210.0, page_height = 297.0)]
struct WithImage {
    #[ofd(x = 20.0, y = 30.0)]
    title: String,
    #[ofd(x = 150.0, y = 30.0, kind = "image", img_width = 40.0, img_height = 40.0)]
    seal: Vec<u8>,
}

#[test]
fn test_image_field_schema() {
    let schema = WithImage::schema();
    assert_eq!(schema.len(), 2);
    assert_eq!(schema[0].kind, easyofd::OfdFieldKind::Text);
    assert_eq!(schema[1].kind, easyofd::OfdFieldKind::Image);
    assert_eq!(schema[1].name, "seal");
}

#[test]
fn test_image_field_to_page() {
    let item = WithImage {
        title: "Contract".into(),
        seal: vec![0xFF, 0xD8, 0xFF, 0xE0],
    };
    let page = item.to_page().unwrap();
    assert_eq!(page.content.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derive macro: with non-ofd attributes (covers continue branches)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(OfdModel)]
#[ofd(page_width = 210.0, page_height = 297.0)]
#[allow(dead_code)]
struct WithNonOfdAttrs {
    #[ofd(x = 10.0, y = 20.0)]
    #[allow(dead_code)]
    text: String,
}

#[test]
fn test_non_ofd_attrs_ignored() {
    let item = WithNonOfdAttrs { text: "hello".into() };
    let page = item.to_page().unwrap();
    assert_eq!(page.content.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derive macro: float literal values (covers lit_to_u32 float branch)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(OfdModel)]
struct FloatWeight {
    #[ofd(x = 0.0, y = 0.0, weight = 400.0, color = 255.0)]
    text: String,
}

#[test]
fn test_float_weight_and_color() {
    let item = FloatWeight { text: "x".into() };
    let page = item.to_page().unwrap();
    assert_eq!(page.content.len(), 1);
    let schema = FloatWeight::schema();
    assert_eq!(schema[0].weight, 400);
    assert_eq!(schema[0].color, 255);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derive macro: float img_width/img_height
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(OfdModel)]
struct FloatImgSize {
    #[ofd(x = 0.0, y = 0.0)]
    text: String,
    #[ofd(x = 50.0, y = 50.0, kind = "image", img_width = 50.0, img_height = 60.0)]
    seal: Vec<u8>,
}

#[test]
fn test_float_img_size() {
    let item = FloatImgSize { text: "x".into(), seal: vec![0xFF] };
    let page = item.to_page().unwrap();
    assert_eq!(page.content.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derive macro: only page_width (test else-if fallthrough for page_height)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(OfdModel)]
#[ofd(page_width = 150.0)]
struct OnlyWidth {
    #[ofd(x = 0.0, y = 0.0)]
    text: String,
}

#[test]
fn test_only_page_width_specified() {
    // page_height defaults to 297.0 when not specified
    assert_eq!(OnlyWidth::page_size(), (150.0, 297.0));
    let item = OnlyWidth { text: "x".into() };
    let page = item.to_page().unwrap();
    assert!((page.width - 150.0).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derive macro: unknown attribute names (covers match catch-all branches)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(OfdModel)]
#[ofd(custom_note, page_width = 210.0, page_height = 297.0)]
struct WithUnknownStructAttr {
    #[ofd(unknown_tag, x = 10.0, y = 20.0)]
    text: String,
}

#[test]
fn test_unknown_attrs_ignored() {
    let item = WithUnknownStructAttr { text: "hello".into() };
    let page = item.to_page().unwrap();
    assert_eq!(page.content.len(), 1);
    assert_eq!(WithUnknownStructAttr::page_size(), (210.0, 297.0));
}

#[derive(OfdModel)]
#[ofd(page_width = 200, page_height = 300)]
struct IntPageDims {
    #[ofd(x = 10, y = 20, size = 14)]
    text: String,
}

#[test]
fn test_integer_page_dimensions() {
    assert_eq!(IntPageDims::page_size(), (200.0, 300.0));
    let schema = IntPageDims::schema();
    assert_eq!(schema[0].position, (10.0, 20.0));
    assert_eq!(schema[0].size, 14.0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derive macro: default page dimensions (no page_width/page_height)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(OfdModel)]
struct DefaultPageDims {
    #[ofd(x = 0.0, y = 0.0)]
    text: String,
}

#[test]
fn test_default_page_dimensions() {
    assert_eq!(DefaultPageDims::page_size(), (210.0, 297.0));
}

// ═══════════════════════════════════════════════════════════════════════════════
// End-to-end: write with derive model
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_easyofd_write_derive_model() {
    let data = vec![
        Invoice {
            title: "Invoice #001".into(),
            amount: "$100.00".into(),
            note: "First invoice".into(),
        },
        Invoice {
            title: "Invoice #002".into(),
            amount: "$200.00".into(),
            note: "Second invoice".into(),
        },
    ];

    let bytes = EasyOfd::write::<Invoice>("test_derive.ofd")
        .metadata_title("Invoices")
        .metadata_author("easyofd-rs")
        .do_write_to_bytes(&data)
        .unwrap();

    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..2], b"PK");

    let names = zip_entry_names(&bytes);
    assert!(names.contains(&"OFD.xml".to_string()));
    assert!(names.contains(&"Doc_0/Document.xml".to_string()));
    assert!(names.contains(&"Doc_0/Pages/Page_0.xml".to_string()));
    assert!(names.contains(&"Doc_0/Pages/Page_1.xml".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// End-to-end: manual page construction
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_manual_page_construction() {
    let mut page = OfdPage::new(297.0, 210.0);
    page.add_text(TextObject::new(20.0, 30.0, "Manual Page Title").size(24.0).bold());
    page.add_text(TextObject::new(20.0, 60.0, "Body text goes here"));
    page.add_path(PathObject::hline(20.0, 55.0, 277.0));
    page.add_image(ImageObject::jpeg(
        200.0, 30.0, 50.0, 50.0, vec![0xFF, 0xD8, 0xFF, 0xE0],
    ));

    let bytes = EasyOfd::write_pages("manual.ofd")
        .metadata_title("Manual Construction")
        .do_write_to_bytes(vec![page])
        .unwrap();

    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..2], b"PK");
}

// ═══════════════════════════════════════════════════════════════════════════════
// End-to-end: multi-page document
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_multi_page_document() {
    let mut pages = Vec::new();
    for i in 0..5 {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, format!("Page {} of 5", i + 1)));
        pages.push(page);
    }

    let bytes = EasyOfd::write_pages_to_bytes(pages).unwrap();
    let names = zip_entry_names(&bytes);

    for i in 0..5 {
        assert!(
            names.contains(&format!("Doc_0/Pages/Page_{i}.xml")),
            "missing Page_{i}.xml"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EasyOfd facade: all builder methods
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_write_builder_all_metadata() {
    let data = vec![Invoice {
        title: "T".into(),
        amount: "A".into(),
        note: "N".into(),
    }];
    let bytes = EasyOfd::write::<Invoice>("x.ofd")
        .metadata_title("Title")
        .metadata_author("Author")
        .metadata_creator("Creator")
        .do_write_to_bytes(&data)
        .unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_write_builder_do_write_to_file() {
    let dir = std::env::temp_dir().join("easyofd_facade_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("builder_write.ofd");

    let data = vec![Invoice {
        title: "T".into(),
        amount: "A".into(),
        note: "N".into(),
    }];
    EasyOfd::write::<Invoice>(path.to_string_lossy().into_owned())
        .do_write(&data)
        .unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[0..2], b"PK");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_pages_builder_all_metadata() {
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
fn test_pages_builder_do_write_to_file() {
    let dir = std::env::temp_dir().join("easyofd_facade_test2");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("pages_write.ofd");

    let page = OfdPage::new(210.0, 297.0);
    EasyOfd::write_pages(path.to_string_lossy().into_owned())
        .do_write(vec![page])
        .unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[0..2], b"PK");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_write_pages_to_static() {
    let dir = std::env::temp_dir().join("easyofd_facade_test3");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("static_write.ofd");

    let page = OfdPage::new(210.0, 297.0);
    EasyOfd::write_pages_to(&path, vec![page]).unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[0..2], b"PK");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_write_pages_to_bytes_static() {
    let page = OfdPage::new(210.0, 297.0);
    let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..2], b"PK");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_page() {
    let page = OfdPage::new(210.0, 297.0);
    let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_no_pages() {
    let bytes = EasyOfd::write_pages_to_bytes(vec![]).unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..2], b"PK");
}

#[test]
fn test_large_text_content() {
    let big_text = "x".repeat(10_000);
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(TextObject::new(0.0, 0.0, &big_text));
    let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_special_xml_chars_in_content() {
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(TextObject::new(0.0, 0.0, "<tag>&\"'amp"));
    let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_empty_string_text() {
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(TextObject::new(0.0, 0.0, ""));
    let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_multiple_images_same_page() {
    let mut page = OfdPage::new(210.0, 297.0);
    for i in 0..5 {
        page.add_image(ImageObject::jpeg(
            f64::from(i) * 40.0, 0.0, 30.0, 30.0, vec![0xFF],
        ));
    }
    let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
    let names = zip_entry_names(&bytes);
    for i in 0..5 {
        assert!(
            names.contains(&format!("Doc_0/Res/Image_{i}.jpeg")),
            "missing Image_{i}.jpeg"
        );
    }
}

#[test]
fn test_path_with_stroke_color_hex() {
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_path(PathObject::new(0.0, 0.0, "M0 0L100 100").stroke_color(0xAB_CD_EF));
    let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
    assert!(!bytes.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn zip_entry_names(bytes: &[u8]) -> Vec<String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect()
}
