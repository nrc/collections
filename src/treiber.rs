// Treiber's stack - a lock free, concurrent stack.

// TODO do the fancy lock free stuff.

use std::cell::UnsafeCell;
use std::sync::Mutex;

pub struct Stack<T> {
    len: UnsafeCell<usize>,
    head: UnsafeCell<Option<Box<Node<T>>>>,
    lock: Mutex<i32>,
}

unsafe impl<T> Sync for Stack<T> {}

struct Node<T> {
    datum: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack {
            len: UnsafeCell::new(0),
            head: UnsafeCell::new(None),
            lock: Mutex::new(0),
        }
    }

    pub fn len(&self) -> usize {
        unsafe {
            let _l = self.lock.lock().unwrap();
            *self.len.get()
        }
    }

    pub fn push(&self, datum: T) {
        unsafe {
            let _l = self.lock.lock().unwrap();
            let head = self.head.get();
            *self.len.get() += 1;
            *head = Some(Box::new(Node {
                datum: datum,
                next: (*head).take()
            }));
        }
    }

    pub fn try_pop(&self) -> Option<T> {
        unsafe {
            let _l = self.lock.lock().unwrap();
            let head = self.head.get();
            let len = self.len.get();
            if *len == 0 {
                return None;
            }
            *len -= 1;
            let Node { datum, next } = *(*head).take().unwrap();
            *head = next;
            Some(datum)
        }        
    }

    pub fn pop(&self) -> T {
        unsafe {
            let _l = self.lock.lock().unwrap();
            let head = self.head.get();
            let len = self.len.get();
            assert!(*len > 0);
            *len -= 1;
            let Node { datum, next } = *(*head).take().unwrap();
            *head = next;
            datum
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crossbeam::scope;
    use std::sync::Mutex;
    use std::thread::yield_now;

    #[test]
    #[should_panic]
    fn test_push_pop_empty() {
        let s: Stack<usize> = Stack::new();
        s.pop();
    }

    #[test]
    fn test_push_pop() {
        let s = Stack::new();
        assert!(s.len() == 0);
        s.push(42);
        assert!(s.len() == 1);
        assert!(s.pop() == 42);
        assert!(s.len() == 0);
    }

    #[test]
    fn test_push_pop_3() {
        let s = Stack::new();
        assert!(s.len() == 0);
        s.push(42);
        s.push(43);
        s.push(44);
        assert!(s.len() == 3);
        assert!(s.pop() == 44);
        assert!(s.pop() == 43);
        assert!(s.pop() == 42);
        assert!(s.len() == 0);
    }

    #[test]
    fn test_push_try_pop_empty() {
        let s: Stack<usize> = Stack::new();
        assert!(s.try_pop().is_none());
    }

    #[test]
    fn test_push_try_pop_3() {
        let s = Stack::new();
        assert!(s.len() == 0);
        s.push(42);
        s.push(43);
        s.push(44);
        assert!(s.len() == 3);
        assert!(s.try_pop().unwrap() == 44);
        assert!(s.try_pop().unwrap() == 43);
        assert!(s.try_pop().unwrap() == 42);
        assert!(s.len() == 0);
    }

    #[test]
    fn test_parallel() {
        let s = Stack::new();
        let sum = Mutex::new(0);
        scope(|sc| {
            for _ in 0..10 {
                sc.spawn(|| {
                    let mut local_sum = 0;
                    for i in 0..1000 {
                        s.push(i);
                        let mut v = s.try_pop();
                        while v.is_none() {
                            yield_now();
                            v = s.try_pop();
                        }
                        local_sum += v.unwrap();
                    }
                    println!("{}", local_sum);
                    *sum.lock().unwrap() += local_sum;
                });
            }
        });
        assert!(*sum.lock().unwrap() == 4995000);
    }
}
