# OBITO ENGINE - Next-Gen Hybrid C & Rust AAA Game Engine

![Language: Rust](https://img.shields.io/badge/language-Rust-orange)
![Language: C](https://img.shields.io/badge/language-C-blue)
![Graphics: wgpu (WebGPU)](https://img.shields.io/badge/graphics-wgpu-purple)

**Obito Engine** is a high-performance, next-generation AAA game engine designed with a hybrid architecture that leverages the absolute control and low overhead of **C** alongside the memory safety, concurrency, and modern ecosystem of **Rust**. By using a cross-language FFI (Foreign Function Interface) design, the engine achieves bare-metal execution speeds suitable for high-fidelity 3D rendering and simulation.

---

## 🏗️ Architectural Overview & Cross-Language FFI

The engine utilizes a split-subsystem architecture where the lightweight management and top-level execution loop are orchestrated in **C**, while the heavy lifting—such as modern cross-platform graphics rendering, math layout, and window abstractions—is built safely and concurrently in **Rust**. Functions are exposed cleanly via a custom C-ABI binding layer.

---

## 🛠️ File Mapping & Source Breakdown

As shown in the repository root (referencing the current structure), here is the breakdown of how the engine's core components interface with one another:

### 🚀 Core Application & Entry Point
*   `main.c`: The primary entry point of the application. It orchestrates the engine’s main execution loop, handles OS signaling, and directly invokes the underlying high-performance Rust subsystems via FFI.

### 🔗 Cross-Language Interoperability Layer
*   `lib.rs`: The bridge file for the engine. It exposes the internal Rust functions, safe abstractions, and rendering pipelines to the C runtime environment using `#[no_mangle]` and `extern "C"` declarations, compiling into a static/dynamic library that `main.c` links against.

### 🖥️ Windowing Subsystem
*   `Obito_Window.rs`: Dedicated windowing module built using the **winit** library. It delivers native, high-performance window creation, hardware input event polling, and multi-threaded event handling across platforms.

### 🎨 AAA Graphics Pipeline
*   `Graphics_AAA.rs`: The heart of the engine's renderer. Built upon the modern, low-level **wgpu** graphics API and backed by **glam** for SIMD-accelerated linear algebra, this module manages rendering pipelines, device queues, bind groups, and high-fidelity 3D asset draw calls.
*   `Engine.wgsl`: The core WebGPU Shading Language (WGSL) file containing optimized vertex and fragment shaders for the AAA rendering pipeline.

### 📦 Build & Package Configuration
*   `Cargo.toml`: The manifest file defining engine crates, dependencies (wgpu, winit, glam), profile optimizations, and library target outputs (e.g., `staticlib` or `cdylib`).
*   `Cargo.lock`: Automatically generated lockfile ensuring exact, reproducible dependency trees for consistent engine builds across testing environments.
