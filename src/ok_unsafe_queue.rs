use std::ptr;

struct Node<T> {
    elem: T,
    next: Link<T>,
}
type Link<T> = *mut Node<T>;

// we have `*mut T`and `*const T`. We can only deref a const pointer to &T, though you can get around this by casting to a `*mut T`. But like even then we still need perms to mutate the underlying value in the first place
// another consequence is no null ptr optimization, since we can't use `None` for the empty tail. We have to do `ptr::null` instead
pub struct List<T> {
    head: Link<T>,
    tail: Link<T>, // raw pointer, unsafe
}

impl<T> List<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        List {
            head: ptr::null_mut(),
            tail: ptr::null_mut(), // can also do `0 as *mut _`
        }
    }

    /*
    * Previously we ran into issues with the borrow stack and aliasing. When two pointers point to overlapping regions of memory, they are said to alias. The compiler uses aliasing to optimize memory access, so it can cache things or avoid comitting them to memory
    * Normally, shared references can't mutate so aliasing is fine, and mutable references can't alias each other. But we can reborrow mutable references which can fuck shit up.
            let mut data = 10;
            let ref1 = &mut data;
            let ref2 = &mut *ref1;

            // ORDER SWAPPED! INCORRECT
            *ref1 += 1;
            *ref2 += 2;

    * See how we use `ref1`, and then use `ref2`. Notice how it would fuck shit up if we did this, since `ref2` is dependent on ref1. The correct order should be use `ref2`, use `ref1`.
    * Essentially, when we have repeated borrows of the same value in memmory, there is a stack of borrows created and only the borrow at the top is live (i.e can write). The borrow checker checks this for normal pointers, while MIRI checks this for raw pointers
    * Now note that if we derive raw pointers from other raw pointers, MIRI doesn't give a fuck anymore. Every pointer shares the same tag (position in the borrow stack), cause the compiler won't do aliasing optimizations for raw ptrs.
    * We can also use `split_at_mut` to break up an array or smth into slices, so the borrow stack behaves more like a tree than a stack. This makes it so when borrowing from arrays, we can have equally valid mutable references to different parts of the array
            let (slice2_at_0, slice3_at_1) = slice1.split_at_mut(1);

    * we can also add 1 to a raw pointer, to increments its location. Since ptrs are  just integers lmfao
    * and shared references can be used interchangabely, since they're shared. Once a shared references is on the borrow stack, everything above should have read-only access.
    */
    pub fn push(&mut self, elem: T) {
        // technically we only need the unsafe for acc derefferening the raw ptr, but yk
        unsafe {
            let newtail = Box::into_raw(Box::new(Node {
                elem,
                next: ptr::null_mut(),
            }));

            // non-null check
            if !self.tail.is_null() {
                (*self.tail).next = newtail;
            } else {
                self.head = newtail
            }
            self.tail = newtail;
        }
    }

    /*
    * At one point, we tried:
           pub struct List<'a, T> {
               head: Link<T>,
               tail: Option<&'a mut Node<T>>, // NEW!
           }

           pub fn pop(&'a mut self) -> Option<T> {
               self.head.take().map(|head| {
                   let head = *head;
                   self.head = head.next;
                   if self.head.is_none() {
                       self.tail = None;
                   }
                   head.elem
               })
           }

    * but like the lifetimes are retarded lmao.
    * we borrow self mutably for 'a, so we can't use `self` till the end of 'a. But then `self` also stores a node with lifetime 'a inside it. So we can only call `push` or `pop` once since those essentially pin the list in place.
    * i.e, we're storing a reference to ourself inside ourselves. that's dumb af
    */
    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            if self.head.is_null() {
                None
            } else {
                let oldhead = Box::from_raw(self.head); // from_raw is unsafe
                self.head = oldhead.next;

                if self.head.is_null() {
                    self.tail = ptr::null_mut();
                }
                Some(oldhead.elem)
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        unsafe { self.head.as_ref().map(|node| &node.elem) }
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        unsafe { self.head.as_mut().map(|node| &mut node.elem) }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

pub struct IntoIter<T>(List<T>);

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

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
        self.0.pop()
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            Iter {
                next: self.head.as_ref(),
            }
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.map(|node| {
                self.next = node.next.as_ref();
                &node.elem
            })
        }
    }
}

impl<'a, T> IntoIterator for &'a mut List<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            IterMut {
                next: self.head.as_mut(),
            }
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.take().map(|node| {
                self.next = node.next.as_mut();
                &mut node.elem
            })
        }
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
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn miri_food() {
        let mut list = List::new();

        list.push(1);
        list.push(2);
        list.push(3);

        assert!(list.pop() == Some(1));
        list.push(4);
        assert!(list.pop() == Some(2));
        list.push(5);

        assert!(list.peek() == Some(&3));
        list.push(6);
        if let Some(x) = list.peek_mut() {
            *x *= 10;
        }
        assert!(list.peek() == Some(&30));
        assert!(list.pop() == Some(30));

        for elem in &mut list {
            *elem *= 100;
        }

        let mut iter = IntoIterator::into_iter(&list);
        assert_eq!(iter.next(), Some(&400));
        assert_eq!(iter.next(), Some(&500));
        assert_eq!(iter.next(), Some(&600));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        assert!(list.pop() == Some(400));
        if let Some(x) = list.peek_mut() {
            *x *= 10;
        }
        assert!(list.peek() == Some(&5000));
        list.push(7);

        // Drop it on the ground and let the dtor exercise itself
    }
}
