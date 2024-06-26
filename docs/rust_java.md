# Rust backend

The Rust backend has been created to be usable by other UI languages than Java through the C FFI layer of Rust.

The backend itself exports the C header file [popcorn-fx.hpp](../include/popcorn-fx.hpp), _generated by cbingen_, to provide the necessary
information to integrate your own UI with the Popcorn FX backend system.

Creating a new backend instance can be done as follows:
```rust
fn main() {
    let popcorn_fx = new_popcorn_fx();
}
```

## Rust/Java interop

Rust and Java communicate through the C FFI layer of Rust and JNA library of Java.
Each Rust package creates a C ABI library which contains wrappers for the Rust structs 
and functions for exposing the Rust struct functions.
These wrappers are then used by Java JNA to handle the functionality provided within the application UI.

### Rust FFI

All information and functionality is exported as C function through the [application/src/lib.rs](../application/src/lib.rs) library.
This library is exported to be C compatible. When a Rust struct contains fields which are not ABI compatible, 
a C compatible wrapper is foreseen within the Rust library.

#### ABI compatibility

The Rust `String` type is not ABI compatible and therefore cannot be directly returned
through the FFI layer to Java.
Each of these `String` fields should therefore be translated to a `c_char` array.

_Example:_
```rust
fn example() -> *mut c_char{
    let c_compatible_text: *mut c_char;
    let text = String::from("lorem ipsum dolor");

    c_compatible_text = CString::new(text).unwrap().into_raw();
    
    return c_compatible_text;
}
```

#### Memory management

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