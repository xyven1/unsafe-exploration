use std::ptr::NonNull;

struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

enum CursorPos<T> {
    Start,
    End,
    Node(NonNull<Node<T>>),
}

struct Cursor<'a, T> {
    parent: &'a LinkedList<T>,
    position: CursorPos<T>,
}

struct CursorMut<'a, T> {
    parent: &'a mut LinkedList<T>,
    position: CursorPos<T>,
}

struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
    value: T,
}

fn to_non_null<T>(node: Node<T>) -> NonNull<Node<T>> {
    NonNull::from(Box::leak(Box::new(node)))
}

#[allow(dead_code)]
impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: None,
            tail: None,
            len: 0,
        }
    }
    pub fn iter(&self) -> Cursor<'_, T> {
        Cursor {
            parent: self,
            position: CursorPos::Start,
        }
    }
    pub fn push_front(&mut self, value: T) {
        match self.head.take() {
            Some(mut old_head) => {
                let new_node_ptr = to_non_null(Node {
                    next: Some(old_head),
                    prev: None,
                    value,
                });
                unsafe {
                    old_head.as_mut().prev = Some(new_node_ptr);
                }
                self.head = Some(new_node_ptr)
            }
            None => {
                let new_node_ptr = to_non_null(Node {
                    next: None,
                    prev: None,
                    value,
                });
                self.head = Some(new_node_ptr);
                self.tail = Some(new_node_ptr);
            }
        }
        self.len += 1;
    }
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            match unsafe { old_head.as_ref().next } {
                Some(mut new_head) => {
                    unsafe {
                        new_head.as_mut().prev = None;
                    }
                    self.head = Some(new_head);
                }
                None => {
                    self.tail = None;
                }
            }
            self.len -= 1;
            let value = unsafe { old_head.as_ptr().read().value };
            unsafe {
                drop(Box::from_raw(old_head.as_ptr()));
            }
            value
        })
    }
    pub fn push_back(&mut self, value: T) {
        match self.tail.take() {
            Some(mut old_tail) => {
                let new_node_ptr = to_non_null(Node {
                    next: None,
                    prev: Some(old_tail),
                    value,
                });
                unsafe {
                    old_tail.as_mut().next = Some(new_node_ptr);
                }
                self.tail = Some(new_node_ptr);
            }
            None => {
                let new_node_ptr = to_non_null(Node {
                    next: None,
                    prev: None,
                    value,
                });
                self.head = Some(new_node_ptr);
                self.tail = Some(new_node_ptr);
            }
        }
        self.len += 1;
    }
    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            match unsafe { old_tail.as_ref().prev } {
                Some(mut new_tail) => {
                    unsafe {
                        new_tail.as_mut().next = None;
                    }
                    self.tail = Some(new_tail);
                }
                None => {
                    self.head = None;
                }
            }
            self.len -= 1;
            let value = unsafe { old_tail.as_ptr().read().value };
            unsafe {
                drop(Box::from_raw(old_tail.as_ptr()));
            }
            value
        })
    }
    pub fn cursor_front<'a>(&'a self) -> Cursor<'a, T> {
        Cursor {
            parent: self,
            position: self
                .head
                .map(|v| CursorPos::Node(v))
                .unwrap_or(CursorPos::Start),
        }
    }
    pub fn cursor_back<'a>(&'a self) -> Cursor<'a, T> {
        Cursor {
            parent: self,
            position: self
                .tail
                .map(|v| CursorPos::Node(v))
                .unwrap_or(CursorPos::End),
        }
    }
    pub fn cursor_front_mut<'a>(&'a mut self) -> CursorMut<'a, T> {
        let position = self
            .head
            .map(|v| CursorPos::Node(v))
            .unwrap_or(CursorPos::Start);
        CursorMut {
            parent: self,
            position,
        }
    }
    pub fn cursor_back_mut<'a>(&'a mut self) -> CursorMut<'a, T> {
        let position = self
            .tail
            .map(|v| CursorPos::Node(v))
            .unwrap_or(CursorPos::End);
        CursorMut {
            parent: self,
            position,
        }
    }
}

impl<'a, T> Iterator for Cursor<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.move_next();
        self.current()
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_back().is_some() {}
    }
}

#[allow(dead_code)]
impl<'a, T> Cursor<'a, T> {
    pub fn move_next(&mut self) {
        self.position = match self.position {
            CursorPos::Start => match self.parent.head {
                Some(v) => CursorPos::Node(v),
                None => CursorPos::End,
            },
            CursorPos::End => CursorPos::End,
            CursorPos::Node(v) => match unsafe { v.as_ref().next } {
                Some(next) => CursorPos::Node(next),
                None => CursorPos::End,
            },
        }
    }
    pub fn move_prev(&mut self) {
        self.position = match self.position {
            CursorPos::Start => CursorPos::Start,
            CursorPos::End => match self.parent.tail {
                Some(v) => CursorPos::Node(v),
                None => CursorPos::Start,
            },
            CursorPos::Node(v) => unsafe {
                match v.as_ref().prev {
                    Some(prev) => CursorPos::Node(prev),
                    None => CursorPos::Start,
                }
            },
        }
    }
    pub fn current(&self) -> Option<&'a T> {
        match self.position {
            CursorPos::Node(v) => Some(unsafe { &v.as_ref().value }),
            _ => None,
        }
    }
}

#[allow(dead_code)]
impl<'a, T> CursorMut<'a, T> {
    pub fn move_next(&mut self) {
        self.position = match self.position {
            CursorPos::Start => match self.parent.head {
                Some(v) => CursorPos::Node(v),
                None => CursorPos::End,
            },
            CursorPos::End => CursorPos::End,
            CursorPos::Node(v) => unsafe {
                match v.as_ref().next {
                    Some(next) => CursorPos::Node(next),
                    None => CursorPos::End,
                }
            },
        }
    }
    pub fn move_prev(&mut self) {
        self.position = match self.position {
            CursorPos::Start => CursorPos::Start,
            CursorPos::End => match self.parent.tail {
                Some(v) => CursorPos::Node(v),
                None => CursorPos::Start,
            },
            CursorPos::Node(v) => unsafe {
                match v.as_ref().prev {
                    Some(prev) => CursorPos::Node(prev),
                    None => CursorPos::Start,
                }
            },
        }
    }
    pub fn current(&mut self) -> Option<&mut T> {
        match self.position {
            CursorPos::Node(mut v) => Some(unsafe { &mut v.as_mut().value }),
            _ => None,
        }
    }
    pub fn remove(&mut self) -> Option<T> {
        match self.position {
            CursorPos::Node(v) => {
                let node = v.as_ptr();
                let next = unsafe { node.read().next };
                let prev = unsafe { node.read().prev };
                match (prev, next) {
                    (Some(mut prev), Some(mut next)) => {
                        unsafe {
                            prev.as_mut().next = Some(next);
                            next.as_mut().prev = Some(prev);
                        }
                        self.position = CursorPos::Node(next);
                    }
                    (Some(mut prev), None) => {
                        unsafe {
                            prev.as_mut().next = None;
                        }
                        self.position = CursorPos::End;
                    }
                    (None, Some(mut next)) => {
                        unsafe {
                            next.as_mut().prev = None;
                        }
                        self.position = CursorPos::Node(next);
                    }
                    (None, None) => {
                        self.position = CursorPos::End;
                    }
                }
                self.parent.len -= 1;
                Some(unsafe { Box::from_raw(node).value })
            }
            _ => None,
        }
    }
    pub fn insert_after(&mut self, value: T) {
        match self.position {
            CursorPos::Start => {
                self.parent.push_front(value);
            }
            CursorPos::End => {
                self.parent.push_back(value);
            }
            CursorPos::Node(mut v) => {
                let next = unsafe { v.as_ref().next };
                let new_node_ptr = to_non_null(Node {
                    next,
                    prev: Some(v),
                    value,
                });
                unsafe {
                    v.as_mut().next = Some(new_node_ptr);
                }
                match next {
                    Some(mut next) => unsafe {
                        next.as_mut().prev = Some(new_node_ptr);
                    },
                    None => {
                        self.parent.tail = Some(new_node_ptr);
                    }
                }
            }
        }
        self.parent.len += 1;
    }
}

#[test]
fn test_push_pop_front() {
    let mut list = LinkedList::new();
    list.push_front(1);
    list.push_front(2);
    list.push_front(3);
    assert_eq!(list.pop_front(), Some(3));
    assert_eq!(list.pop_front(), Some(2));
    assert_eq!(list.pop_front(), Some(1));
    assert_eq!(list.pop_front(), None);
}

#[test]
fn test_push_pop_back() {
    let mut list = LinkedList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    assert_eq!(list.pop_back(), Some(3));
    assert_eq!(list.pop_back(), Some(2));
    assert_eq!(list.pop_back(), Some(1));
    assert_eq!(list.pop_back(), None);
}

#[test]
fn test_cursor_basic() {
    let mut list = LinkedList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    let mut cursor = list.cursor_front();
    assert_eq!(cursor.parent.len, 3);
    assert_eq!(cursor.current(), Some(&1));
    cursor.move_next();
    assert_eq!(cursor.current(), Some(&2));
    cursor.move_next();
    assert_eq!(cursor.current(), Some(&3));
    cursor.move_next();
    assert_eq!(cursor.current(), None);
    cursor.move_next();
    assert_eq!(cursor.current(), None);
    cursor.move_prev();
    assert_eq!(cursor.current(), Some(&3));
}

#[test]
fn test_iterator() {
    let mut list = LinkedList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    let mut iter = list.iter();
    assert_eq!(iter.next(), Some(&1));
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn test_cursor_insert_after() {
    let mut list = LinkedList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    let mut cursor = list.cursor_front_mut();
    assert_eq!(cursor.parent.len, 3);
    cursor.move_next();
    cursor.insert_after(4);
    assert_eq!(cursor.parent.len, 4);
    cursor.move_prev();
    assert_eq!(cursor.current(), Some(&mut 1));
    cursor.move_next();
    assert_eq!(cursor.current(), Some(&mut 2));
    cursor.move_next();
    assert_eq!(cursor.current(), Some(&mut 4));
    cursor.move_next();
    assert_eq!(cursor.current(), Some(&mut 3));
    cursor.move_next();
    assert_eq!(cursor.current(), None);
}
