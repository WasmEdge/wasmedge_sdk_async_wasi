// rustup target add wasm32-wasi
// rustc --target=wasm32-wasi -O hello.rs
fn main() {
    let id = std::env::var("id");
    println!("hello world {id:?}");
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("bye {id:?}");
}
