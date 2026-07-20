# easyofd-rs Architecture Design Document &middot; &#26550;&#26500;&#35774;&#35745;&#25991;&#26723;

> **Version**: 0.1.0 | **Date**: 2026-07-21 | **Status**: Phase 1 Complete  
> **Author**: easyofd-rs team | **License**: Apache-2.0

---

## Table of Contents &#30446;&#24405;

1. [Project Vision &#39033;&#30446;&#24895;&#26223;](#1-project-vision-&#39033;&#30446;&#24895;&#26223;)
2. [Design Goals &#35774;&#35745;&#30446;&#26631;](#2-design-goals-&#35774;&#35745;&#30446;&#26631;)
3. [Crate Architecture &#21253;&#26550;&#26500;](#3-crate-architecture-&#21253;&#26550;&#26500;)
4. [Dependency Graph &#20381;&#36182;&#22270;](#4-dependency-graph-&#20381;&#36182;&#22270;)
5. [Data Flow &#25968;&#25454;&#27969;](#5-data-flow-&#25968;&#25454;&#27969;)
6. [Core Abstractions &#26680;&#24515;&#25277;&#35937;](#6-core-abstractions-&#26680;&#24515;&#25277;&#35937;)
7. [Builder Pattern Design &#26500;&#24314;&#22120;&#27169;&#24335;&#35774;&#35745;](#7-builder-pattern-design-&#26500;&#24314;&#22120;&#27169;&#24335;&#35774;&#35745;)
8. [OFD Format Compliance OFD &#26684;&#24335;&#21512;&#35268;](#8-ofd-format-compliance-ofd-&#26684;&#24335;&#21512;&#35268;)
9. [Error Handling &#38169;&#35823;&#22788;&#29702;](#9-error-handling-&#38169;&#35823;&#22788;&#29702;)
10. [Derive Macro &#27966;&#29983;&#23439;](#10-derive-macro-&#27966;&#29983;&#23439;)
11. [Conventions from easyexcel-rs &#32487;&#25215;&#32422;&#23450;](#11-conventions-from-easyexcel-rs-&#32487;&#25215;&#32422;&#23450;)
12. [OFD vs Excel Paradigm Differences OFD&#19982;Excel&#33539;&#24335;&#24046;&#24322;](#12-ofd-vs-excel-paradigm-differences-ofd&#19982;excel&#33539;&#24335;&#24046;&#24322;)
13. [Future Architecture &#26410;&#26469;&#26550;&#26500;](#13-future-architecture-&#26410;&#26469;&#26550;&#26500;)

---

## 1. Project Vision &#39033;&#30446;&#24895;&#26223;

`easyofd-rs` aims to provide **the same developer experience for OFD operations** that
[easyexcel-rs](https://github.com/hiwepy/easyexcel-rs) provides for Excel:

> **Type-safe Builders + Compile-time Reflection + GB/T 33190-2016 Compliant = Ergonomic OFD manipulation in idiomatic Rust.**

The library covers three primary use cases:

| Use Case &#29992;&#20363; | Description &#25551;&#36848; | EasyExcel Analogy &#31867;&#27604; |
|:---|:---|:---|
| **Create** &#21019;&#24314; | Generate new OFD files with text, images, paths, metadata | `EasyExcel.write()` |
| **Read** &#35835;&#21462; | Parse and extract content from existing OFD files (planned) | `EasyExcel.read()` |
| **Fill** &#22635;&#20805; | Populate placeholders in OFD templates (planned) | `EasyExcel.fill()` |

---

## 2. Design Goals &#35774;&#35745;&#30446;&#26631;

| # | Goal &#30446;&#26631; | Rationale &#29702;&#30001; |
|:---|:---|:---|
| G1 | **Pure Rust, zero unsafe** | `#![forbid(unsafe_code)]` in every crate. Aligns with easyexcel-rs's safety policy. |
| G2 | **Fluent Builder API** | `mut self -> Self` with `#[must_use]`. Method chains read like natural language. |
| G3 | **Compile-time reflection** | `#[derive(OfdModel)]` replaces Java's runtime annotation scanning. |
| G4 | **Single facade entry point** | `EasyOfd::write()`, `EasyOfd::write_pages()` &mdash; discoverable static factory. |
| G5 | **GB/T 33190-2016 compliant** | Output is valid OFD ZIP archives with correct XML namespaces and structure. |
| G6 | **Single error type** | `OfdError` enum with `thiserror`, `type OfdResult<T> = ...` &mdash; no scattered error types. |
| G7 | **Separation of concerns** | Core types != writer implementation != facade. Each crate has one job. |
| G8 | **Follow easyexcel-rs conventions** | Naming, structure, quality gates &mdash; consistency across the ecosystem. |

---

## 3. Crate Architecture &#21253;&#26550;&#26500;

```
easyofd-rs/
&#9500;&#9472;&#9472; Cargo.toml                     Virtual workspace (edition 2021, resolver="2")
&#9474;
&#9500;&#9472;&#9472; crates/
&#9474;   &#9500;&#9472;&#9472; easyofd/                   &#55356;&#57270; FACADE &mdash; user-facing entry point
&#9474;   &#9474;   &#9500;&#9472;&#9472; src/lib.rs             EasyOfd struct, all Builder types, re-exports
&#9474;   &#9474;   &#9492;&#9472;&#9472; tests/
&#9474;   &#9474;       &#9500;&#9472;&#9472; derive_model.rs     Integration tests for derive macro + write flows
&#9474;   &#9474;       &#9492;&#9472;&#9472; compile_fail.rs     Trybuild compile-fail tests for derive error paths
&#9474;   &#9474;
&#9474;   &#9500;&#9472;&#9472; easyofd-core/              &#55356;&#57311; CORE &mdash; zero engine dependency
&#9474;   &#9474;   &#9492;&#9472;&#9472; src/
&#9474;   &#9474;       &#9500;&#9472;&#9472; lib.rs             Flat re-exports
&#9474;   &#9474;       &#9500;&#9472;&#9472; error.rs           OfdError enum, OfdResult<T> alias
&#9474;   &#9474;       &#9500;&#9472;&#9472; model.rs           OfdPage, TextObject, ImageObject, PathObject, OfdMetadata
&#9474;   &#9474;       &#9492;&#9472;&#9472; ofd_model.rs       OfdModel trait, OfdField, OfdFieldKind
&#9474;   &#9474;
&#9474;   &#9500;&#9472;&#9472; easyofd-derive/            &#9881;&#65039; PROC-MACRO &mdash; compile-time code gen
&#9474;   &#9474;   &#9492;&#9472;&#9472; src/lib.rs             #[proc_macro_derive(OfdModel, attributes(ofd))]
&#9474;   &#9474;
&#9474;   &#9492;&#9472;&#9472; easyofd-writer/            &#9997;&#65039; WRITER &mdash; ZIP/XML generation
&#9474;       &#9492;&#9472;&#9472; src/lib.rs             OfdWriter: add_page, build, build_to_file
```

### Crate responsibility matrix &#21253;&#32844;&#36131;&#30697;&#38453;

| Crate | External deps | Depends on | Role |
|:---|:---|:---|:---|
| **easyofd** | &mdash; (zero prod deps) | all sub-crates | Builder entry points + re-exports |
| **easyofd-core** | thiserror, chrono | &mdash; | Shared types, traits, errors, data model |
| **easyofd-derive** | syn, quote, proc-macro2 | &mdash; (dev: easyofd-core) | Derive macro only |
| **easyofd-writer** | zip, uuid, quick-xml, chrono | easyofd-core | OFD ZIP/XML creation |

---

## 4. Dependency Graph &#20381;&#36182;&#22270;

```
                        &#9476;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;
                        &#9474; easyofd  &#9474;  &#8592; user depends on this
                        &#9474; (facade) &#9474;
                        &#9476;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;
           &#9476;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;
           &#9474;                &#9474;
           &#9660;                &#9660;
      &#9476;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;    &#9476;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;
      &#9474; core    &#9474;    &#9474; writer  &#9474;
      &#9474; +derive &#9474;    &#9474;        &#9474;
      &#9476;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;    &#9476;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;
       &#9474;                     &#9474;
       &#9474;                     &#9474;
  thiserror, chrono      zip, uuid, quick-xml
```

Key observations:

1. **easyofd-core has zero engine dependencies** &mdash; it defines *what* an OFD element is, not *how* to render it.
2. **easyofd-derive produces code that references easyofd-core** &mdash; the generated `impl OfdModel` blocks use types from `easyofd_core`.
3. **easyofd-writer is format-specific** &mdash; it handles ZIP container creation and XML serialization per GB/T 33190-2016.
4. **Facade depends on everyone** &mdash; it wires sub-crates together and provides the ergonomic `EasyOfd::write()` / `EasyOfd::write_pages()` entry points.

---

## 5. Data Flow &#25968;&#25454;&#27969;

### 5.1 Write Flow via Derive Macro &#20889;&#20837;&#27969;&#31243;&#65288;&#27966;&#29983;&#23439;&#65289;

```
User code                    easyofd facade              easyofd-core            easyofd-writer
&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;                    &#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;              &#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;            &#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;
EasyOfd::write::<T>("out.ofd")
  .metadata_title("Doc")
  .do_write(&[data])?
        &#9474;
        &#9660;
  OfdWriterBuilder<T>::do_write()
        &#9474;
        &#9474;&#9472;&#9472; T::to_pages(&[data])
        &#9474;       &#9474;
        &#9474;       &#9660;
        &#9474;   For each item:
        &#9474;     Vec<OfdPage>              &#8592; generated by #[derive(OfdModel)]
        &#9474;
        &#9474;&#9472;&#9472; OfdWriter::with_options(options)
        &#9474;       &#9474;
        &#9474;       &#9660;
        &#9474;   OfdWriter::add_pages(pages)
        &#9474;       &#9474;
        &#9474;       &#9660;
        &#9474;   OfdWriter::build()
        &#9474;       &#9474;
        &#9474;       &#9660;
        &#9474;   Vec<u8>                     &#8592; ZIP archive with XML entries
        &#9474;
        &#9660;
  write to out.ofd
```

### 5.2 Write Flow via Manual Page Construction &#20889;&#20837;&#27969;&#31243;&#65288;&#25163;&#21160;&#26500;&#24314;&#65289;

```
User code                    easyofd facade              easyofd-writer
&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;                    &#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;              &#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;
EasyOfd::write_pages("out.ofd")
  .metadata_title("Doc")
  .do_write(vec![page])?
        &#9474;
        &#9660;
  PageWriterBuilder::do_write()
        &#9474;
        &#9474;&#9472;&#9472; OfdWriter::with_options(options)
        &#9474;       &#9474;
        &#9474;       &#9660;
        &#9474;   OfdWriter::add_pages(pages)
        &#9474;       &#9474;
        &#9474;       &#9660;
        &#9474;   OfdWriter::build()
        &#9474;       &#9474;
        &#9474;       &#9660;
        &#9474;   Build ZIP structure:
        &#9474;     &#9500;&#9472;&#9472; OFD.xml            &#8592; entry point + metadata
        &#9474;     &#9492;&#9472;&#9472; Doc_0/
        &#9474;         &#9500;&#9472;&#9472; Document.xml     &#8592; document structure
        &#9474;         &#9500;&#9472;&#9472; DocumentRes.xml  &#8592; resource declarations
        &#9474;         &#9500;&#9472;&#9472; Pages/
        &#9474;         &#9474;   &#9500;&#9472;&#9472; Page_0.xml      &#8592; TextObject, ImageObject, PathObject
        &#9474;         &#9474;   &#9492;&#9472;&#9472; Page_N.xml
        &#9474;         &#9492;&#9472;&#9472; Res/              &#8592; embedded images
        &#9474;             &#9492;&#9472;&#9472; Image_0.jpeg
        &#9474;
        &#9660;
  write to out.ofd
```

---

## 6. Core Abstractions &#26680;&#24515;&#25277;&#35937;

### 6.1 Type Hierarchy &#31867;&#22411;&#23618;&#32423;

```
OfdError (enum)          &mdash; Central error type, 7 variants
  &#9500;&#9472;&#9472; Io(io::Error)      &mdash; Wraps std I/O errors
  &#9500;&#9472;&#9472; Xml(String)       &mdash; XML serialization errors
  &#9500;&#9472;&#9472; Zip(String)       &mdash; ZIP archive errors
  &#9500;&#9472;&#9472; InvalidDocument(String) &mdash; Invalid OFD document structure
  &#9500;&#9472;&#9472; InvalidPage(String)    &mdash; Invalid page content
  &#9500;&#9472;&#9472; Conversion(String)  &mdash; Type conversion errors
  &#9492;&#9472;&#9472; Model(String)      &mdash; Model mapping errors

OfdMetadata (struct)     &mdash; version, title, author, creator, creation_date

OfdPage (struct)         &mdash; width, height (mm), content: Vec<ContentObject>

ContentObject (enum)     &mdash; The page content building blocks
  &#9500;&#9472;&#9472; Text(TextObject)   &mdash; Positioned text block
  &#9500;&#9472;&#9472; Image(ImageObject) &mdash; Positioned image
  &#9492;&#9472;&#9472; Path(PathObject)   &mdash; Vector path

TextObject (struct)      &mdash; x, y, font, size, weight, italic, color, text, width?, height?
ImageObject (struct)     &mdash; x, y, width, height, data: Vec<u8>, format: ImageFormat
PathObject (struct)      &mdash; x, y, stroke_color, stroke_width, fill_color?, path_data: String

ImageFormat (enum)       &mdash; Jpeg | Png | Bmp | Tiff
OfdFieldKind (enum)      &mdash; Text | Image
```

### 6.2 OfdModel Trait &#27966;&#29983;&#23439; Trait

The central mapping trait. Generated by `#[derive(OfdModel)]` or implemented manually:

```rust
pub trait OfdModel: Sized {
    /// Returns the field schema for this model.
    fn schema() -> &'static [OfdField];

    /// Returns the page size (width, height) in mm. Default: A4 (210 x 297).
    fn page_size() -> (f64, f64) { (210.0, 297.0) }

    /// Convert this model instance into an OFD page.
    fn to_page(&self) -> OfdResult<OfdPage>;

    /// Convert a slice of model instances into a vec of OFD pages.
    fn to_pages(items: &[Self]) -> OfdResult<Vec<OfdPage>> {
        items.iter().map(Self::to_page).collect()
    }
}
```

### 6.3 ContentObject &mdash; The Bridge &#26725;&#25509;&#20803;&#32032;

`ContentObject` is the bridge between model definition and writer rendering:

```rust
pub enum ContentObject {
    Text(TextObject),    // Text block at a position
    Image(ImageObject),  // Image at a position
    Path(PathObject),    // Vector path at a position
}
```

The derive macro generates `to_page()` which produces `Vec<ContentObject>`; the writer consumes them to produce XML.

---

## 7. Builder Pattern Design &#26500;&#24314;&#22120;&#27169;&#24335;&#35774;&#35745;

### 7.1 Pattern Rules

| Rule | Code Pattern | Why |
|:---|:---|:---|
| Owned self | `pub fn method(mut self, ...) -> Self` | Enables chaining, prevents accidental reuse |
| Must-use | `#[must_use]` on all builder structs and methods | Compiler warns if chain result is discarded |
| Terminal execute | `do_write()`, `do_write_to_bytes()` | Builder -> action, builder is consumed |
| Static factory entry | `EasyOfd::write::<T>(path)` | Discoverable, canonical entry point |
| PhantomData typing | `_phantom: PhantomData<T>` on OfdWriterBuilder | Keeps the type parameter without owning a T |

### 7.2 Builder State Machine

```
EasyOfd::write::<T>("out.ofd")
        &#9474;
        &#9660;
  OfdWriterBuilder<T>
        &#9474;
        &#9500;&#9472;&#9472; metadata_title(t) -> OfdWriterBuilder<T> (stay)
        &#9500;&#9472;&#9472; metadata_author(a) -> OfdWriterBuilder<T> (stay)
        &#9500;&#9472;&#9472; metadata_creator(c) -> OfdWriterBuilder<T> (stay)
        &#9474;
        &#9500;&#9472;&#9472; do_write(&[data]) -> OfdResult<()>         &#8592; writes to file
        &#9492;&#9472;&#9472; do_write_to_bytes(&[data]) -> OfdResult<Vec<u8>>  &#8592; returns bytes


EasyOfd::write_pages("out.ofd")
        &#9474;
        &#9660;
  PageWriterBuilder
        &#9474;
        &#9500;&#9472;&#9472; metadata_title(t) -> PageWriterBuilder (stay)
        &#9500;&#9472;&#9472; metadata_author(a) -> PageWriterBuilder (stay)
        &#9500;&#9472;&#9472; metadata_creator(c) -> PageWriterBuilder (stay)
        &#9474;
        &#9500;&#9472;&#9472; do_write(vec![page]) -> OfdResult<()>
        &#9492;&#9472;&#9472; do_write_to_bytes(vec![page]) -> OfdResult<Vec<u8>>
```

### 7.3 Builder Design Rationale

The builder pattern was chosen over:

| Alternative | Rejected because |
|:---|:---|
| Config struct + function | No type-state, easy to forget required fields |
| `&mut self` builders | Cannot enforce single-use, harder to chain |
| Macro-based DSL | Harder to discover API, worse IDE support |

The `mut self -> Self` pattern (owned builder) is the standard Rust convention, used by `std::process::Command`, `reqwest::ClientBuilder`, and easyexcel-rs.

---

## 8. OFD Format Compliance OFD &#26684;&#24335;&#21512;&#35268;

### 8.1 ZIP Structure

The writer produces a GB/T 33190-2016 compliant ZIP archive:

```
output.ofd (ZIP)
&#9500;&#9472;&#9472; OFD.xml                    &#8592; Entry point + DocBody with metadata
&#9492;&#9472;&#9472; Doc_0/
    &#9500;&#9472;&#9472; Document.xml           &#8592; CommonData (PageArea, resources) + Pages list
    &#9500;&#9472;&#9472; DocumentRes.xml        &#8592; MultiMedia resource declarations
    &#9500;&#9472;&#9472; Pages/
    &#9474;   &#9500;&#9472;&#9472; Page_0.xml            &#8592; Area + Content layer (TextObject, ImageObject, PathObject)
    &#9474;   &#9492;&#9472;&#9472; Page_N.xml
    &#9492;&#9472;&#9472; Res/                     &#8592; Embedded image files
        &#9492;&#9472;&#9472; Image_0.jpeg
```

### 8.2 XML Namespace

All XML elements use the standard OFD namespace:

```xml
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.0">
```

### 8.3 Content Element Mapping

| Rust Type | OFD Element | Key Attributes |
|:---|:---|:---|
| `TextObject` | `<ofd:TextObject>` | `ID`, `Boundary`, `Font`, `Size` |
| `ImageObject` | `<ofd:ImageObject>` | `ID`, `Boundary`, `ResourceID` |
| `PathObject` | `<ofd:PathObject>` | `ID`, `Boundary`, `StrokeColor`, `LineWidth` |

### 8.4 Coordinate System

OFD uses **millimeters** as the coordinate unit. All position and dimension values in the API are specified in mm. The origin (0, 0) is the top-left corner of the page.

---

## 9. Error Handling &#38169;&#35823;&#22788;&#29702;

### 9.1 Error Taxonomy &#38169;&#35823;&#20998;&#31867;

```
OfdError
&#9500;&#9472;&#9472; Io              &#8592; wraps std::io::Error (file not found, permission denied, etc.)
&#9500;&#9472;&#9472; Xml             &#8592; XML serialization/deserialization errors
&#9500;&#9472;&#9472; Zip             &#8592; ZIP archive errors (corrupt archive, compression failure)
&#9500;&#9472;&#9472; InvalidDocument &#8592; missing required files, invalid OFD structure
&#9500;&#9472;&#9472; InvalidPage     &#8592; invalid page content or dimensions
&#9500;&#9472;&#9472; Conversion      &#8592; type conversion errors
&#9492;&#9472;&#9472; Model           &#8592; field mapping errors from OfdModel trait
```

### 9.2 Error Flow

```
IO error / ZIP error / XML error
        &#9474;
        &#9660;
  Mapped to OfdError variant in the writer
        &#9474;
        &#9660;
  Propagated via ? through builder chain
        &#9474;
        &#9660;
  User receives easyofd::OfdResult<T>
```

### 9.3 Design Decisions

| Decision | Rationale |
|:---|:---|
| Single `OfdError` enum | Users only need one error type in their code |
| `thiserror` derive | Automatic `Display` + `Error` + `From` impls |
| `type OfdResult<T> = ...` | Less typing, consistent across the codebase |
| `From<std::io::Error>` impl | Direct `?` propagation from `std::fs` operations |
| No `anyhow` in library code | Library should expose structured errors; `anyhow` is for applications |

---

## 10. Derive Macro &#27966;&#29983;&#23439;

### 10.1 `#[derive(OfdModel)]` Architecture

```
User writes:
  #[derive(OfdModel)]
  #[ofd(page_width = 210.0, page_height = 297.0)]
  struct Invoice {
      #[ofd(x = 20.0, y = 30.0, size = 18.0, bold)]
      title: String,
      #[ofd(x = 20.0, y = 50.0)]
      amount: String,
      #[ofd(ignore)]
      internal_id: u64,
  }

        &#9474;  proc_macro expansion
        &#9660;

Generated code:
  impl easyofd_core::OfdModel for Invoice {
      fn schema() -> &'static [easyofd_core::OfdField] {
          static SCHEMA: OnceLock<Vec<OfdField>> = OnceLock::new();
          SCHEMA.get_or_init(|| vec![
              OfdField {
                  name: "title",
                  position: (20.0, 30.0),
                  font: "SimSun",
                  size: 18.0,
                  weight: 700,
                  italic: false,
                  color: 0,
                  kind: OfdFieldKind::Text,
              },
              OfdField {
                  name: "amount",
                  position: (20.0, 50.0),
                  font: "SimSun",
                  size: 12.0,
                  weight: 400,
                  italic: false,
                  color: 0,
                  kind: OfdFieldKind::Text,
              },
              // internal_id is #[ofd(ignore)] &mdash; not included in schema
          ]).as_slice()
      }

      fn page_size() -> (f64, f64) {
          (210.0, 297.0)
      }

      fn to_page(&self) -> easyofd_core::OfdResult<easyofd_core::OfdPage> {
          let (width, height) = Self::page_size();
          let mut content = Vec::with_capacity(2);
          content.push(ContentObject::Text(
              TextObject::new(20.0, 30.0, self.title.to_string())
                  .font("SimSun")
                  .size(18.0)
                  .color(0)
          ));
          content.push(ContentObject::Text(
              TextObject::new(20.0, 50.0, self.amount.to_string())
                  .font("SimSun")
                  .size(12.0)
                  .color(0)
          ));
          Ok(OfdPage { width, height, content })
      }
  }
```

### 10.2 Attribute Parsing Pipeline

```
proc_macro::TokenStream
        &#9474;
        &#9660;
  syn::parse2 -> DeriveInput
        &#9474;
        &#9500;&#9472;&#9472; parse_page_attrs(&input.attrs)
        &#9474;     &#9492;&#9472;&#9472; #[ofd(page_width = ..., page_height = ...)]
        &#9474;         -> (width_f64, height_f64)
        &#9474;
        &#9492;&#9472;&#9472; process_fields(&fields.named)
              &#9474;
              For each named field:
              &#9500;&#9472;&#9472; #[ofd(ignore)] -> skip
              &#9500;&#9472;&#9472; #[ofd(kind = "text"), ...] -> generate TextObject push
              &#9500;&#9472;&#9472; #[ofd(kind = "image"), ...] -> generate ImageObject push
              &#9492;&#9472;&#9472; unknown kind -> compile_error!
```

### 10.3 Compile-time Error Detection

The following errors are caught at compile time:

| Error | Detection |
|:---|:---|
| Derive on non-struct type (enum, union) | `compile_error!("OfdModel can only be derived for structs")` |
| Derive on tuple struct (no named fields) | `compile_error!("OfdModel can only be derived for structs with named fields")` |
| Unknown `kind` attribute value | `compile_error!("unknown ofd kind: ...")` |
| Non-numeric value for numeric attributes | `compile_error!("expected a numeric literal")` |

These are validated via `trybuild` compile-fail tests.

### 10.4 Comparison: Java Annotations vs Rust Derive

| Aspect | Java EasyExcel | Rust easyofd-rs |
|:---|:---|:---|
| Annotation | `@ExcelProperty(value = "Name")` | `#[ofd(x = 20.0, y = 30.0)]` |
| Processing | Runtime reflection | Compile-time code gen |
| Performance | Reflection overhead | Zero-cost, direct calls |
| Error detection | Runtime | Compile-time |
| IDE support | Good (annotation processors) | Good (proc-macro expansion) |

---

## 11. Conventions from easyexcel-rs &#32487;&#25215;&#32422;&#23450;

| Convention &#32422;&#23450; | easyexcel-rs | easyofd-rs | Notes |
|:---|:---|:---|:---|
| **Workspace** | Virtual manifest + shared `[workspace.dependencies]` | &#9989; Same | `resolver = "2"`, edition 2021 |
| **Crate naming** | `easyexcel`, `easyexcel-core`, `easyexcel-derive`, ... | `easyofd`, `easyofd-core`, `easyofd-derive`, ... | &#9989; Same pattern |
| **MSRV** | 1.88 | 1.70 | easyofd-rs is more conservative |
| **Edition** | 2024 | 2021 | easyofd-rs targets broader compatibility |
| **License** | Apache-2.0 | &#9989; Apache-2.0 | |
| **unsafe** | `#![forbid(unsafe_code)]` | &#9989; Same | Workspace-level lint |
| **Lints** | `clippy::pedantic`, `missing_docs` | &#9989; Same | |
| **Error type** | `thiserror` derive, single enum | &#9989; `OfdError` | Seven variants |
| **Result alias** | `pub type Result<T> = ...` | &#9989; `OfdResult<T>` | |
| **Builder** | `mut self -> Self`, `#[must_use]` | &#9989; Same | Owned builder pattern |
| **Facade** | Thin crate with zero prod deps | &#9989; `crates/easyofd` | Only path deps on sub-crates |
| **Derive macro** | `syn`/`quote`/`proc-macro2` | &#9989; Same | `#[derive(OfdModel)]` |

---

## 12. OFD vs Excel Paradigm Differences OFD&#19982;Excel&#33539;&#24335;&#24046;&#24322;

Understanding these differences is critical for API design:

| Dimension | Excel (easyexcel-rs) | OFD (easyofd-rs) |
|:---|:---|:---|
| **Layout model** | Grid-based (rows x columns) | Coordinate-based (x, y in mm) |
| **Data unit** | Cell (A1, B2, ...) | Content object at position (x, y) |
| **Header** | Row 1 with column names | No inherent concept &mdash; text objects at explicit positions |
| **Style** | Per-cell or per-column | Per-text-object (font, size, weight, color) |
| **Template** | `{key}` placeholders in cells | Future: text replacement in XML |
| **Multiple sheets** | Workbook -> Sheet1, Sheet2, ... | Single document -> Page1, Page2, ... |
| **Read direction** | Top-to-bottom, left-to-right | Any order (coordinate-based) |
| **Memory model** | SXSSF (streaming write to disk) | In-memory Vec<ContentObject>, ZIP write at end |
| **File format** | ZIP (Office Open XML) | ZIP (GB/T 33190-2016 XML) |

### Design implications:

1. **No "row" or "cell" abstraction in OFD** &mdash; `TextObject` replaces `Cell`/`Row`. The derive macro maps struct fields to positioned text blocks.
2. **Position is explicit** &mdash; Every element has `(x, y)` coordinates in mm. Future work may add an auto-layout engine.
3. **Pages are the primary container** &mdash; Each `OfdPage` is independent, like a worksheet in Excel but without the grid.
4. **Images are embedded** &mdash; Image data is stored as ZIP entries in `Doc_0/Res/` and referenced by resource ID.

---

## 13. Future Architecture &#26410;&#26469;&#26550;&#26500;

### 13.1 Planned Crate Expansion

```
easyofd-rs/
&#9500;&#9472;&#9472; easyofd (facade)             &#8592; current
&#9500;&#9472;&#9472; easyofd-core (types)          &#8592; current
&#9500;&#9472;&#9472; easyofd-derive (macro)        &#8592; current
&#9500;&#9472;&#9472; easyofd-writer (write)        &#8592; current
&#9474;
&#9500;&#9472;&#9472; easyofd-reader (read)         &#8592; v0.2: parse OFD ZIP, extract content
&#9500;&#9472;&#9472; easyofd-template (fill)       &#8592; v0.2: placeholder-based template engine
&#9500;&#9472;&#9472; easyofd-signature (sign)      &#8592; v0.3: GB/T 38540 electronic seals
&#9492;&#9472;&#9472; easyofd-convert (convert)    &#8592; v0.4: PDF &#8596; OFD bidirectional conversion
```

### 13.2 Reader Architecture (v0.2)

```
User calls EasyOfd::read("input.ofd")
        &#9474;
        &#9660;
  OfdReader::open("input.ofd")
        &#9474;
        &#9660;
  Unzip archive -> parse OFD.xml -> follow DocRoot -> parse Document.xml
        &#9474;
        &#9660;
  For each Page reference:
        &#9474;
        &#9660;
  Parse Page_N.xml -> extract TextObject, ImageObject -> callback to ReadListener
```

### 13.3 Template Engine (v0.2)

```
User calls EasyOfd::fill_template("template.ofd", &data)
        &#9474;
        &#9660;
  Unzip template -> find {placeholders} in XML content
        &#9474;
        &#9660;
  Replace {key} with data values from OfdModel
        &#9474;
        &#9660;
  Re-pack as new OFD ZIP
```

### 13.4 Auto-Layout Engine (v0.3+)

```
User defines:
  FlowLayout::vertical()
      .margin(20.0)
      .spacing(10.0)

Elements auto-position:
  &#9476;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;
  &#9474; Title        (y = 250)   &#9474;  &#8592; auto-placed
  &#9474;              (spacing)   &#9474;
  &#9474; Paragraph 1  (y = 220)   &#9474;
  &#9474; Paragraph 2  (y = 190)   &#9474;
  &#9474; Image        (y = 150)   &#9474;
  &#9474;              (spacing)   &#9474;
  &#9474; Footer       (y = 20)    &#9474;
  &#9476;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;
```

---

## Appendix A: Quality Gates

| Gate | Command | Status |
|:---|:---|:---:|
| Format | `cargo fmt --all -- --check` | &#9989; |
| Lint | `cargo clippy --all-targets` | &#9989; 0 warnings |
| Build | `cargo check --workspace` | &#9989; |
| Test | `cargo test --workspace` | &#9989; 137 tests |
| Coverage | `cargo llvm-cov --all-targets` | &#9989; 98.14% |

## Appendix B: File Inventory

| File | Lines | Purpose |
|:---|:---:|:---|
| `Cargo.toml` | 40 | Workspace manifest |
| `crates/easyofd-core/src/error.rs` | 110 | OfdError + OfdResult |
| `crates/easyofd-core/src/model.rs` | 540 | OfdPage, TextObject, ImageObject, PathObject, page_size |
| `crates/easyofd-core/src/ofd_model.rs` | 170 | OfdModel trait, OfdField, OfdFieldKind |
| `crates/easyofd-core/src/lib.rs` | 14 | Re-exports |
| `crates/easyofd-derive/src/lib.rs` | 322 | #[proc_macro_derive(OfdModel)] |
| `crates/easyofd-writer/src/lib.rs` | 890 | OfdWriter: ZIP + XML generation |
| `crates/easyofd/src/lib.rs` | 335 | EasyOfd + all Builders |
| **Total** | **~2,420** | |

---

> *"Design is not just what it looks like and feels like. Design is how it works."* &mdash; Steve Jobs
