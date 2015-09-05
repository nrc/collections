// Treiber's stack - a lock free, concurrent stack
// TODO make it concurrent and do the fancy lock free stuff.

pub struct Stack<T> {
    len: usize,
    head: Option<Box<Node<T>>>,
}

struct Node<T> {
    datum: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack {
            len: 0,
            head: None,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, datum: T) {
        self.len += 1;
        self.head = Some(Box::new(Node {
            datum: datum,
            next: self.head.take()
        }));
    }

    pub fn pop(&mut self) -> T {
        assert!(self.len > 0);
        self.len -= 1;
        let Node { datum, next } = *self.head.take().unwrap();
        self.head = next;
        datum
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn test_push_pop_empty() {
        let mut s: Stack<usize> = Stack::new();
        s.pop();
    }

    #[test]
    fn test_push_pop() {
        let mut s = Stack::new();
        assert!(s.len() == 0);
        s.push(42);
        assert!(s.len() == 1);
        assert!(s.pop() == 42);
        assert!(s.len() == 0);
    }

    #[test]
    fn test_push_pop_3() {
        let mut s = Stack::new();
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
}
