use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use treadmill::Runtime;

fn main() {
    setup_logging();

    let rt = Runtime::default();
    rt.block_on(async {
        let (tx, rx) = async_channel::bounded(10);

        for i in 0..10 {
            let sender = tx.clone();
            treadmill::spawn(async move {
                for j in 0..10 {
                    sender.send(i * 10 + j).await.unwrap();
                }
            })
            .detach();
        }
        let mut count = 0;
        std::mem::drop(tx);
        while let Ok(res) = rx.recv().await {
            info!("Received {}", res);
            count += 1;
        }
        info!("Received {} messages", count);
    });
}

fn setup_logging() {
    let env_filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("treadmill=trace,channels=info"),
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();
}
