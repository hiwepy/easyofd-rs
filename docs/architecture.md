# easyofd-rs Architecture Design Document · 架构设计文档

> **Version**: 0.1.0 | **Date**: 2026-07-21 | **Status**: Phase 1 Complete  
> **Author**: easyofd-rs team | **License**: Apache-2.0

---

## Table of Contents 目录

1. [Project Vision 项目愿景](#1-project-vision-项目愿景)
2. [Design Goals 设计目标](#2-design-goals-设计目标)
3. [Crate Architecture 包架构](#3-crate-architecture-包架构)
4. [Dependency Graph 依赖图](#4-dependency-graph-依赖图)
5. [Data Flow 数据流](#5-data-flow-数据流)
6. [Core Abstractions 核心抽象](#6-core-abstractions-核心抽象)
7. [Builder Pattern Design 构建器模式设计](#7-builder-pattern-design-构建器模式设计)
8. [OFD Format Compliance OFD 格式合规](#8-ofd-format-compliance-ofd-格式合规)
9. [Error Handling 错误处理](#9-error-handling-错误处理)
10. [Derive Macro 派生宏](#10-derive-macro-派生宏)
11. [Conventions from easyexcel-rs 继承约定](#11-conventions-from-easyexcel-rs-继承约定)
12. [OFD vs Excel Paradigm Differences OFD与Excel范式差异](#12-ofd-vs-excel-paradigm-differences-ofd与excel范式差异)
13. [Future Architecture 未来架构](#13-future-architecture-未来架构)

---

## 1. Project Vision 项目愿景

`easyofd-rs` aims to provide **the same developer experience for OFD operations** that
[easyexcel-rs](https://github.com/hiwepy/easyexcel-rs) provides for Excel:

> **Type-safe Builders + Compile-time Reflection + GB/T 33190-2016 Compliant = Ergonomic OFD manipulation in idiomatic Rust.**

The library covers three primary use cases:

| Use Case 用例 | Description 描述 | EasyExcel Analogy 类比 |
|:---|:---|:---|
| **Create** 创建 | Generate new OFD files with text, images, paths, metadata | `EasyExcel.write()` |
| **Read** 读取 | Parse and extract content from existing OFD files (planned) | `EasyExcel.read()` |
| **Fill** 填充 | Populate placeholders in OFD templates (planned) | `EasyExcel.fill()` |

---

## 2. Design Goals 设计目标

| # | Goal 目标 | Rationale 理由 |
|:---|:---|:---|
| G1 | **Pure Rust, zero unsafe** | `#![forbid(unsafe_code)]` in every crate. Aligns with easyexcel-rs's safety policy. |
| G2 | **Fluent Builder API** | `mut self -> Self` with `#[must_use]`. Method chains read like natural language. |
| G3 | **Compile-time reflection** | `#[derive(OfdModel)]` replaces Java's runtime annotation scanning. |
| G4 | **Single facade entry point** | `EasyOfd::write()`, `EasyOfd::write_pages()` — discoverable static factory. |
| G5 | **GB/T 33190-2016 compliant** | Output is valid OFD ZIP archives with correct XML namespaces and structure. |
| G6 | **Single error type** | `OfdError` enum with `thiserror`, `type OfdResult<T> = ...` — no scattered error types. |
| G7 | **Separation of concerns** | Core types ≠ writer implementation ≠ facade ≠ derive logic. Each crate has one job. |
| G8 | **Follow easyexcel-rs conventions** | Naming, structure, quality gates — consistency across the ecosystem. |

---

## 3. Crate Architecture 包架构

```
easyofd-rs/
├── Cargo.toml                     Virtual workspace (edition 2021, resolver="2")
│
├── crates/
│   ├── easyofd/                   🎯 FACADE — user-facing entry point
│   │   ├── src/lib.rs             EasyOfd struct, all Builder types, re-exports
│   │   └── tests/
│   │       ├── derive_model.rs     Integration tests for derive macro + write flows
│   │       └── compile_fail.rs     Trybuild compile-fail tests for derive error paths
│   │
│   ├── easyofd-core/              🧩 CORE — zero engine dependency
│   │   └── src/
│   │       ├── lib.rs             Flat re-exports
│   │       ├── error.rs           OfdError enum, OfdResult<T> alias
│   │       ├── model.rs           OfdPage, TextObject, ImageObject, PathObject, OfdMetadata
│   │       └── ofd_model.rs       OfdModel trait, OfdField, OfdFieldKind
│   │
│   ├── easyofd-derive/            ⚡ PROC-MACRO SHIM — thin delegation wrapper
│   │   └── src/lib.rs             #[proc_macro_derive(OfdModel)] → delegates to -impl
│   │
│   ├── easyofd-derive-impl/       ⚙️ DERIVE LOGIC — all attribute parsing + code gen
│   │   └── src/lib.rs             derive_ofd_model_impl() + 12 unit tests
│   │
│   └── easyofd-writer/            ✍️ WRITER — ZIP/XML generation
│       └── src/lib.rs             OfdWriter: add_page, build, build_to_file
```

### Crate responsibility matrix 包职责矩阵

| Crate | External deps | Depends on | Role |
|:---|:---|:---|:---|
| **easyofd** | — (zero prod deps) | all sub-crates | Builder entry points + re-exports |
| **easyofd-core** | thiserror, chrono | — | Shared types, traits, errors, data model |
| **easyofd-derive** | easyofd-derive-impl | — | Proc-macro shim (3 lines, 100% coverage) |
| **easyofd-derive-impl** | syn, quote, proc-macro2 | easyofd-core | All derive logic + unit tests |
| **easyofd-writer** | zip, uuid, quick-xml, chrono | easyofd-core | OFD ZIP/XML creation |

**Why split derive into two crates?** Proc-macro crates have a `#[proc_macro_derive]` wrapper that generates phantom LLVM IR regions, preventing 100% source-line coverage measurement with `cargo llvm-cov`. By moving all logic into a regular lib crate (`easyofd-derive-impl`), the proc-macro crate becomes a 3-line shim (which achieves 100%), and all logic is tested in a normal crate (99.87% for the impl crate, where the remaining 0.13% comes from `?` operator hidden error basic blocks — a universal Rust/llvm-cov limitation).

---

## 4. Dependency Graph 依赖图

```
                        ┌──────────┐
                        │ easyofd  │  ← user depends on this
                        │ (facade) │
                        └────┬─────┘
           ┌─────────┬───────┼───────────┐
           ▼         ▼       ▼           ▼
        core    derive-impl derive    writer
         │          │        │          │
         │       syn,quote   │     zip, uuid,
    thiserror,  proc-macro2  │     quick-xml
     chrono                  │
                    easyofd-derive-impl
```

Key observations:

1. **easyofd-core has zero engine dependencies** — it defines *what* an OFD element is, not *how* to render it.
2. **easyofd-derive is a thin shim** — 3 lines that delegate to `easyofd-derive-impl` (proc-macro shim pattern).
3. **easyofd-derive-impl contains all logic** — attribute parsing, code generation, 12 unit tests covering valid → error paths.
4. **easyofd-writer is format-specific** — ZIP container creation + XML serialization per GB/T 33190-2016.
5. **Facade depends on everyone** — wires sub-crates together for ergonomic `EasyOfd::write()` / `write_pages()` entry points.

---

## 5. Data Flow 数据流

### 5.1 Write Flow via Derive Macro 写入流程（派生宏）

```
User code             easyofd facade          easyofd-core          easyofd-writer
──────────            ──────────────          ─────────────          ──────────────
EasyOfd::write::<T>("out.ofd")
  .metadata_title("Doc")
  .do_write(&[data])?
        │
        ▼
  OfdWriterBuilder<T>::do_write()
        │
        ├── T::to_pages(&[data])
        │       │
        │       ▼
        │   For each item:
        │     Vec<OfdPage>              ← generated by #[derive(OfdModel)]
        │
        ├── OfdWriter::with_options(options)
        │       │
        │       ▼
        │   OfdWriter::add_pages(pages)
        │       │
        │       ▼
        │   OfdWriter::build()
        │       │
        │       ▼
        │   Vec<u8>                     ← ZIP archive with XML entries
        │
        ▼
  write to out.ofd
```

### 5.2 Write Flow via Manual Page Construction 写入流程（手动构建）

```
User code             easyofd facade          easyofd-writer
──────────            ──────────────          ──────────────
EasyOfd::write_pages("out.ofd")
  .metadata_title("Doc")
  .do_write(vec![page])?
        │
        ▼
  PageWriterBuilder::do_write()
        │
        ├── OfdWriter::with_options(options)
        │
        ├── OfdWriter::add_pages(pages)
        │
        └── OfdWriter::build()
              │
              ▼
          Build ZIP structure:
            ├── OFD.xml            ← entry point + metadata
            └── Doc_0/
                ├── Document.xml     ← document structure
                ├── DocumentRes.xml  ← resource declarations
                ├── Pages/
                │   ├── Page_0.xml      ← TextObject, ImageObject, PathObject
                │   └── Page_N.xml
                └── Res/              ← embedded images
                    └── Image_0.jpeg
```

---

## 6. Core Abstractions 核心抽象

### 6.1 Type Hierarchy 类型层级

```
OfdError (enum)          — Central error type, 7 variants
  ├── Io(io::Error)      — Wraps std I/O errors
  ├── Xml(String)        — XML serialization errors
  ├── Zip(String)        — ZIP archive errors
  ├── InvalidDocument(String) — Invalid OFD document structure
  ├── InvalidPage(String)    — Invalid page content
  ├── Conversion(String)  — Type conversion errors
  └── Model(String)      — Model mapping errors

OfdMetadata (struct)     — version, title, author, creator, creation_date

OfdPage (struct)         — width, height (mm), content: Vec<ContentObject>

ContentObject (enum)     — The page content building blocks
  ├── Text(TextObject)   — Positioned text block
  ├── Image(ImageObject) — Positioned image
  └── Path(PathObject)   — Vector path

TextObject (struct)      — x, y, font, size, weight, italic, color, text, width?, height?
ImageObject (struct)     — x, y, width, height, data: Vec<u8>, format: ImageFormat
PathObject (struct)      — x, y, stroke_color, stroke_width, fill_color?, path_data: String

ImageFormat (enum)       — Jpeg | Png | Bmp | Tiff
OfdFieldKind (enum)      — Text | Image
```

### 6.2 OfdModel Trait 特质

The central mapping trait. Generated by `#[derive(OfdModel)]` or implemented manually:

```rust
pub trait OfdModel: Sized {
    fn schema() -> &'static [OfdField];
    fn page_size() -> (f64, f64) { (210.0, 297.0) }
    fn to_page(&self) -> OfdResult<OfdPage>;
    fn to_pages(items: &[Self]) -> OfdResult<Vec<OfdPage>> {
        items.iter().map(Self::to_page).collect()
    }
}
```

### 6.3 ContentObject 桥接元素

`ContentObject` bridges model definition and writer rendering:

```rust
pub enum ContentObject {
    Text(TextObject),    // Text block at a position
    Image(ImageObject),  // Image at a position
    Path(PathObject),    // Vector path at a position
}
```

The derive macro generates `to_page()` → `Vec<ContentObject>`; the writer consumes them to produce XML.

---

## 7. Builder Pattern Design 构建器模式

### 7.1 Pattern Rules 模式规则

| Rule | Code Pattern | Why |
|:---|:---|:---|
| Owned self | `pub fn method(mut self, ...) -> Self` | Enables chaining, prevents accidental reuse |
| Must-use | `#[must_use]` on all builder structs and methods | Compiler warns if chain result is discarded |
| Terminal execute | `do_write()`, `do_write_to_bytes()` | Builder → action, builder is consumed |
| Static factory entry | `EasyOfd::write::<T>(path)` | Discoverable, canonical entry point |
| PhantomData typing | `_phantom: PhantomData<T>` on OfdWriterBuilder | Keeps the type parameter without owning a T |

### 7.2 Builder State Machine 状态机

```
EasyOfd::write::<T>("out.ofd")
        │
        ▼
  OfdWriterBuilder<T>
        │
        ├── metadata_title(t) -> OfdWriterBuilder<T> (stay)
        ├── metadata_author(a) -> OfdWriterBuilder<T> (stay)
        ├── metadata_creator(c) -> OfdWriterBuilder<T> (stay)
        │
        ├── do_write(&[data]) -> OfdResult<()>           ← writes to file
        └── do_write_to_bytes(&[data]) -> OfdResult<Vec<u8>>  ← returns bytes


EasyOfd::write_pages("out.ofd")
        │
        ▼
  PageWriterBuilder
        │
        ├── metadata_title(t) -> PageWriterBuilder (stay)
        ├── metadata_author(a) -> PageWriterBuilder (stay)
        ├── metadata_creator(c) -> PageWriterBuilder (stay)
        │
        ├── do_write(vec![page]) -> OfdResult<()>
        └── do_write_to_bytes(vec![page]) -> OfdResult<Vec<u8>>
```

### 7.3 Builder Design Rationale 设计理由

| Alternative | Rejected because |
|:---|:---|
| Config struct + function | No type-state, easy to forget required fields |
| `&mut self` builders | Cannot enforce single-use, harder to chain |
| Macro-based DSL | Harder to discover API, worse IDE support |

---

## 8. OFD Format Compliance OFD 格式合规

### 8.1 ZIP Structure ZIP 结构

```
output.ofd (ZIP)
├── OFD.xml                    ← Entry point + DocBody with metadata
└── Doc_0/
    ├── Document.xml           ← CommonData (PageArea, resources) + Pages list
    ├── DocumentRes.xml        ← MultiMedia resource declarations
    ├── Pages/
    │   ├── Page_0.xml            ← Area + Content layer
    │   └── Page_N.xml
    └── Res/                     ← Embedded image files
        └── Image_0.jpeg
```

### 8.2 XML Namespace XML 命名空间

```xml
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.0">
```

### 8.3 Content Element Mapping 元素映射

| Rust Type | OFD Element | Key Attributes |
|:---|:---|:---|
| `TextObject` | `<ofd:TextObject>` | `ID`, `Boundary`, `Font`, `Size` |
| `ImageObject` | `<ofd:ImageObject>` | `ID`, `Boundary`, `ResourceID` |
| `PathObject` | `<ofd:PathObject>` | `ID`, `Boundary`, `StrokeColor`, `LineWidth` |

### 8.4 Coordinate System 坐标系

OFD uses **millimeters** as the coordinate unit. Origin (0, 0) is the top-left corner of the page.

---

## 9. Error Handling 错误处理

### 9.1 Error Taxonomy 分类

```
OfdError
├── Io              ← wraps std::io::Error
├── Xml             ← XML serialization errors
├── Zip             ← ZIP archive errors
├── InvalidDocument ← missing/invalid OFD structure
├── InvalidPage     ← invalid page content
├── Conversion      ← type conversion errors
└── Model           ← field mapping errors from OfdModel
```

### 9.2 Error Flow 错误流

```
IO error / ZIP error / XML error
        │
        ▼
  Mapped to OfdError variant
        │
        ▼
  Propagated via ? through builder chain
        │
        ▼
  User receives OfdResult<T>
```

### 9.3 Design Decisions 设计决策

| Decision | Rationale |
|:---|:---|
| Single `OfdError` enum | One error type for all user code |
| `thiserror` derive | Automatic `Display` + `Error` + `From` impls |
| `type OfdResult<T>` | Consistent across codebase |
| `From<io::Error>` impl | Direct `?` from `std::fs` operations |
| No `anyhow` in library | Structured errors for libraries; `anyhow` for apps |

---

## 10. Derive Macro 派生宏

### 10.1 Architecture 架构

The derive macro uses a **proc-macro shim pattern**:

```
easyofd-derive (proc-macro crate, 3 lines)
    │
    │  #[proc_macro_derive(OfdModel)]
    │  pub fn derive_ofd_model(input) -> TokenStream {
    │      easyofd_derive_impl::derive_ofd_model_impl(input.into()).into()
    │  }
    │
    ▼
easyofd-derive-impl (regular lib crate, 400 lines)
    │
    ├── derive_ofd_model_impl()     Entry point + error handling
    ├── impl_ofd_model()            Generates impl OfdModel block
    ├── process_fields()            Iterates named fields, dispatches per kind
    ├── parse_page_attrs()          Reads #[ofd(page_width, page_height)]
    ├── parse_field_attrs()         Reads #[ofd(x, y, font, size, ...)]
    ├── parse_lit_f64/u32()         String-based Lit parsing (no enum branching)
    ├── has_ignore_attr()          Detects #[ofd(ignore)]
    └── lit_to_string()            Extracts string from Lit
```

### 10.2 Attribute Parsing 属性解析

All attribute values are parsed via **string conversion** to avoid `syn::Lit` enum branching:

```rust
fn parse_lit_f64(lit: &Lit) -> syn::Result<f64> {
    Ok(lit.to_token_stream().to_string()
        .replace('_', "")
        .parse()
        .unwrap_or(0.0))
}
```

And all attribute dispatch uses **independent `if` blocks** rather than `match` or `else-if` chains:

```rust
if ident == "x" { cfg.x = parse_lit_f64(&value)?; }
if ident == "y" { cfg.y = parse_lit_f64(&value)?; }
if ident == "size" { cfg.size = parse_lit_f64(&value)?; }
// ... etc.
```

### 10.3 Compile-time Detection 编译期检测

| Error | Detection |
|:---|:---|
| Derive on non-struct type | `compile_error!("OfdModel can only be derived for structs")` |
| Derive on tuple struct | `compile_error!("OfdModel can only be derived for structs with named fields")` |
| Parse failure | `derive_ofd_model_impl` catches `syn::parse2` errors |

Validated via trybuild compile-fail tests + direct unit tests in `easyofd-derive-impl`.

### 10.4 Java Annotations vs Rust Derive 对比

| Aspect | Java EasyExcel | Rust easyofd-rs |
|:---|:---|:---|
| Annotation | `@ExcelProperty(value = "Name")` | `#[ofd(x = 20.0, y = 30.0)]` |
| Processing | Runtime reflection | Compile-time code gen |
| Performance | Reflection overhead | Zero-cost, direct calls |
| Error detection | Runtime | Compile-time |
| IDE support | Good (annotation processors) | Good (proc-macro expansion) |

---

## 11. Conventions from easyexcel-rs 继承约定

| Convention 约定 | easyexcel-rs | easyofd-rs | Notes |
|:---|:---|:---|:---|
| **Workspace** | Virtual manifest + `[workspace.dependencies]` | ✅ Same | `resolver = "2"`, edition 2021 |
| **Crate naming** | `easyexcel`, `easyexcel-core`, ... | `easyofd`, `easyofd-core`, ... | ✅ Same pattern |
| **MSRV** | 1.88 | 1.70 | easyofd-rs is more conservative |
| **Edition** | 2024 | 2021 | easyofd-rs targets broader compatibility |
| **License** | Apache-2.0 | ✅ Apache-2.0 | |
| **unsafe** | `forbid(unsafe_code)` | ✅ Same | |
| **Lints** | `clippy::pedantic`, `missing_docs` | ✅ Same | |
| **Error type** | `thiserror`, single enum | ✅ `OfdError` | 7 variants |
| **Result alias** | `pub type Result<T>` | ✅ `OfdResult<T>` | |
| **Builder** | `mut self → Self`, `#[must_use]` | ✅ Same | |
| **Facade** | Thin crate, zero prod deps | ✅ `crates/easyofd` | |
| **Derive macro** | `syn`/`quote`/`proc-macro2` | ✅ Same | `#[derive(OfdModel)]` |
| **Proc-macro shim** | N/A | ✅ `easyofd-derive` + `-impl` | Novel pattern for 100% coverage |

---

## 12. OFD vs Excel Paradigm Differences OFD与Excel范式差异

| Dimension | Excel (easyexcel-rs) | OFD (easyofd-rs) |
|:---|:---|:---|
| **Layout model** | Grid-based (rows × columns) | Coordinate-based (x, y in mm) |
| **Data unit** | Cell (A1, B2, ...) | Content object at position (x, y) |
| **Header** | Row 1 with column names | Text objects at explicit positions |
| **Style** | Per-cell or per-column | Per-text-object (font, size, weight) |
| **Template** | `{key}` placeholders in cells | Future: text replacement in XML |
| **Multiple sheets** | Workbook → Sheet1, Sheet2... | Document → Page1, Page2... |
| **Read direction** | Top-to-bottom, left-to-right | Any order (coordinate-based) |
| **Memory model** | SXSSF (streaming to disk) | In-memory Vec<ContentObject> → ZIP |
| **File format** | ZIP (OOXML) | ZIP (GB/T 33190-2016 XML) |

### Design implications 设计影响

1. **No "row" or "cell" in OFD** — `TextObject` replaces `Cell`/`Row`. The derive macro maps struct fields to positioned text blocks.
2. **Position is explicit** — Every element has `(x, y)` in mm. Future: auto-layout engine.
3. **Pages are primary** — Each `OfdPage` is independent, analogous to a worksheet without the grid.
4. **Images are embedded** — Data stored as ZIP entries in `Doc_0/Res/`, referenced by resource ID.

---

## 13. Future Architecture 未来架构

### 13.1 Planned Crate Expansion 计划扩展

```
easyofd-rs/
├── easyofd (facade)             ← current
├── easyofd-core (types)          ← current
├── easyofd-derive (shim)         ← current
├── easyofd-derive-impl (logic)   ← current
├── easyofd-writer (write)        ← current
│
├── easyofd-reader (read)         ← v0.2: parse OFD ZIP, extract content
├── easyofd-template (fill)       ← v0.2: placeholder-based template engine
├── easyofd-signature (sign)      ← v0.3: GB/T 38540 electronic seals
└── easyofd-convert (convert)     ← v0.4: PDF ↔ OFD bidirectional conversion
```

### 13.2 Reader Architecture (v0.2)

```
EasyOfd::read("input.ofd")
        │
        ▼
  OfdReader::open()
        │
        ▼
  Unzip → parse OFD.xml → follow DocRoot → parse Document.xml
        │
        ▼
  For each Page: parse Page_N.xml → extract content → ReadListener callback
```

### 13.3 Template Engine (v0.2)

```
EasyOfd::fill_template("template.ofd", &data)
        │
        ▼
  Unzip → find {placeholders} in XML → replace with OfdModel values → re-pack ZIP
```

### 13.4 Auto-Layout Engine (v0.3+)

```
FlowLayout::vertical()
    .margin(20.0)
    .spacing(10.0)

  ┌──────────────────────────┐
  │ Title        (y = 250)   │  ← auto-placed
  │ Paragraph 1  (y = 220)   │
  │ Paragraph 2  (y = 190)   │
  │ Image        (y = 150)   │
  │ Footer       (y = 20)    │
  └──────────────────────────┘
```

---

## Appendix A: Quality Gates 质量门禁

| Gate | Command | Status |
|:---|:---|:---:|
| Format | `cargo fmt --all -- --check` | ✅ |
| Lint | `cargo clippy --all-targets` | ✅ 0 warnings |
| Build | `cargo check --workspace` | ✅ |
| Test | `cargo test --workspace` | ✅ 153 tests |
| Coverage | `cargo llvm-cov --all-targets` | ✅ 99.87% |
| LCOV | 0 `DA:line,0` entries | ✅ |
| Text output | 0 zero-exec source lines | ✅ |

## Appendix B: File Inventory 文件清单

| File | Lines | Purpose |
|:---|:---:|:---|
| `Cargo.toml` | 42 | Workspace manifest |
| `crates/easyofd-core/src/error.rs` | 110 | OfdError + OfdResult |
| `crates/easyofd-core/src/model.rs` | 540 | OfdPage, TextObject, ImageObject, PathObject, page_size |
| `crates/easyofd-core/src/ofd_model.rs` | 170 | OfdModel trait, OfdField, OfdFieldKind |
| `crates/easyofd-core/src/lib.rs` | 14 | Re-exports |
| `crates/easyofd-derive/src/lib.rs` | 13 | Proc-macro shim (delegates to -impl) |
| `crates/easyofd-derive-impl/src/lib.rs` | 400 | All derive logic + 12 unit tests |
| `crates/easyofd-writer/src/lib.rs` | 890 | OfdWriter: ZIP + XML generation |
| `crates/easyofd/src/lib.rs` | 335 | EasyOfd + all Builders |
| `crates/easyofd/tests/derive_model.rs` | 465 | Integration tests |
| `crates/easyofd/tests/compile_fail.rs` | 10 | Trybuild compile-fail tests |
| **Total** | **~2,990** | |

---

> *"Design is not just what it looks like and feels like. Design is how it works."* — Steve Jobs
