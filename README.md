# Conrod Chat
This is a basic sample crate for creating a chat client with Rust using Conrod with several backend options. Conrod allows different graphical backend like "glium", "winit" or even "SDL2". This crate allows different socket backend options. At the moment, only the websocket crate.

# Hot code reloading
This is a feature that is only available on desktop. It uses the crate "libloading" to reload "test_shared.rs" code during runtime. It helps programmers to adjust components of the gui on the fly.

To get started.
1. You need to compile test_shared.rs into a crate.
```
rustc src/test_shared.rs --crate-name test_shared --crate-type dylib --out-dir target/debug
```
2. Run cargo with --feature="hotload"
3. During runtime, edit `src/test_shared.rs` and run `rustc src/test_shared.rs --crate-name test_shared --crate-type dylib --out-dir target/debug` to see hot reloading in action.

# Without Hot code reloading

The Chat's components' positions will be taken from `staticapplication.rs` instead of `dyapplication.rs`. Run cargo without the hotload feature.

# Running examples

Currently, there are only one example which uses websocket backend. The default features in cargo have already the correct feature implementations so you don't need to specify.

```
cargo run --example websocket_glium
```



