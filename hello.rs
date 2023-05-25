// rustup target add wasm32-wasi
// rustc --target=wasm32-wasi -O hello.rs
fn main() {
    let id = std::env::var("id");
    println!("hello world {id:?}");
    for i in 0..8000000 {
        if i % 1000000 == 0 {
            println!("i={i}");
        }
        if i == 2000000 {
            println!("sleep");
            std::thread::sleep(std::time::Duration::from_secs(3));
            println!("wake");
        }
    }
    println!("bye {id:?}");
}
