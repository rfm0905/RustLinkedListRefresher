use std::rc::Rc;

use std::cell::{Ref, RefCell, RefMut};
/*
* RefCell does borrows (both mutable and shared) at runtime instead of compile time, but still follows the same ownership rules. Implemented via:
  fn borrow(&self) -> Ref<'_, T>;
  fn borrow_mut(&self) -> RefMut<'_, T>;

* `RefCell`s can be changed by mutable references but also shared references through interior mutability. With a mutable reference, we have the standard compile-time guarantee, while with the shared reference this is at run-time and we use borrow/borrow_mut.
* `Cell` has similar behaviour, but implements `get` and `set`, and is usually used for types with Copy. So it never gives out a reference to the interior value, just copy in/out. but still allows us to mutate with shared references.
* The process through which `RefCell` does this is termed "Dynamic Borrowing," where one gains temporary and exclusive access to the inner value, tracked at runtime. This is the "interior mutability" pattern.
* This allows us to mutate values inside `Rc/Arc`, which otherwise is not possible since we can only take shared references for those types.
* Note that `RefCells` are for single-thread scenarios, if we need multiple threads use a Mutex.
*/

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem,
            prev: None,
            next: None,
        }))
    }
}

impl<T> List<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let newhead = Node::new(elem);
        match self.head.take() {
            Some(oldhead) => {
                /* remember, clone for an `Rc` just clones the ptr and increments the reference count
                 * So for a `RefCell`, we have to use `borrow_mut()` to edit.
                 */
                oldhead.borrow_mut().prev = Some(newhead.clone()); // `borrow_mut` auto derefs, cause 'Rc' implements `Deref`
                newhead.borrow_mut().next = Some(oldhead);
                self.head = Some(newhead);
            }
            None => {
                self.tail = Some(newhead.clone()); // need to clone cause it moves setting directly
                self.head = Some(newhead);
            }
        }
    }

    pub fn push_back(&mut self, elem: T) {
        let newtail = Node::new(elem);
        match self.tail.take() {
            Some(oldtail) => {
                oldtail.borrow_mut().next = Some(newtail.clone());
                newtail.borrow_mut().prev = Some(oldtail);
                self.tail = Some(newtail);
            }
            None => {
                self.head = Some(newtail.clone());
                self.tail = Some(newtail);
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|oldhead| {
            match oldhead.borrow_mut().next.take() {
                Some(newhead) => {
                    newhead.borrow_mut().prev.take();
                    self.head = Some(newhead);
                }
                None => {
                    self.tail.take();
                }
            }
            Rc::try_unwrap(oldhead).ok().unwrap().into_inner().elem
        })
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|oldtail| {
            match oldtail.borrow_mut().prev.take() {
                Some(newtail) => {
                    newtail.borrow_mut().next.take();
                    self.tail = Some(newtail);
                }
                None => {
                    self.head.take();
                }
            }
            Rc::try_unwrap(oldtail).ok().unwrap().into_inner().elem
        })
    }

    pub fn peek_front(&'_ self) -> Option<Ref<'_, T>> {
        self.head.as_ref().map(|head| {
            /*
             * recall `borrow` and `borrowMut` return `Ref` and `RefMut` respectively
             * so we can't just directly return node.borow().elem (since that returns a `Ref`), but need to get out of the `Ref`
             * we could just return `Option<Ref<T>>` and use map directly, or do a whole ass wrapper with `Deref` for that type, but that's boring.
             * `Ref::map()`` does this. It takes in a `Ref`, and returns a new `Ref`, just like a `option::map`; And consumes the original Ref just like `option::map`
             */
            Ref::map(head.borrow(), |head| &head.elem)
        })
    }

    pub fn peek_back(&'_ self) -> Option<Ref<'_, T>> {
        self.tail
            .as_ref()
            .map(|tail| Ref::map(tail.borrow(), |tail| &tail.elem))
    }

    pub fn peek_front_mut(&'_ mut self) -> Option<RefMut<'_, T>> {
        self.head
            .as_ref() // this is interior mutability; we are mutating the interior, not the acc Rc so we take it as a normal reference
            .map(|head| RefMut::map(head.borrow_mut(), |head| &mut head.elem))
    }

    pub fn peek_back_mut(&'_ mut self) -> Option<RefMut<'_, T>> {
        self.tail
            .as_ref()
            .map(|tail| RefMut::map(tail.borrow_mut(), |tail| &mut tail.elem))
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

pub struct IntoIter<T>(List<T>);

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self) // how we acc make the List the Iterator
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

// for going in reverse
impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}

/*
// Iter and IterMut are too dumb/hard to implement
    pub struct Iter<'a, T>(Option<Ref<'a, Node<T>>>);
    impl<T> List<T> {
        pub fn iter(&self) -> Iter<T> {
            Iter(self.head.as_ref().map(|head| head.borrow()))
        }
    }

// first try:
    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = Ref<'a, T>;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.take().map(|node_ref| {
                self.0 = node_ref.next.as_ref().map(|head| head.borrow());
                Ref::map(node_ref, |node| &node.elem)
            })
        }
    }

// this won't work cause like think about it. We mutably borrow `Iter` first in the call to next(). This is fine in itself.
// Then we declare `node_ref` as a `Ref` to the initial head. also fine no new borrow.
// Then inside the closure, we borrow the next value (A ref) with head.borrow(). Note this borrow only has the lifetime of the closure we're mappping node_ref with.
// But we're trying to set the iterators head, which is going to exist outside the closure (and also even longer outside `next()` itself), to this borrow. The lifetimes just don't add up.
// (like we can't set to a borrow done inside the function call, since the iterators outlasts the reference we're returning)
// With a normal `Box` or smth, we can do this since the reference is tied to the original parameter. but since we're using a `RefCell`, each refernce has to be done dynamically in-place.
// we could try using `map_split` to split up the OG Ref like that, with the same lifetime as the origial. but that doesn't work since we're tryna do a new borrow here, not just split up the old one. using Clone() makes it confusing as to whether return a `Rc` or `T` or what.
// there's a crate that ac lets us creating an owning reference to directly get an Rc to the inner value (so Rc<Node<T> --> Rc<T>) but then the iterator could be invalid. like someone could call pop on the returned value and fuck it up.
*/

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);

        // ---- back -----

        // Check empty list behaves right
        assert_eq!(list.pop_back(), None);

        // Populate list
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_back(4);
        list.push_back(5);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
        assert!(list.peek_front_mut().is_none());
        assert!(list.peek_back_mut().is_none());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(&*list.peek_front().unwrap(), &3);
        assert_eq!(&mut *list.peek_front_mut().unwrap(), &mut 3);
        assert_eq!(&*list.peek_back().unwrap(), &1);
        assert_eq!(&mut *list.peek_back_mut().unwrap(), &mut 1);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}
