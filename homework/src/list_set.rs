use std::cmp;
use std::ptr;
use std::sync::{Mutex, MutexGuard};

#[derive(Debug)]
struct Node<T> {
    data: T,
    next: Mutex<*mut Node<T>>,
}

unsafe impl<T: Send> Send for Node<T> {}
unsafe impl<T: Sync> Sync for Node<T> {}

/// Concurrent sorted singly linked list using lock-coupling.
#[derive(Debug)]
pub struct OrderedListSet<T> {
    head: Mutex<*mut Node<T>>,
}

unsafe impl<T: Send> Send for OrderedListSet<T> {}
unsafe impl<T: Sync> Sync for OrderedListSet<T> {}

// reference to the `next` field of previous node which points to the current node
struct Cursor<'l, T>(MutexGuard<'l, *mut Node<T>>);

impl<T> Node<T> {
    fn new(data: T, next: *mut Self) -> *mut Self {
        Box::into_raw(Box::new(Self {
            data,
            next: Mutex::new(next),
        }))
    }
}

impl<'l, T: Ord> Cursor<'l, T> {
    /// Move the cursor to the position of key in the sorted list. If the key is found in the list,
    /// return `true`.
    fn find(&mut self, key: &T) -> bool {
        //todo!()

        unsafe {
            while !(*(self.0)).is_null() {
                let node_key = &(**(self.0)).data;
                if node_key.cmp(key) == std::cmp::Ordering::Equal {
                    return true;
                }

                let new_cursor = Cursor((**(self.0)).next.lock().unwrap());
                let old_cursor = std::mem::replace(self, new_cursor);

                drop(old_cursor.0);
            }
            false
        }
    }
}

impl<T> OrderedListSet<T> {
    /// Creates a new list.
    pub fn new() -> Self {
        Self {
            head: Mutex::new(ptr::null_mut()),
        }
    }
}

impl<T: Ord> OrderedListSet<T> {
    fn find(&self, key: &T) -> (bool, Cursor<T>) {
        //todo!()
        let mut cursor = Cursor(self.head.lock().unwrap());
        let result = cursor.find(key);
        (result, cursor)
    }

    /// Returns `true` if the set contains the key.
    pub fn contains(&self, key: &T) -> bool {
        let mut cursor = Cursor(self.head.lock().unwrap());
        let result = cursor.find(key);
        drop(cursor.0);
        result
    }

    /// Insert a key to the set. If the set already has the key, return the provided key in `Err`.
    pub fn insert(&self, key: T) -> Result<(), T> {
        let mut head_node_guard = self.head.lock().unwrap();

        unsafe {
            // Case1: list is empty -> insert at first
            if (*head_node_guard).is_null() {
                let new_node = Node::new(key, ptr::null_mut());
                *head_node_guard = new_node;
                return Ok(());
            }

            // Case2: given key is smaller than the key of head
            let first_key = &(**head_node_guard).data;
            if first_key.cmp(&key) == std::cmp::Ordering::Greater {
                let new_node = Node::new(key, *head_node_guard);
                *head_node_guard = new_node;
                return Ok(());
            }

            while !(*head_node_guard).is_null() {
                let first_key = &(**head_node_guard).data;
                let mut next_node_guard = (**head_node_guard).next.lock().unwrap();

                // find a duplicate key
                if key.cmp(first_key) == std::cmp::Ordering::Equal {
                    return Err(key);
                }

                // reaching last node of the list
                if (*next_node_guard).is_null() {
                    *next_node_guard = Node::new(key, ptr::null_mut());
                    return Ok(());
                }

                // find an appropriate place
                let next_key = &(**next_node_guard).data;
                if key.cmp(next_key) == std::cmp::Ordering::Less {
                    *next_node_guard = Node::new(key, *next_node_guard);
                    return Ok(());
                }

                // move to next node
                drop(head_node_guard);
                head_node_guard = next_node_guard;
            }
            Err(key)
        }
    }

    /// Remove the key from the set and return it.
    pub fn remove(&self, key: &T) -> Result<T, ()> {
        let mut head_node_guard = self.head.lock().unwrap();

        unsafe {
            // list is empty
            if (*head_node_guard).is_null() {
                return Err(());
            }

            // remove the first node
            if (**head_node_guard).data.cmp(key) == std::cmp::Ordering::Equal {
                let box_node = Box::from_raw(*head_node_guard);
                let second_node_guard = (**head_node_guard).next.lock().unwrap();
                *head_node_guard = *second_node_guard;

                return Ok((*box_node).data);
            }

            while !(*head_node_guard).is_null() {
                let mut next_node_guard = (**head_node_guard).next.lock().unwrap();

                // If second node is empty
                if (*next_node_guard).is_null() {
                    return Err(());
                }

                let mut next_node_data = &(**next_node_guard).data;

                // compare key with that of second node
                if next_node_data.cmp(key) == std::cmp::Ordering::Equal {
                    let box_node = Box::from_raw(*next_node_guard);
                    let third_guard = (**next_node_guard).next.lock().unwrap();
                    *next_node_guard = *third_guard;

                    return Ok((*box_node).data);
                }

                drop(head_node_guard);
                head_node_guard = next_node_guard;
            }
            Err(())
        }
    }
}

#[derive(Debug)]
pub struct Iter<'l, T>(Option<MutexGuard<'l, *mut Node<T>>>);

impl<T> OrderedListSet<T> {
    /// An iterator visiting all elements.
    pub fn iter(&self) -> Iter<T> {
        Iter(Some(self.head.lock().unwrap()))
    }
}

impl<'l, T> Iterator for Iter<'l, T> {
    type Item = &'l T;

    fn next(&mut self) -> Option<Self::Item> {
        //todo!()
        unsafe {
            let guard = self.0.take().unwrap();

            if (*guard).is_null() {
                return None;
            }

            let data = &(**guard).data;
            let next_guard = (**guard).next.lock().unwrap();
            drop(guard);

            self.0 = Some(next_guard);
            Some(data)
        }
    }
}

impl<T> Drop for OrderedListSet<T> {
    fn drop(&mut self) {
        let mut node_ptr = *self.head.get_mut().unwrap();
        unsafe {
            // list is empty
            if (node_ptr).is_null() {
                return;
            }

            let mut next_node_ptr = *(*node_ptr).next.get_mut().unwrap();
            let node_to_delete = Box::from_raw(node_ptr);
            drop(node_to_delete);

            while !(next_node_ptr).is_null() {
                node_ptr = next_node_ptr;
                next_node_ptr = *(*node_ptr).next.lock().unwrap();
                let node_to_delete = Box::from_raw(node_ptr);
                drop(node_to_delete);
            }
        }
    }
}

impl<T> Default for OrderedListSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
