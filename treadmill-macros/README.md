# Treadmill Macros

Most async runtimes offer their own macros to initialise the runtime and block
on a future. This means for the rest of us we merely have to do something like:

```rust
#[treadmill::main]
async fn main() {

}
```

Instead of:

```rust
fn main() {
    treadmill::Runtime::new()
        .block_on(async {

        })
}
```

This planned crate will implement the proc-macro magic to make this kind
of feature possible explaining it step by step.
