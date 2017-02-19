# Futures vs Threads benchmark

Compare asynchronous tokio + futures with blocking `std::net` + `std::thread`.

The benchmark is a multiecho server: client connects to the server, sends
"Hello, World!" and the server broadcasts the message to all previously
connected clients. Thus, `n` clients will send and receive `n^2` messages
in total.

* `src/threads.rs` is implementation which spawns a thread per each client.
* `src/fut.rs` is a single threaded future based implementation.
* `src/bench.rs` simulated 3000 clients.
