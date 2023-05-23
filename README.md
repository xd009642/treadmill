# Treadmill

I was fortunate enough to be able to attend Katharina Fey's Expert Async
Workshop at Rustnation 2022 as well as see their excellent talk on the
next day
[link](https://www.youtube.com/watch?v=Z-2siR9Ki84&list=PL1AoGvxomykTuOMzY5KrI4WiPCsIlYnAM&index=14).

Naturally, the next step is to dive more deeply into the part that most
interested me - writing an async runtime. So get on the treadmill because it's 
runtime.

This project is not intended for production. There are no guarantees of
performance, stability, or correctness. What it is, is a project for me to dive
deeper and learn more. And maybe some day I'll write what I learn up and take
you all along for the ride. But until then, gotta run!

## Crates

This project is split into multiple crates in the workspace here they are and
a brief description of their purpose:

* treadmill: the runtime implementation. Handles scheduling tasks and is the
core of the project
* treadmill-macros: provide convenience macros like `#[treadmill::main]` to
reduce boilerplate
* treadmill-hyper: a hyper executor so we can do web servers with treadmill!
* examples: example projects

## Roadmap

* [x] Basic `block_on` implementation
* [x] Multi-threaded run queue
* [x] Work stealing
* [x] Macro crate for standard UX
* [ ] A hyper executor
* [ ] Different implementations
* [ ] Blocking task handling
* [ ] Instrumentation and metrics
* [ ] Cool tools

## References

* [Kat's Expert Async Workshop](https://learn.spacekookie.de/rust/).
* [Making the Tokio scheduler 10x Faster](https://tokio.rs/blog/2019-10-scheduler)
