// use std::rc::Rc;
use std::sync::Arc;
/*
 * `Rc` does (R)eference (C)ounting, which allows for shared ownership. Each Rc<T> stores one ptr for the data, and the "strong_count", which is the number of references in scope.
 * We allocate data on the heap like `Box`, but unlike `Box` we can duplicate it; Memory is freed when the (strong) count is zero.
 * As a result, we can only take shared references to the value in `Rc`, but not mutable ones.
 * Note that this is still pretty weak though, like cycles would completely mess it up.
 * `Arc` is threadsafe. A normal Rc could have two threads increment the reference count at the same time, with only one going through. `Arc` (Atomic Reference Count) solves this by incrementing atomically.
 * In general, thread safety in Rust is modeled by the traits Send/Sync. *Send* says its okay to move to another thread, while *Sync* means its okay to share between threads. (Like T is Send, while &T is Sync). Rust doesn't let you violate this
 */

struct Node<T> {
    elem: T,
    next: Link<T>,
}

type Link<T> = Option<Arc<Node<T>>>;

pub struct List<T> {
    head: Link<T>,
}

impl<T> List<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        List { head: None }
    }

    // here, we're returning a whole new list cause of the Rc
    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Arc::new(Node {
                elem, // it can auto put elem to the node.elem field

                // clone: pointer is copied (not the value), while strong count incremented by 1
                next: self.head.clone(),
            })),
        }
    }

    pub fn tail(&self) -> List<T> {
        List {
            // and_then returns a full option; not an unwrapped value like map/take
            // we can't just use unwrap cause of the Rc.
            head: self.head.as_ref().and_then(|head| head.next.clone()),
        }
    }

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|head| &head.elem)
    }
}

// we can't implement IterMut (mutable references) or IntoIter (full ownership) for this type, since Rc only has shared references
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        /*  RC Code,
        let mut head = self.head.take();
        while let Some(node) = head {
            // try_unwrap checks to make sure we are the last reference (strong count), so we can safely mutate.
            // If we are the last node, it returns Ok(node), otherwise it returns Err(Rc<...>) which we can't mutate since its "stuck" behind the RC
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
        */

        let mut head = self.head.take();
        while let Some(node) = head.take() {
            /*
             * For `Arc`, we need to use `into_inner` instead of `try_unwrap` cause of race conditions
             * If two threads tried to drop at the same time, and both saw they don't have the last strong count, `try_unwrap` would fail for both of them
             * This would prevent the drop code from running in both cases, even if one of them should have been the last reference. This would cause the stack overflow we needed to avoid (see badstack: 80), since the compiler would be forced to drop the node itself.
             * `into_inner` solves this, by guaranteeing at least one thread gets the inner value (provided we call into_inner on every thread), so the node isn't automatically dropped and we can process it ourselves.
             */
            if let Some(mut node) = Arc::into_inner(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}
