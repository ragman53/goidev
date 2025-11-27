# GOIDEV Learnings

This document captures key concepts, patterns, and learnings encountered during development. It's a reference for the "why" behind our code.

## M1: PDF Parser MVP (Initial Implementation)

### 1. Test-Driven Development (TDD) Cycle

We are following the "Red -> Green -> Refactor" cycle:

3.**Refactor (Improve the Code)**: Our next step will be to improve the implementation to get real data, knowing our test provides a safety net.

### 2. Rust `#[derive]` Attribute

We used `#[derive(Debug, Clone, PartialEq)]` on our structs (`BBox`, `TextChunk`). This is a powerful Rust feature that tells the compiler to automatically generate code for these common traits.

| Trait | Purpose | Why We Need It |
|:---|:---|:---|
| **`Debug`** | Enables printing the struct for debugging (e.g., `println!("{:?}", my_struct);`). | Essential for inspecting our data during development. |
| **`Clone`** | Allows creating deep copies of an instance via `.clone()`. | Needed so that structs containing other structs (like `TextChunk` containing `BBox`) can also be cloneable. |
| **`PartialEq`**| Allows comparing two instances for equality with `==`. | Absolutely critical for testing with `assert_eq!`. |

### 3. Inner vs. Outer Doc Comments

### 4. Handling Paths in Tests

To make tests runnable from anywhere, we build paths relative to the crate's root directory using the `env!` macro.

```rust
// Gets the path to the directory containing Cargo.toml
let mut pdf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
pdf_path.push("tests/resources/sample.pdf");
```

### 5. Generics & Trait Bounds

APIs like `std::fs::File::open` and `lopdf::Document::load` have highly flexible argument types. This is achieved through Rust's **Generics** and **Trait Bounds**.

#### a. Traits: A Type's "Capabilities"

* **What is a Trait?**: It's an interface (a contract) that defines the **"capabilities" or "behaviors"** a type has.
* **Examples**: `Debug` (the ability to be printed for debugging), `Clone` (the ability to be copied), and `AsRef<T>` (the ability to be referenced as `&T`).
* **Purpose**: If a type implements a specific Trait, operations guaranteed by that Trait (e.g., `println!`, `.clone()`, `.as_ref()`) can be performed on it through a common interface.

#### b. Generics (`<T>`) and Trait Bounds (`:`)

Generics make functions and types versatile without being tied to a specific type. Trait bounds are a mechanism to tell the compiler the **"minimum required capabilities (Traits)"** that a generic type must have.

Let's look at the signature for `lopdf::Document::load`:

```rust
// Simplified signature
pub fn load<P: AsRef<Path>>(path: P) -> Result<Document> {
    // ...
}
```

| Syntax Element | Role | Explanation |
|:---|:---|:---|
| `<P>` | Generic Type Parameter | Declares that `P` is some type to be determined by the caller. |
| `: AsRef<Path>` | Trait Bound | Imposes a constraint: `P` can be any type, but it **must** implement the `AsRef<Path>` trait. |
| `path: P` | Argument | The function accepts a value of type `P` that satisfies the trait bound. |

#### c. The `where` Clause: For Readability

When constraints become long or complex, the `where` clause improves readability. The function above can be rewritten with a `where` clause and have the exact same meaning:

```rust
pub fn load<P>(path: P) -> Result<Document>
where
    P: AsRef<Path>, // P must implement AsRef<Path>
{
    // ...
}
```

#### d. Why is `AsRef<Path>` Important?

The `AsRef<T>` trait indicates that a type can be cheaply converted into a reference `&T`.

Because the `load` function requires `AsRef<Path>`, the caller can freely pass any of the following types without needing to perform manual conversions:

* `&str` (a string slice)
* `String` (an owned string)
* `&Path` (a path slice)
* `PathBuf` (an owned path)

Inside the function, any of these types can be safely handled as a `&Path` (by calling `.as_ref()` if needed). This creates a flexible and ergonomic API that doesn't force unnecessary conversions on the user.

### 6. PDF Text Extraction: It's Not Just ASCII

A common misconception is that PDF text streams (`Tj` operators) contain ASCII or Unicode characters. In reality, they contain **glyph indices** or **character codes** that map to glyphs in a specific font.

To extract meaningful text, we must traverse a hierarchy of mapping mechanisms:

1.  **ToUnicode CMap**: The gold standard. A map embedded in the font that explicitly translates character codes to Unicode strings. If present, this should be the primary source of truth.
2.  **Encoding Dictionary**: Defines a base encoding (e.g., `WinAnsiEncoding`) and a `Differences` array. The `Differences` array maps specific codes to **Glyph Names** (e.g., `/quoteleft`, `/fi`).
3.  **Glyph Name Mapping**: If we get a glyph name like `/fi`, we need a lookup table (Adobe Glyph List) to convert it to the Unicode string `"fi"`.
4.  **Fallback**: If all else fails, we might assume `WinAnsiEncoding` or Latin-1, but this leads to "mojibake" (garbled text) for smart quotes (`0x93`, `0x94`) and ligatures.

**Key Lesson**: Hardcoding mappings for specific bytes (e.g., `0x93` -> `"`) works for one PDF but fails for others. A robust parser must implement the full lookup chain (`ToUnicode` -> `Encoding` -> `Built-in`).
