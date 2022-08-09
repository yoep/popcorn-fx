# Rust & Java interop

Rust and Java communicate through the C FFI layer of Rust and JNA library of Java.
Each Rust package creates a C ABI library which contains wrappers for the Rust structs 
and functions for exposing the Rust struct functions.
These wrappers are then used by Java JNA to handle the functionality provided within the application UI.

## Rust FFI

Each platform contains a `lib.rc` which expose the rust functionality over ABI.
When a Rust struct contains fields which are not ABI compatible, a wrapper is foreseen/should be foreseen.

### ABI compatibility

The Rust `String` type is not ABI compatible and therefore cannot be directly returned
through the FFI layer to Java.
Each of these `String` fields should therefore be translated to a `c_char` array.

_Example:_
```rust
fn example() -> *const c_char{
    let c_compatible_text: *const c_char;
    let text = String::from("lorem ipsum dolor");

    c_compatible_text = CString::new(text).unwrap().into_raw();
    
    return c_compatible_text;
}
```

### Memory management

As Rust by default cleans memory through scope or lifespan indicators, 
most of the returned structs would be cleaned by Rust directly after the FFI function has been called.

Therefore most of the functions return the struct as a Reference in a `Box`.
This means that the caller will become responsible for managing the memory.

_Return an owned reference to the caller_
```rust
/// Create a new Foo instance for which the caller will become responsible.
/// The Box returns a reference to the struct on the heap and not the actual struct.
#[no_mangle]
pub extern "C" fn new_instance() -> Box<Foo> {
    Box::new(Foo::new())
}
```
```java
/**
 * This means that Java needs to interpret the return value as a pointer and not an actual struct.
 */
public class Foo extends PointerType {
}

interface Example extends Library {
    Foo new_instance();
}
```

The ownership of the `struct`'s memory can be returned to Rust by passing it as a Box
within one of the FFI functions.
Never do this when you still want to use the struct later on within the application.

```rust
#[no_mangle]
pub extern "C" fn example(instance: Box<Foo>) {
    // when this function ends, the instance will be deleted by Rust
    // as it's now owned by Rust and no scope/lifespan manages it anymore afterwards
}
```