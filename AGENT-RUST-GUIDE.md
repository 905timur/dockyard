# Rust Agent Guide

Quick reference for AI coding agents working with Rust.
Focus on common patterns, anti-patterns, and critical rules.


## Common Programming Concepts

### Key Rules

- You should receive an error message
regarding an immutability error, as shown in this output:

```console
{{#include
- You received the error message ` cannot assign twice to immutable variable x  because you tried to assign a second value to the immutable x` variable
- It’s important that we get compile-time errors when we attempt to change a
value that’s designated as immutable, because this very situation can lead to
bugs
- If one part of our code operates on the assumption that a value will
never change and another part of our code changes that value, it’s possible
that the first part of the code won’t do what it was designed to do
- Constants aren’t just
immutable by default—they’re always immutable
- Just know that you must always annotate the type
- txt}}
``

Shadowing is different from marking a variable as mut because we’ll get a
compile-time error if we accidentally try to reassign to this variable without
using the let` keyword
- However, if we try to use mut for this, as shown
here, we’ll get a compile-time error:

```rust,ignore,does_not_compile
{{#rustdoc_include

### Examples

```rust
{{#rustdoc_include ../listings/ch03-common-programming-concepts/no-listing-02-adding-mut/src/main.rs}}
```

```rust
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
```

```rust
{{#rustdoc_include ../listings/ch03-common-programming-concepts/no-listing-03-shadowing/src/main.rs}}
```


## Ownership (CRITICAL)

### Key Rules

- Some languages have garbage collection that regularly looks for no-longer-used
memory as the program runs; in other languages, the programmer must explicitly
allocate and free the memory
- > ### The Stack and the Heap
>
> Many programming languages don’t require you to think about the stack and the
> heap very often
- All
> data stored on the stack must have a known, fixed size
- Data with an unknown
> size at compile time or a size that might change must be stored on the heap
> instead
- Because the pointer to the heap is a
> known, fixed size, you can store the pointer on the stack, but when you want
> the actual data, you must follow the pointer
- >
> Pushing to the stack is faster than allocating on the heap because the
> allocator never has to search for a place to store new data; that location is
> always at the top of the stack
- Comparatively, allocating space on the heap
> requires more work because the allocator must first find a big enough space
> to hold the data and then perform bookkeeping to prepare for the next
> allocation
- rs:here}}
``

</Listing>

In other words, there are two important points in time here:

- When s` comes _into_ scope, it is valid

### Examples

```rust
let s = "hello";
```

```rust
{{#rustdoc_include ../listings/ch04-understanding-ownership/listing-04-01/src/main.rs:here}}
```

```rust
let s = String::from("hello");
```


## Enums and Pattern Matching

### Key Rules

- Both version four and version six addresses are still fundamentally IP
addresses, so they should be treated as the same type when the code is handling
situations that apply to any kind of IP address
- rs:instance}}
```

Note that the variants of the enum are namespaced under its identifier, and we
use a double colon to separate the two
- Version four IP
addresses will always have four numeric components that will have values
between 0 and 255
- Note that even though the standard library contains a definition for IpAddr,
we can still create and use our own definition without conflict because we
haven’t brought the standard library’s definition into our scope
- Expressing this concept in terms of the type system means the compiler can
check whether you’ve handled all the cases you should be handling; this
functionality can prevent bugs that are extremely common in other programming
languages
- Programming language design is often thought of in terms of which features you
include, but the features you exclude are important too
- In languages with null, variables can always be in one of
two states: null or not-null
- My
> goal was to ensure that all use of references should be absolutely safe, with
> checking performed automatically by the compiler

### Examples

```rust
{{#rustdoc_include ../listings/ch06-enums-and-pattern-matching/no-listing-01-defining-enums/src/main.rs:def}}
```

```rust
{{#rustdoc_include ../listings/ch06-enums-and-pattern-matching/no-listing-01-defining-enums/src/main.rs:instance}}
```

```rust
{{#rustdoc_include ../listings/ch06-enums-and-pattern-matching/no-listing-01-defining-enums/src/main.rs:fn}}
```


## Collections

### Key Rules

- rs:here}}
```

</Listing>

Note that we added a type annotation here
- This is an important point
- rs:here}}
```

</Listing>

Note a few details here
- rs:here}}
``

</Listing>

When we run this code, the first []` method will cause the program to panic
because it references a nonexistent element
- rs:here}}
``

</Listing>

Compiling this code will result in this error:

``console
{{#include
- txt}}
```

The code in Listing 8-6 might look like it should work: Why should a reference
to the first element care about changes at the end of the vector
- This error is
due to the way vectors work: Because vectors put the values next to each other
in memory, adding a new element onto the end of the vector might require
allocating new memory and copying the old elements to the new space, if there
isn’t enough room to put all the elements next to each other where the vector
is currently stored
- > Note: For more on the implementation details of the Vec<T> type, see [“The
> Rustonomicon”][nomicon]

### Examples

```rust
{{#rustdoc_include ../listings/ch08-common-collections/listing-08-01/src/main.rs:here}}
```

```rust
{{#rustdoc_include ../listings/ch08-common-collections/listing-08-02/src/main.rs:here}}
```

```rust
{{#rustdoc_include ../listings/ch08-common-collections/listing-08-03/src/main.rs:here}}
```


## Error Handling (CRITICAL)

### Key Rules

- ## Unrecoverable Errors with `panic
- In these cases, Rust has the `panic
- There are two ways to cause a
panic in practice: by taking an action that causes our code to panic (such as
accessing an array past the end) or by explicitly calling the `panic
- In both cases, we cause a panic in our program
- Via an
environment variable, you can also have Rust display the call stack when a
panic occurs to make it easier to track down the source of the panic
- > ### Unwinding the Stack or Aborting in Response to a Panic
>
> By default, when a panic occurs, the program starts _unwinding_, which means
> Rust walks back up the stack and cleans up the data from each function it
> encounters
- If in your project you need to make the resultant binary as
> small as possible, you can switch from unwinding to aborting upon a panic by
> adding panic = 'abort' to the appropriate [profile] sections in your
> _Cargo
- For example, if you want to abort on panic in release mode,
> add this:
>
> ```toml
> [profile

### Examples

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

```rust
{{#rustdoc_include ../listings/ch09-error-handling/listing-09-03/src/main.rs}}
```

```rust
{{#include ../listings/ch09-error-handling/listing-09-06/src/main.rs:here}}
```


## Generics, Traits, Lifetimes (CRITICAL)

### Key Rules

- Note that this code won’t
compile yet
- rs}}
``

</Listing>

If we compile this code right now, we’ll get this error:

``console
{{#include
- For now, know that this error
states that the body of largest won’t work for all possible types that T
could be
- Note that because we’ve used only one generic type to define Point<T>, this
definition says that the Point<T> struct is generic over some type T, and
the fields x and y are _both_ that same type, whatever that type may be
- rs" caption="The fields x and y must be the same type because both have the same generic data type T
- 0 for y, which we’ve defined to have
the same type as x, we’ll get a type mismatch error like this:

``console
{{#include
- Let’s take another look at the Option<T> enum that the standard
library provides, which we used in Chapter 6:

``rust
enum Option<T> {
    Some(T),
    None,
}
``

This definition should now make more sense to you
- This definition makes it convenient to use the Result enum anywhere we
have an operation that might succeed (return a value of some type T) or fail
(return an error of some type E)

### Examples

```rust
{{#rustdoc_include ../listings/ch10-generic-types-traits-and-lifetimes/listing-10-04/src/main.rs:here}}
```

```rust
{{#rustdoc_include ../listings/ch10-generic-types-traits-and-lifetimes/listing-10-06/src/main.rs}}
```

```rust
{{#rustdoc_include ../listings/ch10-generic-types-traits-and-lifetimes/listing-10-08/src/main.rs}}
```


## Iterators and Closures

### Key Rules

- Closures don’t
usually require you to annotate the types of the parameters or the return value
like fn functions do
- Type annotations are required on functions because the
types are part of an explicit interface exposed to your users
- Defining this
interface rigidly is important for ensuring that everyone agrees on what types
of values a function uses and returns
- The
add_one_v3 and add_one_v4 lines require the closures to be evaluated to be
able to compile because the types will be inferred from their usage
- Note that we haven’t added any type annotations to the definition
- If we then try to call
example_closure with an integer, we’ll get an error
- rs:here}}
``

</Listing>

The compiler gives us this error:

``console
{{#include
- Those
types are then locked into the closure in example_closure, and we get a type
error when we next try to use a different type with the same closure

### Examples

```rust
{{#rustdoc_include ../listings/ch13-functional-features/listing-13-02/src/main.rs:here}}
```

```rust
{{#rustdoc_include ../listings/ch13-functional-features/listing-13-04/src/main.rs}}
```

```rust
{{#rustdoc_include ../listings/ch13-functional-features/listing-13-05/src/main.rs}}
```


## Smart Pointers

### Key Rules

- You’ll use them most often in these situations:

- When you have a type whose size can’t be known at compile time, and you want
  to use a value of that type in a context that requires an exact size
- When you have a large amount of data, and you want to transfer ownership but
  ensure that the data won’t be copied when you do so
- When you want to own a value, and you care only that it’s a type that
  implements a particular trait rather than being of a specific type

We’ll demonstrate the first situation in “Enabling Recursive Types with
Boxes”<
- Note that this is not the same as the “null” or “nil” concept discussed
in Chapter 6, which is an invalid or absent value
- Note that this code
won’t compile yet, because the List type doesn’t have a known size, which
we’ll demonstrate
- rs:here}}
``

</Listing>

> Note: We’re implementing a cons list that holds only i32` values for the
> purposes of this example
- If we try to compile the code in Listing 15-3, we get the error shown in
Listing 15-4
- <Listing number="15-4" caption="The error we get when attempting to define a recursive enum">

```console
{{#include
- txt}}
```

</Listing>

The error shows this type “has infinite size
- Let’s break down why we get this error

### Examples

```rust
{{#rustdoc_include ../listings/ch15-smart-pointers/listing-15-01/src/main.rs}}
```

```rust
{{#rustdoc_include ../listings/ch06-enums-and-pattern-matching/listing-06-02/src/main.rs:here}}
```

```rust
{{#rustdoc_include ../listings/ch15-smart-pointers/listing-15-05/src/main.rs}}
```


## Async/Await

### Key Rules

- (In fact, Rust will show a compiler warning if you don’t use a future
- This laziness
allows Rust to avoid running async code until it’s actually needed
- > Note: This is different from the behavior we saw when using thread::spawn
> in the [“Creating a New Thread with spawn”][thread-spawn]<
- But it’s important for Rust to be able to provide its
> performance guarantees, just as it is with iterators
- To the compiler, a function definition such as the
async fn page_title in Listing 17-1 is roughly equivalent to a non-async
function defined like this:

```rust
# extern crate trpl; // required for mdbook test
use std::future::Future;
use trpl::Html;

fn page_title(url: &str) -> impl Future<Output = Option<String>> {
    async move {
        let text = trpl::get(url)
- -- manual-regeneration
cd listings/ch17-async-await/listing-17-03
cargo build
copy just the compiler error
-->

``text
error[E0752]: main function is not allowed to be async`
 --> src/main
- rs:enum}}
```

Writing the code to transition between each state by hand would be tedious and
error-prone, however, especially when you need to add more functionality and
more states to the code later
- The
normal borrowing and ownership rules around data structures all still apply,
and happily, the compiler also handles checking those for us and provides
useful error messages

### Examples

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-01/src/main.rs:all}}
```

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-02/src/main.rs:chaining}}
```

```rust
# extern crate trpl; // required for mdbook test
use std::future::Future;
use trpl::Html;

fn page_title(url: &str) -> impl Future<Output = Option<String>> {
    async move {
        let text = trpl::get(url).await.text().await;
        Html::parse(&text)
            .select_first("title")
            .map(|title| title.inner_html())
    }
}
```


## Common Anti-Patterns to Avoid

- Using `.unwrap()` in production code (use proper error handling)
- Excessive `.clone()` without understanding ownership
- Ignoring compiler warnings
- Using `panic!` for recoverable errors
- Not using iterators (prefer `.iter()` over manual loops)
- Ignoring lifetime annotations when needed
