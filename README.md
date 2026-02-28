# Linked Lists Tutorial 
- Just me following along with [https://rust-unofficial.github.io/too-many-lists](https://rust-unofficial.github.io/too-many-lists) for a refresher. First part in a series of tutorials I'm doing for myself, following up with [Atomics and Locks - Mara Box](https://github.com/rfm0905/concurrencyPrimitivesRust), [the Nomicon + async book](https://github.com/rfm0905/asyncAndRustNomicon) and *CSAPP* labs. 
- Not rlly useful to anyone else but me, though maybe my comments might be better at explaining then the book. 
- Tests (and comments on tests) are directly copied from the book. 

Contains: 
1. [`bad_stack.rs`](src/bad_stack.rs) A very basic stack 
2. [`ok_stack.rs`](src/ok_stack.rs) A normal stack that upgrades the previous one with `std::option`, and iterators 
3. [`peristent_stack.rs`](src/persistent_stack.rs) An FP-style stack which is immutable. Uses reference-counted `std::Rc`, though I modified for thread-safety with `std::Arc` 
4. [`bad_safe_deque.rs`](src/bad_safe_deque.rs) A deque that uses no unsafe code, but via `std::RefCell` has interior mutability. 
5. [`ok_unsafe_queue.rs`](src/ok_unsafe_queue.rs) A queue that finally gets into unsafe pointers and `unsafe` Rust. 
6. [`linkedlist.rs`](src/linkedlist.rs), One-to-one equivalent of `std::LinkedList`, but also with cursors (and iterators). Kinda got lazy copy pasting on this one since I wasn't really learning anything new
