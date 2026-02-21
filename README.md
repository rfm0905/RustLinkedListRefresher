# Linked Lists Tutorial 
- Just me following along with [https://rust-unofficial.github.io/too-many-lists](https://rust-unofficial.github.io/too-many-lists) for a refresher. First part in a series of tutorials i'm doing for myself, following up with [Rust Atomics and Locks](https://mara.nl/atomics/), [the nomicon](https://doc.rust-lang.org/nomicon/) and light reading of *CSAPP* 
- Not rlly useful to anyone else but myself, though maybe my comments might be better at explaining then the book. 

Contains: 
1. `bad_stack.rs` A very basic stack 
2. `ok_stack.rs` A normal stack that upgrades the previous one with `std::option`, and iterators 
3. `peristent_stack.rs` An FP Style stack which is immutable. Use reference counted `std::Rc`, though I modified for thread-safety with `std::Arc` 
4. `bad_safe_deque.rs` A deque that uses no unsafe code, but via `std::RefCell` has interior mutability. 
5. `ok_unsafe_queue.rs` A queue that finally gets into unsafe pointers and unsafe rust. 
6. `linkedlist.rs`, one-to-one equivalent of `std::LinkedList`, but also with cursors. Kinda got lazy copy pasting on this one since wasn't really learning anything new
