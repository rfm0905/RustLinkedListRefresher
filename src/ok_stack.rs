struct Node<T> {
    elem: T,
    next: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

pub struct List<T> {
    head: Link<T>,
}

impl<T> List<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let newhead = Box::new(Node {
            elem,
            // `take` does `mem::replace` for us. It replaces what we're taking with the default, which is `None` and returns what was there originally.
            next: self.head.take(),
        });
        self.head = Some(newhead);
    }

    pub fn pop(&mut self) -> Option<T> {
        // let node = self.head.take()?;
        // The `?` on the option pattern matches. `Some(...)` => sets the variable and proceeds, `None` => returns None (thereby aborting the function)

        // `map` on option sends `None => None` and `Some(node) => Some(...)` (based on whatever the closure does)
        // we have to `take` before `map`, since `map` moves (takes ownership of) the value
        // (for some reason chat gets mad when you say move, but like it is lmao)
        self.head.take().map(|head| {
            self.head = head.next;
            head.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        // as_ref: option<T> --> option<&T>
        // `as_ref` lets us do the `map` without moving, cause the closure acts on the option with a reference, not the acc value.
        self.head.as_ref().map(|head| &head.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|head| &mut head.elem)
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();

        while let Some(mut boxed_node) = current {
            current = boxed_node.next.take();
        }
    }
}

/* ITERATORS
`IntoIter`: for x in List<T>; Consuming/Owning Iterator;
`Iter`: For &x in List<T>; doesn't take ownership
`IterMut`: For &mut x in List<T>;

Each iterator has a wrapper type: `IntoIter<T>`, `Iter<T>`and `IterMut<T>`. These are what the list is converted to when we implement an iterator. allows it to work with `for x in List<T>` syntax

we implement the `IntoIterator<T>` to define how to convert to an iterator. It requires us to implement:
- `type Item = `, what the type of each element will be
- `type IntoIter = `, what type of iterator we're converting this into
- `fn into_iter(self) -> Self::IntoIter {}`, how to acc convert our underlying List into this iterator

we then also implement `Iterator<T>` for the acc interface of teh iterator. It has `type Item = `, and then `fn next(&mut self) -> Option<Self::Item>` which is what each iteration does.
*/

pub struct IntoIter<T>(List<T>); // one-element product type, has one field which is the list

// this one captures ownership, so we implement it on `List<T>` directly
impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop() // 0 is the one unamed field (so the list) in IntoIter<T>
    }
}

/*
 * lifetimes basically annotate how long a reference is valid for. the borrow checker confirms the code aligns with the lifetime
 * The compiler follow 3 rules to decide when to elide lifetimes
    - Each parameter that is a reference gets its own lifetime.
        - foo<'a, 'b, 'c>(&'a A, &'b B, &'c C); we have three input references, so each input gets its own lifetiem
    - If we only have one input lifetime, the output obv gets that lifetime
        - &'a s --> &'a b; so we can elide lifetimes here
        -  foo<'a, 'b, 'c>(&'a A, &'b B, &'c C); but here we can't, and need to explicitly mark the output lifetime since we don't know which one it depends on.
    - if one of the parameters is `&self` or `&mut self`, the output lifetime derives from that param.
        - fn foo<'a, 'b, 'c>(&'a self, &'b B, &'c C) -> &'a D; the output lifetime is the same as `self`

* we need lifetimes when
    - structs store references, like the code we have rn
    - function signatures when output borrows/is related to the input
*/

// here, the iterator depends on some lifetime, but doesn't matter which one
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

// The iterator has a lifetime, since Iter has one
impl<'a, T> Iterator for Iter<'a, T> {
    /*
     * this is okay since we defined `Iter` as a container that stores references, not the acc list
     * if we didn't do that `fn next` would mutably borrow the whole list, while also yielding a reference in the previous iteration to something in the list. that reference might still be valid after next().
     * so we'd have a mutable borrow simultaneously with the previous immutable borrow, to the same collection. that's why we can't have self-borrowing iterators, or iterators that borrow items from themselves
     * but in our case, we mutuably borrow the iterator in next, but return a reference to the og collection. and that's okay since the iterator is a diff object from the collection.
     */
    type Item = &'a T;

    // this one gets it lifetime from the above and self
    // note that the option gets its lifetime as 'a, not the lifetime of self
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

// same as above, as long as the reference to the list is valid so is the iterator
impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// we can't just rely on `IntoIterator` here, since that defines a way to go from &List<T> --> Iter, not List --> Iter.
impl<T> List<T> {
    // lifetime for IterList can be elided since this takes in self, but clippy gets mad so we do the _ thing to indicate its elided
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            // next: self.head.map(|node| &node) // WRONG; since we would get `Box<Node>` not `Node`.
            // `as_deref` dereferences the value inside the option, but doesn't move. like here we have `Box<Node>`, and we get `&Node`. `as_ref` would just give `&Box<Node>`
            next: self.head.as_deref(),
        }
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        // we have to use take here, since we're taking a mutable reference. map moves the original
        // for the non-mut case we could use use map straight cause shared references are Copy, so map doesn't acc move anything. But mutable references are acc moved
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}

impl<'a, T> IntoIterator for &'a mut List<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
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

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));

        /*
        list.peek_mut().map(|&mut value| value = 42);
        * ^^this doesn't work, since a closure arg is a pattern match
        * so rn, we just match a mutable reference, then copy the value into value
        list.peek_mut().map(|value| *value = 42);
        ^^so we need this, which acc puts a mutuable reference into value
        */

        if let Some(value) = list.peek_mut() {
            *value = 42;
        }

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}
