## Reading from TcpStream

In this server we make use of `[0u8; 1024]` as the buffer capacity but that can easily be extended
You can use [BufRead](https://doc.rust-lang.org/std/io/trait.BufRead.html) or you can rely on the following code : 

```rust
use std::io::{Read, Result};

// ... other code ...

let mut buffer = Vec::new();

loop {
    // Read into a temporary buffer of 1024 bytes
    let mut temp_buffer = [0_u8; 1024];
    let bytes_read = stream.read(&mut temp_buffer)?;

    // Append the read bytes to the main buffer
    buffer.extend_from_slice(&temp_buffer[..bytes_read]);

    // Break if we've reached the end of the stream
    if bytes_read == 0 {
        break;
    }
}

trace!("{bytes_read} bytes read: {:?}", &buffer);
```