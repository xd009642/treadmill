use treadmill::spawn;
use futures_lite::future;

fn main() {
    let t = spawn(async {
        println!("Hello world");
    });

    future::block_on(t);
}
