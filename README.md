<div align="center">
  <h1><code>breakout</code></h1>
  <sub>Hosted at <a href="http://rusty-games.eu">rusty-games.eu</a></sub>
</div>

## About

An implementation of the variation of breakout game in rust using webassembly.

## Usage

Go to the webpage and play.

## How to build.

### Required tools:
The Rust toolchain `rustup`, `rustc`, `cargo`. A tool to generate WebAssembly: `wasm-pack`. A package manager for JavaScript `npm`.

### Build commands:
- Front-end:
  - start locally: `npm run start` from the `www` directory
  - build release version: `npm run build` from the `www` directory
  - build development version: `npm run build-dev` from the `www` directory
  - remove build files: `npm run clean` from the `www` directory
- Web assembly:
  - `wasm-pack build` from the main directory
- Back-end: `cargo build` from the `api` directory
