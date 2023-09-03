// A more serious attempt this time.
// We want a linked likst where I can do
// list1 = A -> B -> C -> D
// list2 = tail(list1) = B -> C -> D
// list3 = push(list2, X) = X -> B -> C -> D
// But the memory must end like this:
// list1 -> A ---+
//               |
//               v
// list2 ------> B -> C -> D
//               ^
//               |
// list3 -> X ---+

// You can't use Box(B), as it is owned.
// Who should drop B, list1, 2 or 3?
// We use Rc to _reference count_ the elements.
// Rc is like Box, but we can only get
// shared references of the internal values :(

use std::rc::Rc;

type Link<T> = Option<Rc<Node<T>>>;
struct Node<T> {
    elem: T,
    next: Link<T>,
}

pub struct List<T> {
    head: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                elem,
                // Note: head is a Link which is an Option<Rc>
                // cloning an Option is like cloning only if Some
                // and cloning an Rc is cloning only the pointer!
                // (and incrementing the reference count)
                next: self.head.clone(),
            })),
        }
    }

    pub fn tail(&self) -> List<T> {
        // Note: and_then is like what you though map did
        // map takes Fn: Option<U> -> T, infallible
        // and_then takes Fn: Option<U> -> Option<T>, fallible
        let tail = self.head.as_ref().and_then(|rc_node| rc_node.next.clone());
        List { head: tail }
    }

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|rc_node| &rc_node.elem)
    }
}

impl<T> Drop for List<T> {
    // We can't iterate over "next" and replace them with None 
    // like we did with our non-rc version: that involves 
    // mutating a value inside an Rc and we can't do that. 
    // But we can use Rc::try_unwrap() to check if we 
    // are the last holder of the data inside an rc 
    // If we are the last one, Rc gives us the value 
    // back to use as we please
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

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
