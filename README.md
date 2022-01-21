# Tanks

🎮 A Realtime Multiplayer Server/Client Game example built entirely with Rust 🦀

✨Zero lines of JavaScript besides [html file for bootstrapping wasm](./tanks-worker/index.html)✨

## Background

So I wanted to make a game completely in Rust but could only find small bits of documentation in different articles/posts.

Therefore I decided to make this as a general example for myself (and practice my Rust) regarding how to impliment these critical Rust WASM features, such as implementing WebSockets and drawing to a browser Canvas completely from Rust APIs.

## Libraries

The Server is a [`Warp`](https://github.com/seanmonstar/warp) server setup handles WebSockets.
<br>
I actually wrote my own WebSocker-server wrapper so that accepts a custom `MessageHandler` and `ServerTickHandler`, that way I can reuse it for browser games.


