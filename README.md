# easyofd-rs &middot; [English](#easyofd-rs) | [&#20013;&#25991;](#easyofd-rs-&#20013;&#25991;)

> **An idiomatic Rust library for quick OFD document operations.**  
> Inspired by [Alibaba EasyExcel](https://github.com/alibaba/easyexcel)'s builder-pattern API design.  
> **Pure Rust &middot; Zero unsafe &middot; Builder pattern &middot; GB/T 33190-2016 compliant**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance)

---

`easyofd-rs` provides a fluent, type-safe builder API for creating OFD (Open Fixed-layout Document) files. OFD is the Chinese national standard GB/T 33190-2016, widely used in government and enterprise document workflows &mdash; especially for electronic invoices, official documents, and archival purposes.

---

## Table of Contents | &#30446;&#24405;

- [Features &#21151;&#33021;](#features--&#21151;&#33021;)
- [Architecture &#26550;&#26500;](#architecture--&#26550;&#26500;)
- [Quick Start &#24555;&#36895;&#24320;&#22987;](#quick-start--&#24555;&#36895;&#24320;&#22987;)
  - [Derive Model + One-liner Write &#27966;&#29983;&#23439; + &#19968;&#34892;&#20889;&#20837;](#1-derive-model--one-liner-write--&#27966;&#29983;&#23439;--&#19968;&#34892;&#20889;&#20837;)
  - [Manual Page Construction &#25163;&#21160;&#26500;&#24314;&#39029;&#38754;](#2-manual-page-construction--&#25163;&#21160;&#26500;&#24314;&#39029;&#38754;)
  - [Multiple Pages with Images & Paths &#22810;&#39029;&#21547;&#22270;&#29255;&#21644;&#36335;&#24452;](#3-multiple-pages-with-images--paths--&#22810;&#39029;&#21547;&#22270;&#29255;&#21644;&#36335;&#24452;)
- [API Reference API &#21442;&#32771;](#api-reference--api-&#21442;&#32771;)
  - [EasyOfd Entry Points &#20837;&#21475;&#26041;&#27861;](#easyofd--entry-points)
  - [OfdWriterBuilder](#ofdwriterbuilder)
  - [PageWriterBuilder](#pagewriterbuilder)
  - [Core Types &#26680;&#24515;&#31867;&#22411;](#core-types--&#26680;&#24515;&#31867;&#22411;)
- [Design Principles &#35774;&#35745;&#21407;&#21017;](#design-principles--&#35774;&#35745;&#21407;&#21017;)
- [Roadmap &#36335;&#32447;&#22270;](#roadmap--&#36335;&#32447;&#22270;)
- [License &#35768;&#21487;&#35777;](#license--&#35768;&#21487;&#35777;)

---

## Features | &#21151;&#33021;

| Feature &#21151;&#33021; | Status &#29366;&#24577; | Description &#25551;&#36848; |
|:---|:---:|:---|
| Create OFD (text, metadata) | &#9989; v0.1 | &#21019;&#24314;&#21547;&#25991;&#26412;&#12289;&#20803;&#25968;&#25454;&#30340; OFD |
| `#[derive(OfdModel)]` macro | &#9989; v0.1 | &#32534;&#35793;&#26399;&#21453;&#23556;&#23439; |
| Fluent Builder API | &#9989; v0.1 | &#27969;&#24335; Builder &#27169;&#24335; |
| Multi-page support | &#9989; v0.1 | &#22810;&#39029;&#25903;&#25345; |
| Image embedding (JPEG/PNG/BMP/TIFF) | &#9989; v0.1 | &#22270;&#29255;&#23884;&#20837; |
| Vector paths (lines, rectangles) | &#9989; v0.1 | &#30690;&#37327;&#36335;&#24452;&#65288;&#30452;&#32447;&#12289;&#30697;&#24418;&#65289; |
| OFD Reader | &#128679; v0.2 | OFD &#35835;&#21462;&#35299;&#26512; |
| Template filling | &#128679; v0.2 | &#27169;&#26495;&#22635;&#20805; |
| Digital signatures (GB/T 38540) | &#128679; v0.3 | &#25968;&#23383;&#31614;&#31456; |
| Custom fonts (TTF/OTF) | &#128679; v0.3 | &#33258;&#23450;&#20041;&#23383;&#20307; |
| PDF &#8596; OFD conversion | &#128679; v0.4 | PDF/OFD &#20114;&#36716; |

---

## Architecture | &#26550;&#26500;

```
&#9492;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;
&#9474;                  easyofd                       &#9474;
&#9474;         (facade &middot; Builder entry points)        &#9474;
&#9474;    EasyOfd::write()  write_pages()  ...          &#9474;
&#9500;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9500;
&#9474; core  &#9474; derive &#9474; writer                       &#9474;
&#9474; types &#9474; macro  &#9474;  ZIP + XML (GB/T 33190)       &#9474;
&#9474; errors&#9474;#[derive&#9474;  thiserror, uuid, zip,       &#9474;
&#9474; traits&#9474;(OfdMod-  &#9474;  quick-xml, chrono          &#9474;
&#9474; model &#9474;  el)    &#9474;                              &#9474;
&#9496;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9496;
```

| Crate &#23376;&#21253; | Purpose &#29992;&#36884; | Dependencies &#20381;&#36182; |
|:---|:---|---|
| **easyofd** | Facade + Builder entry points &#22806;&#35266;&#20837;&#21475; | All sub-crates |
| **easyofd-core** | Types, traits, errors, model &#26680;&#24515;&#25277;&#35937; | thiserror, chrono |
| **easyofd-derive** | `#[derive(OfdModel)]` proc-macro &#32534;&#35793;&#26399;&#21453;&#23556; | syn, quote, proc-macro2 |
| **easyofd-writer** | OFD ZIP/XML writer &#21512;&#35268;&#20889;&#20837;&#22120; | easyofd-core, zip, uuid, quick-xml |

---

## Quick Start | &#24555;&#36895;&#24320;&#22987;

Add to your `Cargo.toml`:

```toml
[dependencies]
easyofd = "0.1"
```

### 1. Derive Model + One-liner Write | &#27966;&#29983;&#23439; + &#19968;&#34892;&#20889;&#20837;

```rust
use easyofd::{EasyOfd, OfdModel};

#[derive(OfdModel)]
#[ofd(page_width = 210.0, page_height = 297.0)]
struct Invoice {
    #[ofd(x = 20.0, y = 30.0, size = 18.0, bold)]
    title: String,

    #[ofd(x = 20.0, y = 50.0)]
    amount: String,

    #[ofd(x = 20.0, y = 70.0)]
    note: String,

    // Fields with `#[ofd(ignore)]` are excluded from the OFD output
    #[ofd(ignore)]
    internal_id: u64,
}

// One-liner: multiple data items -> multiple pages
let data = vec![
    Invoice { title: "Invoice #001".into(), amount: "$100.00".into(), note: "First".into(), internal_id: 1 },
    Invoice { title: "Invoice #002".into(), amount: "$200.00".into(), note: "Second".into(), internal_id: 2 },
];

EasyOfd::write::<Invoice>("invoices.ofd")
    .metadata_title("Invoices")
    .metadata_author("easyofd-rs")
    .do_write(&data)?;
```

### 2. Manual Page Construction | &#25163;&#21160;&#26500;&#24314;&#39029;&#38754;

```rust
use easyofd::{EasyOfd, OfdPage, TextObject, page_size};

let mut page = OfdPage::new(page_size::A4.0, page_size::A4.1);
page.add_text(
    TextObject::new(20.0, 30.0, "Hello OFD!")
        .font("SimHei")
        .size(24.0)
        .bold()
);

// Write a single page
EasyOfd::write_pages_to_bytes(vec![page])?;
```

### 3. Multiple Pages with Images & Paths | &#22810;&#39029;&#21547;&#22270;&#29255;&#21644;&#36335;&#24452;

```rust
use easyofd::{EasyOfd, OfdPage, TextObject, ImageObject, PathObject};

// Page 1: text + image + separator line
let mut page1 = OfdPage::new(210.0, 297.0);
page1.add_text(TextObject::new(20.0, 30.0, "Invoice").size(18.0).bold());
page1.add_path(PathObject::hline(20.0, 55.0, 190.0));
page1.add_text(TextObject::new(20.0, 60.0, "Item A  ......  $100.00"));
page1.add_image(ImageObject::jpeg(
    150.0, 30.0, 30.0, 30.0,
    std::fs::read("seal.jpg")?,
));

// Page 2
let mut page2 = OfdPage::new(210.0, 297.0);
page2.add_text(TextObject::new(20.0, 30.0, "Terms & Conditions").size(14.0));
page2.add_path(PathObject::rect(10.0, 10.0, 190.0, 277.0).stroke_color(0x333333));

EasyOfd::write_pages("document.ofd")
    .metadata_title("My Document")
    .do_write(vec![page1, page2])?;
```

---

## API Reference | API &#21442;&#32771;

### EasyOfd | Entry Points

| Method &#26041;&#27861; | Signature &#31614;&#21517; | Returns &#36820;&#22238; |
|:---|---|:---|
| `write::<T>` | `(path: impl Into<String>)` | `OfdWriterBuilder<T>` |
| `write_pages` | `(path: impl Into<String>)` | `PageWriterBuilder` |
| `write_pages_to` | `(path, pages: Vec<OfdPage>)` | `OfdResult<()>` |
| `write_pages_to_bytes` | `(pages: Vec<OfdPage>)` | `OfdResult<Vec<u8>>` |

### OfdWriterBuilder\<T: OfdModel\>

| Method | Signature | Description |
|:---|---|:---|
| `metadata_title` | `(title: impl Into<String>) -> Self` | Set document title |
| `metadata_author` | `(author: impl Into<String>) -> Self` | Set document author |
| `metadata_creator` | `(creator: impl Into<String>) -> Self` | Set creator application name |
| `do_write` | `(&self, data: &[T]) -> OfdResult<()>` | Write to file path |
| `do_write_to_bytes` | `(&self, data: &[T]) -> OfdResult<Vec<u8>>` | Write to in-memory bytes |

### PageWriterBuilder

| Method | Signature | Description |
|:---|---|:---|
| `metadata_title` | `(title: impl Into<String>) -> Self` | Set document title |
| `metadata_author` | `(author: impl Into<String>) -> Self` | Set document author |
| `metadata_creator` | `(creator: impl Into<String>) -> Self` | Set creator application name |
| `do_write` | `(&self, pages: Vec<OfdPage>) -> OfdResult<()>` | Write to file path |
| `do_write_to_bytes` | `(&self, pages: Vec<OfdPage>) -> OfdResult<Vec<u8>>` | Write to in-memory bytes |

### Core Types | &#26680;&#24515;&#31867;&#22411;

<details>
<summary>Click to expand &#28857;&#20987;&#23637;&#24320;</summary>

#### OfdModel &#27966;&#29983;&#23439; Trait

```rust
pub trait OfdModel: Sized {
    fn schema() -> &'static [OfdField];     // Field metadata
    fn page_size() -> (f64, f64);            // Page dimensions (mm)
    fn to_page(&self) -> OfdResult<OfdPage>; // Convert to page
    fn to_pages(items: &[Self]) -> OfdResult<Vec<OfdPage>>; // Batch convert
}
```

#### `#[ofd(...)]` Attribute Reference

| Attribute | Type | Default | Description |
|:---|:---|:---|:---|
| `x` | f64 | required | X position in mm |
| `y` | f64 | required | Y position in mm |
| `font` | str | `"SimSun"` | Font family name |
| `size` | f64 | `12.0` | Font size in pt |
| `bold` | flag | false | Bold text (weight 700) |
| `weight` | u32 | `400` | Font weight |
| `italic` | flag | false | Italic text |
| `color` | u32 | `0` | RGB color as hex (e.g. `0xFF0000`) |
| `kind` | str | `"text"` | Render kind: `"text"` or `"image"` |
| `img_width` | f64 | `30.0` | Image width in mm (kind="image") |
| `img_height` | f64 | `30.0` | Image height in mm (kind="image") |
| `ignore` | flag | false | Skip this field entirely |

#### Struct-level Attributes

| Attribute | Type | Default | Description |
|:---|:---|:---|:---|
| `page_width` | f64 | `210.0` | Page width in mm |
| `page_height` | f64 | `297.0` | Page height in mm |

#### Content Objects

| Type | Description |
|:---|:---|
| `TextObject` | Positioned text with font, size, weight, color, italic |
| `ImageObject` | Positioned image (JPEG/PNG/BMP/TIFF) |
| `PathObject` | Vector path: lines, rectangles with stroke/fill |

#### Page Sizes (in mm)

| Constant | Dimensions |
|:---|:---|
| `page_size::A4` | `(210.0, 297.0)` |
| `page_size::A4_LANDSCAPE` | `(297.0, 210.0)` |
| `page_size::A3` | `(297.0, 420.0)` |
| `page_size::LETTER` | `(215.9, 279.4)` |

#### Errors

| Variant | Description |
|:---|:---|
| `OfdError::Io(e)` | Wraps `std::io::Error` |
| `OfdError::Xml(msg)` | XML serialization error |
| `OfdError::Zip(msg)` | ZIP archive error |
| `OfdError::InvalidDocument(msg)` | Invalid OFD document structure |
| `OfdError::InvalidPage(msg)` | Invalid page content |
| `OfdError::Conversion(msg)` | Type conversion error |
| `OfdError::Model(msg)` | Model mapping error |

```rust
pub type OfdResult<T> = std::result::Result<T, OfdError>;
```

</details>

---

## Design Principles | &#35774;&#35745;&#21407;&#21017;

| Principle &#21407;&#21017; | Practice &#23454;&#36341; |
|:---|:---|
| **Pure Rust** | `#![forbid(unsafe_code)]` in every crate |
| **Type-safe builders** | `mut self -> Self`, `#[must_use]` on all builders |
| **Compile-time reflection** | `#[derive(OfdModel)]` generates mapping code &mdash; no runtime reflection |
| **Trait extensibility** | `OfdModel` trait for custom implementations |
| **Error transparency** | Single `OfdError` enum with `thiserror`, single `OfdResult<T>` alias |
| **Zero-cost abstractions** | Builder chains compile to direct calls, derive macros expand at compile time |
| **Separation of concerns** | Core types != writer implementation != facade. Each crate has one job. |
| **Inspired by Alibaba EasyExcel** | Same builder &middot; derive &middot; facade patterns as `easyexcel-rs` |

---

## Roadmap | &#36335;&#32447;&#22270;

| Phase | Focus | Key Deliverables |
|:---:|:---|:---|
| **v0.1** &#9989; | Foundation | Workspace, 4 crates, core types, ZIP/XML writer, derive macro, builder API, 98%+ coverage |
| **v0.2** &#128679; | Reader & template | OFD reader, template placeholder filling, text rendering with font embedding |
| **v0.3** | Security | Electronic seals & signatures (GB/T 38540), encryption |
| **v0.4** | Converters | PDF &#8596; OFD bidirectional conversion, image format conversion |
| **v1.0** | Stable | Stable API, full test coverage, performance benchmarks, documentation |

---

## License | &#35768;&#21487;&#35777;

Apache-2.0

---

## Related Projects | &#30456;&#20851;&#39033;&#30446;

- [easyexcel-rs](https://github.com/hiwepy/easyexcel-rs) &mdash; Rust port of Alibaba EasyExcel
- [easyexcel](https://github.com/alibaba/easyexcel) &mdash; Original Java library by Alibaba
- [ofd-rs](https://crates.io/crates/ofd-rs) &mdash; Lower-level OFD writer crate
- [ofd-core](https://crates.io/crates/ofd-core) &mdash; OFD XML parsing & data model crate

---

<p align="center">
  <sub>Built with Rust &#129408; &middot; Follows <a href="https://github.com/hiwepy/easyexcel-rs">easyexcel-rs</a> conventions</sub>
</p>
