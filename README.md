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
To run front-end: `npm run start` from the `www` directory.
To build front-end: 
 - for the first time: run `wasm-pack build`
 - `npm run build` - prod, `npm run build-dev` - dev
To build web assembly: `wasm-pack build`
To build backend: `cargo build`
