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

## Roadmap

* [] Basic `block_on` implementation
* [] Multi-threaded run queue
* [] Work stealing
* [] Different implementations
* [] Blocking task handling
* [] Instrumentation and metrics
* [] Cool tools

## References

* [Kat's Expert Async Workshop](https://www.youtube.com/watch?v=Z-2siR9Ki84&list=PL1AoGvxomykTuOMzY5KrI4WiPCsIlYnAM&index=14).
* [Making the Tokio scheduler 10x Faster](https://tokio.rs/blog/2019-10-scheduler)
