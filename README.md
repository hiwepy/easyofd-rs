# easyofd-rust &middot; [English](#easyofd-rust) | [中文](#easyofd-rust-中文)

> **An idiomatic Rust library for quick OFD document operations.**  
> Inspired by [Alibaba EasyExcel](https://github.com/alibaba/easyexcel)'s builder-pattern API design.  
> **纯 Rust &middot; 零 unsafe &middot; Builder 模式 &middot; GB/T 33190-2016 合规**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance)

---

`easyofd-rust` 为 OFD（开放版式文档）操作提供流畅、类型安全的 API：**创建**、**读取**、**模板填充**、**电子签章**、**PDF 互转**。

OFD is the Chinese national standard GB/T 33190-2016, widely used for electronic invoices, official documents, and archival purposes.

---

## Table of Contents 目录

- [Features 功能](#features-功能)
- [Architecture 架构](#architecture-架构)
- [Quick Start 快速开始](#quick-start-快速开始)
  - [1. Write with Derive 派生宏写入](#1-write-with-derive-派生宏写入)
  - [2. Manual Write 手动构建写入](#2-manual-write-手动构建写入)
  - [3. Read 读取](#3-read-读取)
  - [4. Template Fill 模板填充](#4-template-fill-模板填充)
  - [5. Signature 电子签章](#5-signature-电子签章)
- [API Reference API 参考](#api-reference--api-参考)
- [Design Principles 设计原则](#design-principles--设计原则)
- [Roadmap 路线图](#roadmap--路线图)

---

## Features 功能

| Feature 功能 | Status | Description 描述 |
|:---|:---:|:---|
| Create OFD 创建 | ✅ v0.1 | Text, images, paths, metadata; fluent Builder API |
| `#[derive(OfdModel)]` 派生宏 | ✅ v0.1 | Compile-time reflection; zero runtime cost |
| Read OFD 读取 | ✅ v0.2 | SAX-based parsing; text extraction; structured content |
| Template fill 模板填充 | ✅ v0.2 | `{key}` placeholder replacement; binary-preserving |
| Digital signatures 电子签章 | ✅ v0.3 | GB/T 38540 seals; SM2WithSM3 / SHA256WithRSA |
| Custom fonts 自定义字体 | ✅ v0.3 | TTF/OTF embedding; EmbeddedFont + FontFormat |
| PDF ↔ OFD 互转 | ✅ v0.4 | Bidirectional conversion API; ConvertOptions |
| Multi-page 多页 | ✅ v0.1 | Each data item → one page |
| Vector paths 矢量路径 | ✅ v0.1 | hline, vline, rect with stroke/fill |

---

## Architecture 架构

```
easyofd-rust (9 crates)
├── easyofd            🎯 Facade — EasyOfd::write/read/fill_template
├── easyofd-core       🧩 Types, traits, errors, data model
├── easyofd-derive     ⚡ Proc-macro shim (6 lines)
├── easyofd-derive-impl ⚙️ All derive logic (400 lines)
├── easyofd-reader     📖 SAX-based OFD parsing
├── easyofd-writer     ✍️ ZIP/XML generation + custom fonts
├── easyofd-template   📋 Placeholder replacement engine
├── easyofd-signature  🔐 GB/T 38540 electronic seals
└── easyofd-convert    🔄 PDF ↔ OFD bridge
```

---

## Quick Start 快速开始

```toml
[dependencies]
easyofd = "0.4"
```

### 1. Write with Derive 派生宏写入

```rust
use easyofd::{EasyOfd, OfdModel};

#[derive(OfdModel)]
#[ofd(page_width = 210.0, page_height = 297.0)]  // A4
struct Invoice {
    #[ofd(x = 20.0, y = 30.0, size = 18.0, bold)]
    title: String,
    #[ofd(x = 20.0, y = 50.0)]
    amount: String,
    #[ofd(x = 20.0, y = 70.0, kind = "image", img_width = 30.0, img_height = 30.0)]
    seal: Vec<u8>,
    #[ofd(ignore)]
    internal_id: u64,
}

let data = vec![Invoice { /* ... */ }];
EasyOfd::write::<Invoice>("out.ofd")
    .metadata_title("Invoices")
    .do_write(&data)?;
```

### 2. Manual Write 手动构建写入

```rust
use easyofd::{EasyOfd, OfdPage, TextObject, ImageObject, PathObject};

let mut page = OfdPage::new(210.0, 297.0);
page.add_text(TextObject::new(20.0, 30.0, "Title").size(24.0).bold());
page.add_path(PathObject::hline(20.0, 55.0, 190.0));
page.add_image(ImageObject::jpeg(150.0, 30.0, 30.0, 30.0, std::fs::read("seal.jpg")?));

EasyOfd::write_pages("doc.ofd")
    .metadata_title("My Document")
    .do_write(vec![page])?;
```

### 3. Read 读取

```rust
let reader = EasyOfd::read("input.ofd")?;
println!("Pages: {}", reader.page_count());

// Extract per-page text
for (i, text) in reader.extract_text().iter().enumerate() {
    println!("Page {}: {}", i + 1, text);
}

// Or all text at once
let all = reader.extract_all_text();

// Access structured content
for page in reader.pages() {
    for obj in &page.content {
        match obj {
            ContentObject::Text(t) => println!("Text: {}", t.text),
            ContentObject::Image(_) => println!("Image found"),
            ContentObject::Path(_) => println!("Path found"),
        }
    }
}
```

### 4. Template Fill 模板填充

```rust
use std::collections::HashMap;

let mut data = HashMap::new();
data.insert("title".into(), "Invoice #001".into());
data.insert("amount".into(), "$1,234.00".into());
data.insert("date".into(), "2026-01-15".into());

EasyOfd::fill_template("template.ofd", &data)?
    .save("output.ofd")?;
```

### 5. Signature 电子签章

```rust
use easyofd::{OfdSignatureBuilder, ElectronicSeal, SignatureAlgorithm};

let seal = ElectronicSeal {
    image_data: std::fs::read("seal.png")?,
    name: "Company Seal".into(),
    position: (150.0, 200.0),
    page: 1,
};

OfdSignatureBuilder::new("document.ofd")
    .seal(seal)
    .algorithm(SignatureAlgorithm::Sm2WithSm3)
    .sign()?
    .save("signed.ofd")?;
```

---

## API Reference  API 参考

### EasyOfd Entry Points 入口

| Method 方法 | Signature 签名 | Description |
|:---|:---|:---|
| `write::<T>` | `(path) -> OfdWriterBuilder<T>` | 派生宏写入 |
| `write_pages` | `(path) -> PageWriterBuilder` | 手动构建写入 |
| `read` | `(path) -> OfdResult<OfdReader>` | 读取 OFD |
| `fill_template` | `(path, &HashMap) -> OfdResult<OfdTemplateFiller>` | 模板填充 |
| `write_pages_to` | `(path, Vec<OfdPage>) -> OfdResult<()>` | 直接写入文件 |
| `write_pages_to_bytes` | `(Vec<OfdPage>) -> OfdResult<Vec<u8>>` | 写入内存 |
| `read_from_bytes` | `(&[u8]) -> OfdResult<OfdReader>` | 从内存读取 |

### OfdWriterBuilder\<T: OfdModel\>

| Method | Signature | Description |
|:---|:---|:---|
| `metadata_title` | `(title) -> Self` | 设置文档标题 |
| `metadata_author` | `(author) -> Self` | 设置作者 |
| `metadata_creator` | `(creator) -> Self` | 设置创建程序 |
| `do_write` | `(&self, &[T]) -> OfdResult<()>` | 写入文件 |
| `do_write_to_bytes` | `(&self, &[T]) -> OfdResult<Vec<u8>>` | 写入内存 |

### OfdReader

| Method | Returns | Description |
|:---|:---|:---|
| `open(path)` | `OfdResult<Self>` | 打开文件 |
| `from_bytes(&[u8])` | `OfdResult<Self>` | 从内存解析 |
| `page_count()` | `usize` | 页数 |
| `pages()` | `&[OfdPage]` | 结构化页面 |
| `extract_text()` | `Vec<String>` | 每页文本 |
| `extract_all_text()` | `String` | 全部文本 |

### OfdSignatureBuilder

| Method | Description |
|:---|:---|
| `new(path)` | 创建构建器 |
| `seal(ElectronicSeal)` | 添加印章 |
| `algorithm(SignatureAlgorithm)` | 签名算法 |
| `certificate(Vec<u8>)` | 设置证书 |
| `private_key(Vec<u8>)` | 设置私钥 |
| `sign() -> OfdResult<SignedOfd>` | 执行签章 |

### Content Objects 内容对象

| Type | Key Constructors | Description |
|:---|:---|:---|
| `TextObject` | `new(x,y,text) .font().size().bold().italic().color()` | 文本块 |
| `ImageObject` | `new(x,y,w,h,data,fmt) .jpeg() .png()` | 图片 |
| `PathObject` | `new(x,y,data) .hline() .vline() .rect()` | 矢量路径 |
| `OfdPage` | `new(width,height) .add_text() .add_image() .add_path()` | 页面 |

### `#[ofd(...)]` Attributes 属性

| Attribute | Type | Default | Description |
|:---|:---|:---|:---|
| `x, y` | f64/int | required | Position in mm |
| `font` | str | `"SimSun"` | Font family |
| `size` | f64 | `12.0` | Font size in pt |
| `bold` | flag | — | Bold (weight 700) |
| `weight` | u32 | `400` | Font weight |
| `italic` | flag | — | Italic |
| `color` | u32 | `0` | RGB hex (e.g. `0xFF0000`) |
| `kind` | str | `"text"` | `"text"` or `"image"` |
| `ignore` | flag | — | Skip field |
| `page_width/height` | f64/int | `210.0/297.0` | Struct-level: page size |

---

## Design Principles 设计原则

| Principle 原则 | Practice 实践 |
|:---|:---|
| **Pure Rust 纯 Rust** | `#![forbid(unsafe_code)]` in every crate |
| **Type-safe builders 类型安全构建器** | `mut self → Self`, `#[must_use]` |
| **Compile-time reflection 编译期反射** | `#[derive(OfdModel)]` — no runtime overhead |
| **Trait extensibility Trait 可扩展** | `OfdModel` trait for custom implementations |
| **Single error type 统一错误** | `OfdError` enum, `OfdResult<T>` alias |
| **Separation of concerns 关注点分离** | 9 crates, each with one job |

---

## Roadmap 路线图

| Phase 阶段 | Status | Deliverables 交付 |
|:---:|:---:|:---|
| **v0.1** | ✅ | Core types, Writer, Derive macro, Builder API |
| **v0.2** | ✅ | OFD Reader, Template engine |
| **v0.3** | ✅ | Electronic seals (GB/T 38540), Custom fonts |
| **v0.4** | ✅ | PDF ↔ OFD conversion bridge |
| **v1.0** | 🔜 | Stable API, full integration tests, performance benchmarks |

---

## License 许可证

Apache-2.0

## Related Projects 相关项目

- [easyexcel-rs](https://github.com/hiwepy/easyexcel-rs) — Rust port of Alibaba EasyExcel
- [easyexcel](https://github.com/alibaba/easyexcel) — Original Java library
- [ofd-rs](https://crates.io/crates/ofd-rs) — Lower-level OFD writer crate
- [ofd-core](https://crates.io/crates/ofd-core) — OFD XML parsing & data model

---

<p align="center">
  <sub>Built with Rust 🦀 &middot; 9 crates &middot; 177 tests &middot; Follows <a href="https://github.com/hiwepy/easyexcel-rs">easyexcel-rs</a> conventions</sub>
</p>
