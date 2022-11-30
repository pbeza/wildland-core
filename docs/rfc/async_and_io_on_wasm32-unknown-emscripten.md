# Going async on wasm32-unknown-emscripten

## Key points

This document aims to answer these questions:

- Is it possible to use [Tokio](https://tokio.rs/) or [async_std](https://docs.rs/async-std/latest/async_std/) on wasm32-unknown-emscripten?
- Is it possible to use async/sync io considering wasm/browser limitations?
- Is it possible to implement parallelism or concurency on wasm32-unknown-emscripten?

## Impact on JS API

These are the two main use cases for this research:

- Perform async/sync IO from catlib
- Use async/await internally (not exposing it to a native user)

Performing (sync or async) IO that is hidden behind sync API will block callers thread. It is especially dangerous for web platworm because it will likely block main render thread and will make UI unresponsive.

There are 2 possible solutions to this problem:

- Expose async API
- Wrap wasm code in a [Web Worker](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers)

## Tokio on wasm32-unknown-emscripten

[Tokio](https://tokio.rs/) can be compiled and used with some limitations on wasm32-unknown-emscripten. For example it does not support async io or multithreading. List of all available features can be found [here](https://docs.rs/tokio/latest/tokio/#wasm-support).

Example usage:

<details>
  <summary>main.rs</summary>

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() {
    async {
        println!("Hello tokio!");
    }
    .await;
}
```

</details>

<details>
  <summary>Cargo.toml</summary>

```toml
...

[dependencies]
tokio = { version = "*", features = ["rt", "macros"] }
```

</details>

## async_std on wasm32-unknown-emscripten

Does not compile.

## pthread on wasm32-unknown-emscripten

It is possible to spawn threads from wasm on wasm32-unknown-emscripten.

Example usage:

<details>
  <summary>main.rs</summary>

```rust
use std::thread;

fn main() {
    thread::spawn(|| println!("Hello thread::spawn!"))
        .join()
        .unwrap();
}
```

</details>

<details>
  <summary>.cargo/config.toml</summary>

```toml
[target.wasm32-unknown-emscripten]
rustflags = [
    "-C",
    "target-feature=+atomics,+bulk-memory",
    "-C",
    "link-args=-pthread -s USE_PTHREADS=1 -s PTHREAD_POOL_SIZE=4",
]
```

</details>

<details>
  <summary>compilation</summary>

```bash
cargo +nightly build --target=wasm32-unknown-emscripten -Z build-std
```

</details>

<details>
  <summary>post compilation actions</summary>

locate {project_name}.worker.js file and place it near {project_name}.js

</details>

### Technical details and limitations

pthread support on wasm32-unknown-emscripten has a lot of limitations and incompatibilities with unix implementation:

- The whole project should be compiled with `target-feature=+atomics,+bulk-memory` including all linked libraries (and std).
- `PTHREAD_POOL_SIZE` should be carefully chosen. Creating threads on demand will require yielding control to the browser in order to create a webworker. Joining thread that is waiting for webworker to be created will deadlock.
- [SharedArrayBuffer](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer) is required for this feature to work. It has history of [being disabled for some time](https://www.mozilla.org/en-US/security/advisories/mfsa2018-01/) after meltdown/spectre discowery.
- `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` must to be set to enable SharedArrayBuffer.
- enabling pthread won't fix [tokio multithreaded scheduler](https://docs.rs/tokio/latest/tokio/runtime/index.html#multi-thread-scheduler) because it's disabled for wasm in the tokio source code.

More info about pthread support can be found [here](https://emscripten.org/docs/porting/pthreads.html)

## Websockets using Emscripten bindings

Emscripten prowides C API to interact with a browser. In order to use websocket we need to create bindings to `emscripten/emscripten.h` and `emscripten/websocket.h` headers. These headers contain functions that allow us to interact with websockets.

Example usage:

<details>
  <summary>main.rs</summary>

```rust
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::CString, ptr};

include!("path/to/emscripten/bindings");

extern "C" fn onopen(
    _eventType: i32,
    websocketEvent: *const EmscriptenWebSocketOpenEvent,
    _userData: *mut libc::c_void,
) -> i32 {
    println!("onopen");
    let msg = CString::new("Hi").unwrap();
    unsafe { emscripten_websocket_send_utf8_text((*websocketEvent).socket, msg.as_ptr()) };
    0
}

extern "C" fn onerror(
    _eventType: i32,
    _websocketEvent: *const EmscriptenWebSocketErrorEvent,
    _userData: *mut libc::c_void,
) -> i32 {
    println!("onerror");
    0
}

extern "C" fn onclose(
    _eventType: i32,
    _websocketEvent: *const EmscriptenWebSocketCloseEvent,
    _userData: *mut libc::c_void,
) -> i32 {
    println!("onclose");
    0
}

extern "C" fn onmessage(
    _eventType: i32,
    websocketEvent: *const EmscriptenWebSocketMessageEvent,
    _userData: *mut libc::c_void,
) -> i32 {
    println!("onmessage");
    let msg = CString::new("no reason").unwrap();
    unsafe { emscripten_websocket_close((*websocketEvent).socket, 1000, msg.as_ptr()) };
    0
}

fn main() {
    let url = CString::new("ws://127.0.0.1:8001").unwrap();

    let mut ws_attrs = EmscriptenWebSocketCreateAttributes {
        url: url.as_ptr(),
        protocols: ptr::null(),
        createOnMainThread: EM_TRUE as i32,
    };

    unsafe {
        let ws = emscripten_websocket_new(&mut ws_attrs);
        emscripten_websocket_set_onopen_callback_on_thread(
            ws,
            ptr::null_mut(),
            Some(onopen),
            2 as *mut __pthread,
        );
        emscripten_websocket_set_onerror_callback_on_thread(
            ws,
            ptr::null_mut(),
            Some(onerror),
            2 as *mut __pthread,
        );
        emscripten_websocket_set_onclose_callback_on_thread(
            ws,
            ptr::null_mut(),
            Some(onclose),
            2 as *mut __pthread,
        );
        emscripten_websocket_set_onmessage_callback_on_thread(
            ws,
            ptr::null_mut(),
            Some(onmessage),
            2 as *mut __pthread,
        );
    }
}
```

</details>

<details>
  <summary>.cargo/config.toml</summary>

```toml
[target.wasm32-unknown-emscripten]
rustflags = [
    "-C",
    "link-args=-lwebsocket.js",
]
```

</details>

<details>
  <summary>Cargo.toml</summary>

```toml
...

[dependencies]
libc = { version = "*" }
```

</details>

### Technical details and limitations

- Creating web socket requires yielding control to the browser.
- At this moment callbacks are only executed on the main browser thread. [github issue](https://github.com/emscripten-core/emscripten/issues/17958)
- We would need to write some glue code to create communication between sync and async code. [tokio::sync::mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html) is [able to do such thing](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html#communicating-between-sync-and-async-code).

Example glue code:

<details>
  <summary>main.rs</summary>

```rust
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::CString, ptr};

use tokio::sync::mpsc;

include!("path/to/emscripten/bindings");

extern "C" fn onopen(
    _eventType: i32,
    _websocketEvent: *const EmscriptenWebSocketOpenEvent,
    userData: *mut libc::c_void,
) -> i32 {
    println!("onopen");
    let tx: &mpsc::Sender<String> = unsafe { std::mem::transmute(userData) };
    tx.blocking_send("onopen".to_owned()).unwrap();
    0
}

fn main() {
    std::thread::spawn(|| {
        let (tx, mut rx) = mpsc::channel::<String>(100);

        let url = CString::new("ws://127.0.0.1:8001").unwrap();
        let mut ws_attrs = EmscriptenWebSocketCreateAttributes {
            url: url.as_ptr(),
            protocols: ptr::null(),
            createOnMainThread: EM_TRUE as i32,
        };

        let ws = unsafe { emscripten_websocket_new(&mut ws_attrs) };

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                unsafe {
                    emscripten_websocket_set_onopen_callback_on_thread(
                        ws,
                        std::mem::transmute(&tx),
                        Some(onopen),
                        2 as *mut __pthread,
                    );
                }

                let res = rx.recv().await;
                println!("got = {:?}", res);
            });
    });
}
```

</details>

## Fetch API using Emscripten bindings

In order to use fetch API we need to create bindings to `emscripten/emscripten.h` and `emscripten/fetch.h` headers.

Example usage:

<details>
  <summary>main.rs</summary>

```rust
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;

include!("path/to/emscripten/bindings");

extern "C" fn downloadSucceeded(fetch: *mut emscripten_fetch_t) {
    unsafe {
        println!(
            "Finished downloading {} bytes from URL {:?}",
            (*fetch).numBytes,
            (*fetch).url
        );
        emscripten_fetch_close(fetch);
    };
}

extern "C" fn downloadFailed(fetch: *mut emscripten_fetch_t) {
    unsafe {
        println!(
            "Downloading {:?} failed, HTTP failure status code: {}",
            (*fetch).url,
            (*fetch).status
        );
        emscripten_fetch_close(fetch);
    }
}

fn main() {
    let mut val = unsafe { std::mem::zeroed::<emscripten_fetch_attr_t>() };
    unsafe { emscripten_fetch_attr_init(&mut val) };
    val.requestMethod[0] = 'G' as i8;
    val.requestMethod[1] = 'E' as i8;
    val.requestMethod[2] = 'T' as i8;

    val.attributes = EMSCRIPTEN_FETCH_LOAD_TO_MEMORY;

    val.onsuccess = Some(downloadSucceeded);
    val.onerror = Some(downloadFailed);
    let url = CString::new("http://localhost:8001").unwrap();
    unsafe { emscripten_fetch(&mut val, url.as_ptr()) };
}
```

</details>

<details>
  <summary>.cargo/config.toml</summary>

```toml
[target.wasm32-unknown-emscripten]
rustflags = [
    "-C",
    "link-args=-s FETCH",
]
```

</details>

### Technical details and limitations

- To complete a fetch request, control must be yield to the browser.
- It is possible to perform a blocking fetch request with pthreads enabled.
<details>
  <summary>main.rs</summary>

```rust
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;

include!("path/to/emscripten/bindings");

fn main() {
    std::thread::spawn(move || {
        let mut val = unsafe { std::mem::zeroed::<emscripten_fetch_attr_t>() };
        unsafe { emscripten_fetch_attr_init(&mut val) };
        val.requestMethod[0] = 'G' as i8;
        val.requestMethod[1] = 'E' as i8;
        val.requestMethod[2] = 'T' as i8;

        val.attributes = EMSCRIPTEN_FETCH_LOAD_TO_MEMORY | EMSCRIPTEN_FETCH_SYNCHRONOUS;

        let url = CString::new("http://localhost:8001").unwrap();
        let result = unsafe { emscripten_fetch(&mut val, url.as_ptr()) };
        unsafe {
            println!(
                "Finished downloading {} bytes from URL {:?}",
                (*result).numBytes,
                (*result).url
            );
        }
    });
}
```

</details>

- It is likely that callbacks are executed in the main browser thread.
- Some glue code is required, as is in case with websockets.

More info can be found [here](https://emscripten.org/docs/api_reference/fetch.html)

## Other ideas that did not work

### [stdweb](https://github.com/koute/stdweb)

This project provides bindings to the Web APIs. Unfortunately this project is abandoned since 2019 and examples do not compile.

### wasm-bindgen libraries

It is not possible to use libraries that depends on wasm-bindgen with emscripten.
It will not compile or fail on runtime.

### Emulation of TCP/UDP

Emscripten will try to map tcp/udp sockets on websockets [under the hood](https://emscripten.org/docs/porting/networking.html#emulated-posix-tcp-sockets-over-websockets).
This code will connect to 127.0.0.1:8000 using websocket protocol:

<details>
  <summary>main.rs</summary>

```rust
use std::{net::TcpStream, os::unix::prelude::FromRawFd};

fn main() {
    let socket_d = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };

    let addr = libc::sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_addr: libc::in_addr {
            s_addr: 16777343_u32,
        },
        sin_port: 8000_u16.to_be(),
        sin_zero: [0, 0, 0, 0, 0, 0, 0, 0],
    };

    unsafe {
        libc::connect(
            socket_d,
            std::mem::transmute::<*const libc::sockaddr_in, *const libc::sockaddr>(&addr),
            std::mem::size_of::<libc::sockaddr_in>() as u32,
        )
    };

    let mut con = unsafe { TcpStream::from_raw_fd(socket_d) };
    loop {}
}
```

</details>

<details>
  <summary>Cargo.toml</summary>

```toml
...

[dependencies]
libc = { version = "*" }
```

</details>

Using such socket will return an error if underlying websocket in a CONNECTING state. At this moment there is no api that can check connection state or listen to a change. The lack of such API makes using such socket very complicated.
