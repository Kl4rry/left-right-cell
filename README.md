# left-right-cell
left-right-cell is a lockfree, eventually consistent cell created using the `left-right` crate.  
It allows readers to read from the cell without ever blocking while the writer might block when writing.  
This is achived by storing to copies of the data one for the readers and one for the writer.

```rust
let (mut w, r) = left_right_cell::new(false);

let t = std::thread::spawn(move || {
    loop {
        let value = r.get().unwrap();
        if *value {
            break;
        }
    }
});

w.set(true);
w.publish();
t.join().unwrap();
assert!(true);
```