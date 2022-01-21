use std::fs;

fn main() {
    let index_path = "../index.html";

    let wasm_package = "../tanks-worker/pkg/tanks_worker.js";
    let websocket_uri = "ws://localhost:8000/api/ws";

    fs::write(
        index_path,
        format!(
            r#"
<html>
<style>
  body {{ margin: 0; }}
</style>
<body>
  <script async type="module">
    import init, {{ connect }} from '{}'
    await init()
    connect('{}')
  </script>
</body>

</html>
            "#,
            wasm_package, websocket_uri,
        ),
    )
    .unwrap();
}
