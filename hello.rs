// rustup target add wasm32-wasi
// rustc --target=wasm32-wasi -O hello.rs
fn main() {
    println!(
        "env(a)={:?} env(b)={:?}",
        std::env::var("a"),
        std::env::var("b")
    );
    println!("hello world");
    std::thread::sleep(std::time::Duration::from_secs(10));
    println!("bye");
}
