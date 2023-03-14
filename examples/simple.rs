use futures_lite::future;
use treadmill::spawn;

fn main() {
    let t = spawn(async {
        println!("Hello world");
    });

    future::block_on(t);
}
