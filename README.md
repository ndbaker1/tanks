# Tanks

🎮 A Realtime Multiplayer Server/Client Game example built entirely with Rust 🦀

✨Just 2 lines of JavaScript needed for bootstrapping WASM in the [index file](./tanks_wasm/index.html)✨

## Background

So I wanted to make a web game completely in Rust but could only find small bits of documentation in different articles/posts.

Therefore, I decided to make this as a general example for myself (and practice my Rust) regarding how to implement several critical Rust WASM features, such as implementing WebSockets and drawing to a browser Canvas exclusively from Rust APIs.

## Libraries 📚

The server is a [`Warp`](https://github.com/seanmonstar/warp) server setup to handle WebSockets.
<br>
I wrote [this websocket server](https://github.com/ndbaker1/websocket-server) wrapper that accepts custom `MessageEventHandler` and `ServerTickHandler`, that way I can reuse it for multiple browser games.

## Building 🔨

`pack-wasm` lib is my hacky way of creating a custom build script for packing wasm and including an index page file which I find slightly annoying using `wasm-pack` because you cant specify the `.wasm` target like with `wasm-bindgen-cli`
```sh
cargo install wasm-bindgen-cli
cargo run --bin pack-wasm
```

Start the server binary from the root directory and it will server everything in the `dist` folder
```sh
cargo run --bin tanks_server
```
