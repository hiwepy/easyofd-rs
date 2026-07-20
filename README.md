# easyofd-rs &middot; [English](#easyofd-rs) | [中文](#easyofd-rs-中文)

> **An idiomatic Rust library for quick OFD document operations.**  
> Inspired by [Alibaba EasyExcel](https://github.com/alibaba/easyexcel)'s builder-pattern API design.  
> **纯 Rust &middot; 零 unsafe &middot; Builder 模式 &middot; GB/T 33190-2016 合规**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance)

---

`easyofd-rs` 为 OFD（开放版式文档）操作提供流畅、类型安全的 Builder API。  
OFD 是中国国家标准 GB/T 33190-2016，广泛应用于政府和企业文档工作流——尤其是电子发票、公文和归档场景。

`easyofd-rs` provides a fluent, type-safe builder API for all common OFD tasks: **creation**, **reading**, and **template filling**.  
OFD is the Chinese national standard GB/T 33190-2016, widely used for electronic invoices, official documents, and archival purposes.

---

## Table of Contents | 目录

- [Features 功能](#features--功能)
- [Architecture 架构](#architecture--架构)
- [Quick Start 快速开始](#quick-start--快速开始)
  - [Derive Model + One-liner Write 派生宏 + 一行写入](#1-derive-model--one-liner-write-派生宏--一行写入)
  - [Manual Page Construction 手动构建页面](#2-manual-page-construction-手动构建页面)
  - [Multiple Pages with Images and Paths 多页含图片和路径](#3-multiple-pages-with-images-and-paths-多页含图片和路径)
- [API Reference API 参考](#api-reference--api-参考)
  - [EasyOfd Entry Points 入口方法](#easyofd--entry-points-入口方法)
  - [OfdWriterBuilder](#ofdwriterbuilder)
  - [PageWriterBuilder](#pagewriterbuilder)
  - [Core Types 核心类型](#core-types--核心类型)
- [Design Principles 设计原则](#design-principles--设计原则)
- [Roadmap 路线图](#roadmap--路线图)
- [License 许可证](#license--许可证)

---

## Features | 功能

| Feature 功能 | Status 状态 | Description 描述 |
|:---|:---:|:---|
| Create OFD (text, metadata) 创建含文本、元数据的 OFD | ✅ v0.1 | Build complete OFD ZIP archives |
| `#[derive(OfdModel)]` macro 派生宏 | ✅ v0.1 | 编译期反射，替代 Java 注解扫描 |
| Fluent Builder API 流式构建器 | ✅ v0.1 | `mut self -> Self` + `#[must_use]` |
| Multi-page support 多页支持 | ✅ v0.1 | Each data item -> one page |
| Image embedding (JPEG/PNG/BMP/TIFF) 图片嵌入 | ✅ v0.1 | Positioned image with resource embedding |
| Vector paths (lines, rectangles) 矢量路径 | ✅ v0.1 | hline, vline, rect with stroke/fill |
| OFD Reader OFD 读取 | 🚧 v0.2 | Parse OFD, extract content |
| Template filling 模板填充 | 🚧 v0.2 | Placeholder-based template engine |
| Digital signatures 电子签章 | 🚧 v0.3 | GB/T 38540 electronic seals |
| Custom fonts 自定义字体 | 🚧 v0.3 | TTF/OTF embedding |
| PDF ↔ OFD conversion 互转 | 🚧 v0.4 | PDF/OFD bidirectional |

---

## Architecture | 架构

```
┌───────────────────────────────────────────────┐
│                  easyofd                       │
│         (facade · Builder entry points)        │
│    EasyOfd::write()  write_pages()  ...        │
├───────┬───────┬───────────────────────────────┤
│ core  │derive │ writer                        │
│ types │macro  │  ZIP + XML (GB/T 33190)       │
│ errors│#[derive│  thiserror, zip, uuid,       │
│ traits│(OfdMod- │  quick-xml, chrono           │
│ model │  el)    │                              │
└───────┴───────┴───────────────────────────────┘
```

| Crate 子包 | Purpose 用途 | Dependencies 依赖 |
|:---|:---|---|
| **easyofd** | Facade + Builder entry points 外观入口 | All sub-crates |
| **easyofd-core** | Types, traits, errors, model 核心抽象 | thiserror, chrono |
| **easyofd-derive** | `#[derive(OfdModel)]` proc-macro 编译期反射 | syn, quote, proc-macro2 |
| **easyofd-writer** | OFD ZIP/XML writer 合规写入器 | easyofd-core, zip, uuid, quick-xml |

---

## Quick Start | 快速开始

Add to your `Cargo.toml`:

```toml
[dependencies]
easyofd = "0.1"
```

### 1. Derive Model + One-liner Write 派生宏 + 一行写入

```rust
use easyofd::{EasyOfd, OfdModel};

#[derive(OfdModel)]
#[ofd(page_width = 210.0, page_height = 297.0)]   // A4
struct Invoice {
    #[ofd(x = 20.0, y = 30.0, size = 18.0, bold)]
    title: String,

    #[ofd(x = 20.0, y = 50.0)]
    amount: String,

    #[ofd(x = 20.0, y = 70.0)]
    note: String,

    // Fields with `#[ofd(ignore)]` are excluded from output
    #[ofd(ignore)]
    internal_id: u64,
}

// One-liner: multiple data items → multiple pages
let data = vec![
    Invoice { title: "Invoice #001".into(), amount: "$100.00".into(),
              note: "First".into(), internal_id: 1 },
    Invoice { title: "Invoice #002".into(), amount: "$200.00".into(),
              note: "Second".into(), internal_id: 2 },
];

EasyOfd::write::<Invoice>("invoices.ofd")
    .metadata_title("Invoices")
    .metadata_author("easyofd-rs")
    .do_write(&data)?;
```

### 2. Manual Page Construction 手动构建页面

```rust
use easyofd::{EasyOfd, OfdPage, TextObject, page_size};

let mut page = OfdPage::new(page_size::A4.0, page_size::A4.1);
page.add_text(
    TextObject::new(20.0, 30.0, "Hello OFD!")
        .font("SimHei")
        .size(24.0)
        .bold()
);

// Write to bytes or file
let bytes = EasyOfd::write_pages_to_bytes(vec![page])?;
std::fs::write("hello.ofd", bytes)?;
```

### 3. Multiple Pages with Images and Paths 多页含图片和路径

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

## API Reference | API 参考

### EasyOfd | Entry Points 入口方法

| Method 方法 | Signature 签名 | Returns 返回 |
|:---|---|:---|
| `write::<T>` | `(path: impl Into<String>)` | `OfdWriterBuilder<T>` |
| `write_pages` | `(path: impl Into<String>)` | `PageWriterBuilder` |
| `write_pages_to` | `(path, pages: Vec<OfdPage>)` | `OfdResult<()>` |
| `write_pages_to_bytes` | `(pages: Vec<OfdPage>)` | `OfdResult<Vec<u8>>` |

### OfdWriterBuilder\<T: OfdModel\>

| Method | Signature | Description |
|:---|---|:---|
| `metadata_title(mut self, title) -> Self` | Set document title 设置文档标题 |
| `metadata_author(mut self, author) -> Self` | Set document author 设置作者 |
| `metadata_creator(mut self, creator) -> Self` | Set creator application name 设置创建程序 |
| `do_write(&self, data: &[T]) -> OfdResult<()>` | Write to file path 写入文件 |
| `do_write_to_bytes(&self, data: &[T]) -> OfdResult<Vec<u8>>` | Write to in-memory bytes 写入内存 |

### PageWriterBuilder

| Method | Signature | Description |
|:---|---|:---|
| `metadata_title(mut self, title) -> Self` | Set document title 设置文档标题 |
| `metadata_author(mut self, author) -> Self` | Set document author 设置作者 |
| `metadata_creator(mut self, creator) -> Self` | Set creator application name 设置创建程序 |
| `do_write(&self, pages: Vec<OfdPage>) -> OfdResult<()>` | Write to file path 写入文件 |
| `do_write_to_bytes(&self, pages: Vec<OfdPage>) -> OfdResult<Vec<u8>>` | Write to in-memory bytes 写入内存 |

### Core Types | 核心类型

<details>
<summary>Click to expand 点击展开</summary>

#### OfdModel Trait 派生宏 Trait

```rust
pub trait OfdModel: Sized {
    fn schema() -> &'static [OfdField];
    fn page_size() -> (f64, f64) { (210.0, 297.0) }
    fn to_page(&self) -> OfdResult<OfdPage>;
    fn to_pages(items: &[Self]) -> OfdResult<Vec<OfdPage>>;
}
```

#### `#[ofd(...)]` Field Attributes 字段属性

| Attribute 属性 | Type 类型 | Default 默认 | Description 描述 |
|:---|:---|:---|:---|
| `x` | f64 | required 必填 | X position in mm |
| `y` | f64 | required 必填 | Y position in mm |
| `font` | str | `"SimSun"` | Font family name 字体名称 |
| `size` | f64 | `12.0` | Font size in pt 字号 |
| `bold` | flag | false | Bold text (weight 700) 加粗 |
| `weight` | u32 | `400` | Font weight 字重 |
| `italic` | flag | false | Italic text 斜体 |
| `color` | u32 | `0` | RGB color as hex (e.g. `0xFF0000`) 颜色 |
| `kind` | str | `"text"` | Render kind: `"text"` or `"image"` 渲染类型 |
| `img_width` | f64 | `30.0` | Image width in mm (kind="image") 图片宽度 |
| `img_height` | f64 | `30.0` | Image height in mm (kind="image") 图片高度 |
| `ignore` | flag | false | Skip this field entirely 忽略此字段 |

#### Struct-level Attributes 结构体级属性

| Attribute 属性 | Type 类型 | Default 默认 | Description 描述 |
|:---|:---|:---|:---|
| `page_width` | f64/integer | `210.0` | Page width in mm 页面宽度 |
| `page_height` | f64/integer | `297.0` | Page height in mm 页面高度 |

#### Content Objects 内容对象

| Type 类型 | Description 描述 |
|:---|:---|
| `TextObject` | Positioned text: x, y, font, size, weight, color, italic 文本块 |
| `ImageObject` | Positioned image: x, y, width, height, data, format (JPEG/PNG/BMP/TIFF) 图片 |
| `PathObject` | Vector path: x, y, stroke_color, stroke_width, fill_color, path_data 矢量路径 |
| `OfdPage` | Page: width, height, content: Vec<ContentObject> 页面 |

#### TextObject Builder Methods 文本构建器方法

| Method | Description |
|:---|:---|
| `TextObject::new(x, y, text)` | Create with position and text |
| `.font("SimHei")` | Set font family |
| `.size(24.0)` | Set font size in pt |
| `.bold()` | Set bold (weight 700) |
| `.italic()` | Set italic |
| `.color(0xFF0000)` | Set color as RGB hex |

#### ImageObject Constructors 图片构造方法

| Method | Description |
|:---|:---|
| `ImageObject::new(x, y, w, h, data, format)` | Generic constructor |
| `ImageObject::jpeg(x, y, w, h, data)` | JPEG shortcut |
| `ImageObject::png(x, y, w, h, data)` | PNG shortcut |

#### PathObject Constructors 路径构造方法

| Method | Description |
|:---|:---|
| `PathObject::new(x, y, path_data)` | Generic constructor |
| `PathObject::hline(x1, y, x2)` | Horizontal line |
| `PathObject::vline(x, y1, y2)` | Vertical line |
| `PathObject::rect(x, y, w, h)` | Rectangle outline |
| `.stroke_color(0x333333)` | Set stroke color |
| `.stroke_width(1.5)` | Set stroke width |
| `.fill_color(0xCCCCCC)` | Set fill color |

#### Page Sizes 页面尺寸 (mm)

| Constant 常量 | Dimensions 尺寸 |
|:---|:---|
| `page_size::A4` | `(210.0, 297.0)` |
| `page_size::A4_LANDSCAPE` | `(297.0, 210.0)` |
| `page_size::A3` | `(297.0, 420.0)` |
| `page_size::LETTER` | `(215.9, 279.4)` |

#### Errors 错误类型

| Variant 变体 | Description 描述 |
|:---|:---|
| `OfdError::Io(e)` | Wraps `std::io::Error` I/O 错误 |
| `OfdError::Xml(msg)` | XML serialization error XML 序列化错误 |
| `OfdError::Zip(msg)` | ZIP archive error ZIP 归档错误 |
| `OfdError::InvalidDocument(msg)` | Invalid OFD document structure 无效文档结构 |
| `OfdError::InvalidPage(msg)` | Invalid page content 无效页面内容 |
| `OfdError::Conversion(msg)` | Type conversion error 类型转换错误 |
| `OfdError::Model(msg)` | Model mapping error 模型映射错误 |

```rust
pub type OfdResult<T> = std::result::Result<T, OfdError>;
```

</details>

---

## Design Principles | 设计原则

| Principle 原则 | Practice 实践 |
|:---|:---|
| **Pure Rust 纯 Rust** | `#![forbid(unsafe_code)]` in every crate |
| **Type-safe builders 类型安全构建器** | `mut self -> Self`, `#[must_use]` on all builders |
| **Compile-time reflection 编译期反射** | `#[derive(OfdModel)]` generates mapping code — no runtime reflection |
| **Trait extensibility Trait 可扩展** | `OfdModel` trait for custom implementations |
| **Single error type 统一错误类型** | `OfdError` enum with `thiserror`, single `OfdResult<T>` alias |
| **Zero-cost abstractions 零成本抽象** | Builder chains compile to direct calls, derive macros expand at compile time |
| **Separation of concerns 关注点分离** | Core types ≠ writer implementation ≠ facade |
| **Inspired by EasyExcel 继承设计** | Same builder · derive · facade patterns as `easyexcel-rs` |

---

## Roadmap | 路线图

| Phase 阶段 | Focus 重点 | Key Deliverables 关键交付 |
|:---:|:---|:---|
| **v0.1** ✅ | Foundation 基础 | Workspace, 4 crates, core types, ZIP/XML writer, derive macro, builder API, 98.14% coverage |
| **v0.2** 🚧 | Reader & Template 读取与模板 | OFD reader, template placeholder filling, text rendering with font embedding |
| **v0.3** | Security 安全 | Electronic seals & signatures (GB/T 38540), encryption |
| **v0.4** | Converters 转换 | PDF ↔ OFD bidirectional conversion, image format conversion |
| **v1.0** | Stable 稳定版 | Stable API, full test coverage, performance benchmarks, documentation |

---

## License | 许可证

Apache-2.0

---

## Related Projects | 相关项目

- [easyexcel-rs](https://github.com/hiwepy/easyexcel-rs) — Rust port of Alibaba EasyExcel
- [easyexcel](https://github.com/alibaba/easyexcel) — Original Java library by Alibaba
- [ofd-rs](https://crates.io/crates/ofd-rs) — Lower-level OFD writer crate
- [ofd-core](https://crates.io/crates/ofd-core) — OFD XML parsing & data model crate

---

<p align="center">
  <sub>Built with Rust 🦀 · Follows <a href="https://github.com/hiwepy/easyexcel-rs">easyexcel-rs</a> conventions</sub>
</p>
