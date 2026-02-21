use std::mem;

struct Node {
    elem: i32,
    next: Link,
}

enum Link {
    Empty,
    More(Box<Node>), // null ptr optimized; `enum Link` only takes size of `Box<Node>` (empty state is just all zeros)
}

pub struct List {
    head: Link,
}

impl List {
    // static method
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    // normal method
    pub fn push(&mut self, elem: i32) {
        let newhead = Box::new(Node {
            elem,
            /*
             * using `mem::replace` since setting `newhead.next` to `list.head` (the old head) directly would take ownership of the original list's head, making the original list unusable.
             * `mem::replace` instead moves something (`src: Link::Empty`) into `dest: self.head.` So we're replacing the old_head with an empty node.
             * Then since it returns what was in `dest` originally, we can set `newhead.next` to that. So we're creating the `newhead` with next as the old head
             * This doesn't drop `src` or `dest`, so everything is left valid
             */
            next: mem::replace(&mut self.head, Link::Empty),
        });
        self.head = Link::More(newhead);
    }

    pub fn pop(&mut self) -> Option<i32> {
        /*
        // note: pattern matching must be exhaustive
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(oldhead) => {
                self.head = oldhead.next;
                Some(oldhead.elem)
            }
        }
        */

        /*
         * if/else is an expression, so both branches need to evaluate to same type
         * In this case, since if {...} evaluates to Option<i32>, so must else {...}; we can't just leave the else branch blank

            // if let: if we match the pattern, set var to pattern and do {} with the var; otherwise do what's in the else {}
            if let Link::More(oldhead) = mem::replace(&mut self.head, Link::Empty) {
                self.head = oldhead.next;
                Some(oldhead.elem)
            } else {
                None
            }
            `None`
            // ^^only allowed if we did `return ...` in if  {}, since `return ...` is type "!" (see below, but its the bottom type)

        * "!" is the "never type", Rust's bottom type
        * Basically any "diverging expression" has type !. so return, break, continue statements all have type !. So do infinite loops, sysexits, panics, etc.
        * The bottom type is unihabited and a subtype of every type. So if one branch evaluates to !, the type of the expression depends on the other branches, since the "never" branch coerces to them
        * (so in this case an empty else with `None` outside would coerce the never branch to the unit type (), while `return Some(...)` in if would coerce to an option)
        */

        // matches pattern and sets to var if matched, otherwise does whats in {}
        let Link::More(oldhead) = mem::replace(&mut self.head, Link::Empty) else {
            // the code inside else has to diverge, i.e has to have the never type (!); the else branch can't fallthrough
            return None;
            // cause if we proceed, the var is guaranteed to exist, which might not be true if else fellthrough. and then the code wouldn't work
        };
        self.head = oldhead.next;
        Some(oldhead.elem)
    }
}

impl Drop for List {
    // we have to write our own drop, cause while droppping Links / Nodes are tail recursive, dropping a Box requires deallocating the ptr after dropping it which is not tail recursive
    // that would cause stack overflow
    fn drop(&mut self) {
        let mut current = mem::replace(&mut self.head, Link::Empty);

        // while let: while we can match the pattern, do {}
        while let Link::More(mut boxed_node) = current {
            current = mem::replace(&mut boxed_node.next, Link::Empty)
        }
    }
}

#[cfg(test)] // only compile when testing
mod test {
    use super::List;

    #[test] // this is a test
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }
}
