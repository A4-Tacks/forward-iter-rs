Ensure that side effects in iterators always run forward

# Examples

**Fail case**:
```rust,should_panic
let mut i = 0;
let x: Vec<_> = (0..4).map(|_| { i += 1; i }).rev().collect();
assert_eq!(x, [4, 3, 2, 1]); // fail!
```

**Rewrite to**:

```rust
use forward_iter::ForwardIterExt as _;
let mut i = 0;
let x: Vec<_> = (0..4).map(|_| { i += 1; i }).forward().rev().collect();
assert_eq!(x, [4, 3, 2, 1]); // success!
```
