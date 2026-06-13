# supabase-micro

`supabase-micro` is a `#![no_std]` Supabase client for embedded Rust targets.
It builds PostgREST requests with fixed-capacity buffers and avoids heap
allocation, making it suitable for Cortex-M class devices.

## Features

- `no_std` API
- Fixed-capacity query construction with `heapless`
- PostgREST helpers for `select`, filters, ordering, pagination, insert,
  update, upsert, and delete
- Supabase API key and optional JWT bearer auth
- `reqwless` transport helper for async embedded HTTP clients
- JSON response deserialization through `serde-json-core`

## Usage

```rust
use supabase_micro::{Ordering, SupabaseClient};

let client = SupabaseClient::<256>::new(
    "https://your-project.supabase.co",
    "your-anon-key",
);

let mut query = client.from("users");
query.select("id,name").unwrap();
query.eq("status", "active").unwrap();
query.order("created_at", Ordering::Desc).unwrap();
query.limit(10).unwrap();

assert_eq!(
    query.path(),
    "/rest/v1/users?select=id,name&status=eq.active&order=created_at.desc&limit=10"
);
```

To execute a request, pass the configured builder to the `reqwless` transport
helper with caller-owned URL and response buffers:

```rust,ignore
use supabase_micro::transport::reqwless_transport;

let mut url_buf = [0; 512];
let mut rx_buf = [0; 2048];

let response = reqwless_transport::execute(
    &mut http,
    &query,
    &mut url_buf,
    &mut rx_buf,
)
.await?;
```

## Documentation

Build local API docs with:

```sh
cargo doc --no-deps --open
```

For CI or headless environments:

```sh
cargo doc --no-deps
```

Run tests and doctests with:

```sh
cargo test
```
