# Tanks

🎮 A Realtime Multiplayer Server/Client Game example built entirely with Rust 🦀

✨Zero lines of JavaScript besides [html file for bootstrapping wasm](./tanks-worker/index.html)✨

## Background

So I wanted to make a game completely in Rust but could only find small bits of documentation in different articles/posts.

Therefore, I decided to make this as a general example for myself (and practice my Rust) regarding how to implement several critical Rust WASM features, such as implementing WebSockets and drawing to a browser Canvas exclusively from Rust APIs.

## Libraries

The server is a [`Warp`](https://github.com/seanmonstar/warp) server setup to handle WebSockets.
<br>
I wrote [this websocket server](https://github.com/ndbaker1/websocket-server) wrapper that accepts custom `MessageEventHandler` and `ServerTickHandler`, that way I can reuse it for multiple browser games.
