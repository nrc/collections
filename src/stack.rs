// A stack implemented using an array (well, not even a proper array, but pointer
// offsets).

use std::{mem, ptr, fmt};
use std::rt::heap;

pub struct Stack<T> {
    // Invariants:
    //   length <= capacity
    //   data points to length valid elements.
    data: ptr::Unique<T>,
    length: usize,
    capacity: usize,
}

// TODO
// ZSTs
// overflow of length

impl<T> Stack<T> {
    pub fn with_capacity(capacity: usize) -> Stack<T> {
        Stack {
            data: unsafe { ptr::Unique::new(if capacity == 0 {
                    ptr::null_mut()
                } else {
                    allocate::<T>(capacity)                    
                })},
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
            let new_capacity = self.new_capacity();
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

    fn new_capacity(&self) -> usize {
        if self.capacity == 0 {
            4
        } else {
            self.capacity * 2
        }
    }

    fn resize(&mut self, capacity: usize) {
        if capacity == self.capacity {
            // Nothing to do.
            return;
        }

        if capacity < self.capacity {
            panic!("Shrinking is not yet supported");
        }

        // If capacity was 0, then we are allocating for the first time,
        // otherwise we reallocate.
        if self.capacity == 0 {
            unsafe {
                self.data = ptr::Unique::new(allocate(capacity));
                self.capacity = capacity;
            }

            return;
        }

        let size = capacity * mem::size_of::<T>();
        assert!(size > 0);

        unsafe {
            let new_data = if self.capacity == 0 {
                allocate(size)
            } else {
                heap::reallocate(self.data.get_mut() as *mut T as *mut u8,
                                 self.capacity,
                                 size,
                                 mem::align_of::<T>())
            };

            if new_data.is_null() {
                panic!("Could not resize Stack.");
            }
            self.data = ptr::Unique::new(new_data as *mut T);
            self.capacity = capacity;
        }
    }
}

impl<T> fmt::Debug for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Stack {{ length: {}, capacity: {} }}", self.length, self.capacity)
    }
}

impl<T: fmt::Display> fmt::Display for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "["));
        let mut first = true;
        for i in 0..self.length {
            if first {
                first = false;
            } else {
                try!(write!(f, ", "));
            }

            unsafe {
                let ptr = self.data.offset(i as isize);
                try!(write!(f, "{}", *ptr));
            }
        }
        write!(f, "]")
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.length {
                let ptr = self.data.offset(i as isize);
                ::std::intrinsics::drop_in_place(ptr);           
            }

            if self.capacity > 0 {
                heap::deallocate(self.data.get_mut() as *mut T as *mut u8,
                                 self.capacity * mem::size_of::<T>(),
                                 mem::align_of::<T>())
            }
        }
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Stack<T> {
        Stack::new()
    }
}

impl<T:Clone> Clone for Stack<T> {
    fn clone(&self) -> Stack<T> {
        let mut result = Stack::with_capacity(self.capacity);
        unsafe {
            for i in 0..self.length {
                let result_ptr = result.data.offset(i as isize);
                let ptr = self.data.offset(i as isize);
                *result_ptr = (*ptr).clone();
            }
            result.length = self.length;
        }
        result
    }
}

impl<T: PartialEq> PartialEq for Stack<T> {
    fn eq(&self, other: &Stack<T>) -> bool {
        if self.length != other.length {
            return false;
        }

        unsafe {
            for i in 0..self.length {
                let other_ptr = other.data.offset(i as isize);
                let ptr = self.data.offset(i as isize);
                if *other_ptr != *ptr {
                    return false;
                }
            }
        }

        true        
    }
}

impl<T: Eq> Eq for Stack<T> {}


#[inline]
fn allocate<U>(capacity: usize) -> *mut U {
    let size = capacity * mem::size_of::<U>();
    assert!(size > 0);
    unsafe {
        heap::allocate(size, mem::align_of::<U>()) as *mut U
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

    #[test]
    fn push_with_resize() {
        let mut s = Stack::with_capacity(4);
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

    #[test]
    fn test_print() {
        let mut s = Stack::with_capacity(8);
        for i in 0..8 {
            s.push(i);
        }
        assert!(s.len() == 8);
        println!("Debug: {:?}", s);
        println!("Display: {}", s);
        // FIXME: should test that at least Display gives the expected result.
    }

    // Test the Default, Eq, and Clone traits.
    #[test]
    fn test_traits() {
        let mut s1 = Stack::default();
        s1.push(42);
        s1.push(43);
        let s2 = s1.clone();
        assert!(s1 == s2);
        s1.pop();
        assert!(s1 != s2);        
    }

    #[test]
    fn test_zero_capacity() {
        let mut s = Stack::<i32>::with_capacity(0);
        s.push(42);
        assert!(s.pop() == 42);
        assert!(s.len() == 0);
    }
}
