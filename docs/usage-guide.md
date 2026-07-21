# easyofd-rs Usage Guide 使用指南

> Step-by-step examples for all easyofd-rs operations.

---

## Installation 安装

```toml
[dependencies]
easyofd = "0.4"
```

---

## 1. Creating OFD Documents 创建 OFD 文档

### 1.1 Derive Macro Approach 派生宏方式

The fastest way: annotate a struct with `#[derive(OfdModel)]` and `#[ofd(...)]` attributes.

```rust
use easyofd::{EasyOfd, OfdModel};

#[derive(OfdModel)]
#[ofd(page_width = 210.0, page_height = 297.0)]
struct Certificate {
    #[ofd(x = 30.0, y = 20.0, font = "SimHei", size = 24.0, bold)]
    title: String,

    #[ofd(x = 30.0, y = 50.0, font = "SimSun", size = 14.0)]
    recipient: String,

    #[ofd(x = 30.0, y = 70.0, size = 14.0)]
    date: String,

    #[ofd(x = 150.0, y = 20.0, kind = "image", img_width = 40.0, img_height = 40.0)]
    seal: Vec<u8>,

    #[ofd(ignore)]
    internal_tracking_id: u64,
}

fn generate_certificate() -> easyofd::OfdResult<()> {
    let seals = std::fs::read("seal.png")?;

    let data = vec![
        Certificate {
            title: "Certificate of Completion".into(),
            recipient: "Alice Wang".into(),
            date: "2026-07-21".into(),
            seal: seals.clone(),
            internal_tracking_id: 1,
        },
        Certificate {
            title: "Certificate of Excellence".into(),
            recipient: "Bob Li".into(),
            date: "2026-07-21".into(),
            seal: seals,
            internal_tracking_id: 2,
        },
    ];

    // Each data item → one page
    EasyOfd::write::<Certificate>("certificates.ofd")
        .metadata_title("Certificates")
        .metadata_author("Training Dept")
        .metadata_creator("easyofd-rs")
        .do_write(&data)?;

    Ok(())
}
```

### 1.2 Manual Page Construction 手动页面构建

For maximum control over page layout:

```rust
use easyofd::{EasyOfd, OfdPage, TextObject, ImageObject, PathObject, page_size};

fn create_invoice() -> easyofd::OfdResult<()> {
    let mut page = OfdPage::new(page_size::A4.0, page_size::A4.1);

    // Title
    page.add_text(
        TextObject::new(20.0, 30.0, "INVOICE")
            .font("SimHei")
            .size(24.0)
            .bold()
    );

    // Separator line
    page.add_path(
        PathObject::hline(20.0, 55.0, 190.0)
            .stroke_color(0x333333)
            .stroke_width(0.5)
    );

    // Table header
    page.add_text(TextObject::new(20.0, 60.0, "Item").bold());
    page.add_text(TextObject::new(100.0, 60.0, "Qty").bold());
    page.add_text(TextObject::new(140.0, 60.0, "Price").bold());

    // Table rows
    page.add_text(TextObject::new(20.0, 75.0, "Widget A"));
    page.add_text(TextObject::new(100.0, 75.0, "10"));
    page.add_text(TextObject::new(140.0, 75.0, "$50.00"));

    page.add_text(TextObject::new(20.0, 88.0, "Widget B"));
    page.add_text(TextObject::new(100.0, 88.0, "5"));
    page.add_text(TextObject::new(140.0, 88.0, "$120.00"));

    // Bottom line
    page.add_path(PathObject::hline(20.0, 100.0, 190.0));

    // Total
    page.add_text(
        TextObject::new(100.0, 105.0, "Total: $1,100.00")
            .bold()
    );

    // Company seal
    page.add_image(ImageObject::jpeg(
        150.0, 200.0, 40.0, 40.0,
        std::fs::read("company_seal.jpg")?,
    ));

    // Footer box
    page.add_path(
        PathObject::rect(10.0, 270.0, 190.0, 20.0)
            .stroke_color(0xCCCCCC)
    );
    page.add_text(TextObject::new(20.0, 275.0, "Thank you for your business!").size(10.0));

    EasyOfd::write_pages("invoice.ofd")
        .metadata_title("Invoice #001")
        .do_write(vec![page])
}
```

### 1.3 Multi-Page Documents 多页文档

```rust
fn create_report() -> easyofd::OfdResult<()> {
    let mut pages = Vec::new();

    // Cover page
    let mut cover = OfdPage::new(210.0, 297.0);
    cover.add_text(TextObject::new(50.0, 130.0, "Annual Report 2025")
        .font("SimHei").size(28.0).bold());
    cover.add_text(TextObject::new(50.0, 160.0, "Confidential").size(14.0));
    pages.push(cover);

    // Content pages
    for i in 1..=5 {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, format!("Section {i}")).size(18.0).bold());
        page.add_text(TextObject::new(20.0, 55.0,
            format!("Content for section {i} goes here...")));
        pages.push(page);
    }

    EasyOfd::write_pages("report.ofd")
        .metadata_title("Annual Report 2025")
        .do_write(pages)
}
```

---

## 2. Reading OFD Documents 读取 OFD 文档

### 2.1 Basic Text Extraction

```rust
use easyofd::EasyOfd;

let reader = EasyOfd::read("document.ofd")?;
println!("Total pages: {}", reader.page_count());

for (i, text) in reader.extract_text().iter().enumerate() {
    println!("--- Page {} ---", i + 1);
    println!("{text}");
}
```

### 2.2 Structured Content Access

```rust
use easyofd::{EasyOfd, ContentObject};

let reader = EasyOfd::read("document.ofd")?;

for page in reader.pages() {
    println!("Page: {} × {} mm", page.width, page.height);
    for obj in &page.content {
        match obj {
            ContentObject::Text(t) => {
                println!("  Text at ({}, {}): {}", t.x, t.y, t.text);
            }
            ContentObject::Image(img) => {
                println!("  Image at ({}, {}): {} × {} mm, format: {:?}",
                    img.x, img.y, img.width, img.height, img.format);
            }
            ContentObject::Path(_) => {
                println!("  Path object");
            }
        }
    }
}
```

### 2.3 Processing from Bytes

```rust
let uploaded_bytes: Vec<u8> = get_from_network()?;
let reader = EasyOfd::read_from_bytes(&uploaded_bytes)?;
let all_text = reader.extract_all_text();
```

---

## 3. Template Filling 模板填充

### 3.1 Creating a Template

First, create a template OFD with `{placeholder}` patterns:

```rust
// When creating the template, use {placeholders} in text:
let mut page = OfdPage::new(210.0, 297.0);
page.add_text(TextObject::new(20.0, 30.0, "Invoice: {title}").size(18.0).bold());
page.add_text(TextObject::new(20.0, 60.0, "Amount: {amount}"));
page.add_text(TextObject::new(20.0, 80.0, "Date: {date}"));
page.add_text(TextObject::new(20.0, 100.0, "{notes}"));

let mut writer = OfdWriter::new();
writer.add_page(page);
writer.build_to_file("template.ofd")?;
```

### 3.2 Filling the Template

```rust
use std::collections::HashMap;

let mut data = HashMap::new();
data.insert("title".into(), "INV-2026-0042".into());
data.insert("amount".into(), "$8,500.00".into());
data.insert("date".into(), "2026-07-21".into());
data.insert("notes".into(), "Payment due within 30 days.".into());

let filler = EasyOfd::fill_template("template.ofd", &data)?;
filler.save("invoice-0042.ofd")?;
```

### 3.3 Handling Missing Keys

Keys not present in the data map are **preserved as-is** in the output:

```rust
let mut data = HashMap::new();
data.insert("name".into(), "Alice".into());
// "address" not provided → "{address}" stays in output
```

### 3.4 Batch Filling

```rust
fn batch_generate_invoices(invoices: &[(String, HashMap<String, String>)]) -> easyofd::OfdResult<()> {
    for (filename, data) in invoices {
        EasyOfd::fill_template("template.ofd", data)?
            .save(filename)?;
    }
    Ok(())
}
```

---

## 4. Electronic Signatures 电子签章

### 4.1 Adding a Visual Seal

```rust
use easyofd::{OfdSignatureBuilder, ElectronicSeal};

let seal = ElectronicSeal {
    image_data: std::fs::read("company_seal.png")?,
    name: "Company Official Seal".into(),
    position: (150.0, 200.0),  // mm from top-left
    page: 1,                     // 1-based page number
};

OfdSignatureBuilder::new("contract.ofd")
    .seal(seal)
    .sign()?
    .save("contract-signed.ofd")?;
```

### 4.2 Multiple Seals

```rust
OfdSignatureBuilder::new("contract.ofd")
    .seal(company_seal)
    .seal(supervisor_seal)
    .seal(auditor_seal)
    .sign()?
    .save("fully-sealed.ofd")?;
```

### 4.3 Specifying Algorithm

```rust
use easyofd::SignatureAlgorithm;

OfdSignatureBuilder::new("document.ofd")
    .seal(my_seal)
    .algorithm(SignatureAlgorithm::Sha256WithRsa)
    .sign()?
    .save("signed.ofd")?;
```

### 4.4 With Certificate (Full Cryptographic Signing)

```rust
let cert = std::fs::read("certificate.pem")?;
let key = std::fs::read("private_key.pem")?;

OfdSignatureBuilder::new("document.ofd")
    .seal(my_seal)
    .certificate(cert)
    .private_key(key)
    .sign()?     // ← generates actual Signature.xml with certificate info
    .save("fully-signed.ofd")?;
```

---

## 5. Custom Fonts 自定义字体

```rust
use easyofd::{EmbeddedFont, FontFormat};

let mut writer = OfdWriter::new();

// Register a TTF font that can be referenced in TextObject::font()
writer.embed_font(EmbeddedFont {
    name: "MyCustomFont".into(),
    data: std::fs::read("my-font.ttf")?,
    format: FontFormat::TrueType,
});

let mut page = OfdPage::new(210.0, 297.0);
page.add_text(TextObject::new(20.0, 30.0, "Custom font text")
    .font("MyCustomFont")
    .size(16.0));
writer.add_page(page);
writer.build_to_file("custom-font.ofd")?;
```

---

## 6. Common Patterns 常见模式

### 6.1 Write → Read Roundtrip

```rust
// Create
let mut page = OfdPage::new(210.0, 297.0);
page.add_text(TextObject::new(10.0, 20.0, "Roundtrip test"));
let bytes = EasyOfd::write_pages_to_bytes(vec![page])?;

// Read back
let reader = EasyOfd::read_from_bytes(&bytes)?;
assert_eq!(reader.page_count(), 1);
assert!(reader.extract_all_text().contains("Roundtrip test"));
```

### 6.2 Write → Sign → Read Pipeline

```rust
// Step 1: Create document
EasyOfd::write_pages("draft.ofd").do_write(pages)?;

// Step 2: Apply electronic seal
OfdSignatureBuilder::new("draft.ofd")
    .seal(my_seal)
    .sign()?
    .save("final.ofd")?;

// Step 3: Verify
let reader = EasyOfd::read("final.ofd")?;
println!("Signed document: {} pages", reader.page_count());
```

### 6.3 Template → Fill → Sign Pipeline

```rust
// Prepare data
let mut data = HashMap::new();
data.insert("contract_number".into(), "CT-2026-0042".into());
data.insert("party_a".into(), "Company A Ltd.".into());
data.insert("party_b".into(), "Company B Inc.".into());

// Fill template
let filler = EasyOfd::fill_template("contract_template.ofd", &data)?;
let filled_bytes = filler.into_bytes();

// Save intermediate
std::fs::write("contract-filled.ofd", &filled_bytes)?;

// Sign
OfdSignatureBuilder::new("contract-filled.ofd")
    .seal(company_seal)
    .sign()?
    .save("contract-executed.ofd")?;
```

---

## 7. Error Handling 错误处理

All operations return `OfdResult<T>`:

```rust
use easyofd::{EasyOfd, OfdError};

fn process_ofd(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match EasyOfd::read(path) {
        Ok(reader) => {
            println!("Success: {} pages", reader.page_count());
            println!("{}", reader.extract_all_text());
            Ok(())
        }
        Err(OfdError::Io(e)) => {
            eprintln!("File error: {e}");
            Err(e.into())
        }
        Err(OfdError::Zip(e)) => {
            eprintln!("Not a valid OFD/ZIP: {e}");
            Err(e.into())
        }
        Err(OfdError::Xml(e)) => {
            eprintln!("XML parse error: {e}");
            Err(e.into())
        }
        Err(e) => {
            eprintln!("Other error: {e}");
            Err(e.into())
        }
    }
}
```

---

## 8. Page Size Reference 页面尺寸参考

| Constant 常量 | Dimensions (mm) | Use |
|:---|:---|:---|
| `page_size::A4` | 210 × 297 | Standard document |
| `page_size::A4_LANDSCAPE` | 297 × 210 | Wide tables |
| `page_size::A3` | 297 × 420 | Large diagrams |
| `page_size::LETTER` | 215.9 × 279.4 | US Letter |
| Custom | any (w, h) | OfdPage::new(w, h) |

---

## 9. Best Practices 最佳实践

| Practice | Why |
|:---|:---|
| Use derive macro for repetitive documents | Automatically maps struct → OFD, zero boilerplate |
| Use manual construction for complex layouts | Full control over every element position |
| Embed seal images as PNG | Smaller size, transparent background support |
| Use `{placeholder}` in template text | Enables batch document generation |
| Validate page dimensions | Origin (0,0) is top-left; elements outside page may be clipped |
| Use `page_size::A4` constants | Avoids magic numbers |
| Always handle `OfdResult` | Proper error propagation for I/O, ZIP, and XML errors |
