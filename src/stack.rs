// A stack implemented using an array (well, not even a proper array, but pointer
// offsets).

use std::mem;
use std::ptr;
use std::rt::heap;

pub struct Stack<T> {
    data: *mut T,
    length: usize,
    capacity: usize,
}

// TODO
// resizing
// use Unique
// ZSTs
// overflow of length

impl<T> Stack<T> {
    pub fn with_capacity(capacity: usize) -> Stack<T> {
        Stack {
            data: allocate::<T>(capacity),
            length: 0,
            capacity: capacity,
        }
    }

    pub fn new() -> Stack<T> {
        Stack::with_capacity(32)
    }

    pub fn peek(&self) -> &T {
        assert!(self.length > 0, "Called `peek()` on empty stack");
        unsafe {
            let ptr = self.data.offset((self.length - 1) as isize);
            mem::transmute(ptr)
        }
    }

    pub fn pop(&mut self) -> T {
        assert!(self.length > 0, "Called `pop()` on empty stack");
        unsafe {
            let ptr = self.data.offset((self.length - 1) as isize);
            let result = ptr::read(ptr);
            self.length -= 1;
            result
        }
    }

    pub fn push(&mut self, datum: T) {
        if self.length == self.capacity {
            let new_capacity = self.capacity * 2;
            self.resize(new_capacity);
        }

        assert!(self.length < self.capacity, "No space in stack; should have been resized");

        unsafe {
            let ptr = self.data.offset(self.length as isize);
            ptr::write(ptr, datum);
            self.length += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    fn resize(&mut self, capacity: usize) {
        // TODO
    }
}

#[inline]
fn allocate<U>(capacity: usize) -> *mut U {
    let elem_size = mem::size_of::<U>();
    unsafe {
        heap::allocate(capacity * elem_size, mem::align_of::<U>()) as *mut U
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.length {
                let ptr = self.data.offset(i as isize);
                ::std::intrinsics::drop_in_place(ptr);           
            }

            heap::deallocate(self.data as *mut u8,
                             self.capacity * mem::size_of::<T>(),
                             mem::align_of::<T>())
        }
    }
}

#[cfg(test)]
mod test {
    use super::Stack;

    #[test]
    fn empty() {
        let s = Stack::<usize>::new();
        assert!(s.len() == 0);

        let s = Stack::<usize>::with_capacity(42);
        assert!(s.len() == 0);
    }

    #[test]
    #[should_panic]
    fn empty_pop() {
        let mut s = Stack::<usize>::new();
        s.pop();
    }

    #[test]
    #[should_panic]
    fn empty_peek() {
        let s = Stack::<usize>::new();
        s.peek();
    }

    #[test]
    fn push_1() {
        let mut s = Stack::new();
        s.push(42);
        assert!(s.len() == 1);
        assert!(s.peek() == &42);
        assert!(s.pop() == 42);
        assert!(s.len() == 0);
    }

    #[test]
    fn push_many() {
        let mut s = Stack::new();
        for i in 0..32 {
            s.push(i);
        }
        assert!(s.len() == 32);

        for i in 0..32 {
            let i = 31 - i;
            assert!(s.peek() == &i);
            assert!(s.pop() == i);
            assert!(s.len() == i);
        }
        assert!(s.len() == 0);
    }

    // Test that the dtors of our contents get called.
    #[test]
    fn test_drop() {
        static mut DROP_COUNT: u32 = 0;
        struct TestDrop;
        impl Drop for TestDrop {
            fn drop(&mut self) {
                unsafe {
                    DROP_COUNT += 1;
                }
            }
        }

        {
            let mut s = Stack::new();
            for _ in 0..32 {
                s.push(TestDrop);
            }
            assert!(s.len() == 32);
        }
        unsafe {
            assert!(DROP_COUNT == 32);
        }
    }
}
