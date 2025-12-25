use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

const SHORT_MARKDOWN: &str = r#"# Hello World

This is a **simple** paragraph with some *emphasis*.

- Item 1
- Item 2
- Item 3
"#;

const MEDIUM_MARKDOWN: &str = r#"# Project Documentation

## Overview

This is a **comprehensive** guide to the project. It includes various markdown features
that are commonly used in documentation.

### Features

1. Type-safe templates
2. Fast rendering
3. Easy to use API

### Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```

## Installation

Run the following command:

```bash
cargo install myproject
```

## Links

- [GitHub](https://github.com)
- [Documentation](https://docs.rs)

> This is a blockquote with some important information.

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| A        | B        | C        |
| D        | E        | F        |
"#;

const LONG_MARKDOWN: &str = r#"# Comprehensive Markdown Test Document

## Introduction

This document contains a variety of markdown elements to test parsing performance.
We include multiple sections, code blocks, lists, and other common elements.

### Purpose

The purpose of this benchmark is to compare:

1. **pulldown-cmark** - A fast, compliant CommonMark parser
2. **markdown** crate - Alternative markdown parser

## Features Comparison

### Text Formatting

This paragraph contains **bold text**, *italic text*, and ***bold italic***.
We also have `inline code` and ~~strikethrough~~ text.

### Lists

#### Unordered Lists

- First item
  - Nested item 1
  - Nested item 2
    - Deeply nested
- Second item
- Third item

#### Ordered Lists

1. Step one
2. Step two
   1. Sub-step A
   2. Sub-step B
3. Step three

### Code Blocks

Here's a Rust example:

```rust
use std::collections::HashMap;

fn main() {
    let mut map: HashMap<String, i32> = HashMap::new();
    map.insert("key".to_string(), 42);

    for (key, value) in &map {
        println!("{}: {}", key, value);
    }
}
```

And a Python example:

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

for i in range(10):
    print(fibonacci(i))
```

### Tables

| Feature | pulldown-cmark | markdown |
|---------|----------------|----------|
| Speed | Fast | TBD |
| CommonMark | Yes | Yes |
| GFM | Partial | TBD |
| Size | Small | TBD |

### Blockquotes

> This is a blockquote.
> It can span multiple lines.
>
> > And can be nested too.

### Links and Images

- [External Link](https://example.com)
- [Reference Link][ref]
- ![Alt text](image.png)

[ref]: https://example.com/reference

### Horizontal Rules

---

***

___

## Conclusion

This benchmark helps us make an informed decision about which markdown
parser to use in our project. Performance matters for user experience.

### Summary

The results will show:

1. Parsing speed (operations per second)
2. Memory efficiency
3. Feature completeness

---

*Document ends here.*
"#;

fn bench_pulldown_cmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("pulldown-cmark");

    group.throughput(Throughput::Bytes(SHORT_MARKDOWN.len() as u64));
    group.bench_function("short", |b| {
        b.iter(|| {
            let parser = pulldown_cmark::Parser::new(black_box(SHORT_MARKDOWN));
            let mut html = String::new();
            pulldown_cmark::html::push_html(&mut html, parser);
            html
        })
    });

    group.throughput(Throughput::Bytes(MEDIUM_MARKDOWN.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| {
            let parser = pulldown_cmark::Parser::new(black_box(MEDIUM_MARKDOWN));
            let mut html = String::new();
            pulldown_cmark::html::push_html(&mut html, parser);
            html
        })
    });

    group.throughput(Throughput::Bytes(LONG_MARKDOWN.len() as u64));
    group.bench_function("long", |b| {
        b.iter(|| {
            let parser = pulldown_cmark::Parser::new(black_box(LONG_MARKDOWN));
            let mut html = String::new();
            pulldown_cmark::html::push_html(&mut html, parser);
            html
        })
    });

    group.finish();
}

fn bench_markdown_crate(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown-crate");

    group.throughput(Throughput::Bytes(SHORT_MARKDOWN.len() as u64));
    group.bench_function("short", |b| {
        b.iter(|| markdown::to_html(black_box(SHORT_MARKDOWN)))
    });

    group.throughput(Throughput::Bytes(MEDIUM_MARKDOWN.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| markdown::to_html(black_box(MEDIUM_MARKDOWN)))
    });

    group.throughput(Throughput::Bytes(LONG_MARKDOWN.len() as u64));
    group.bench_function("long", |b| {
        b.iter(|| markdown::to_html(black_box(LONG_MARKDOWN)))
    });

    group.finish();
}

criterion_group!(benches, bench_pulldown_cmark, bench_markdown_crate);
criterion_main!(benches);
