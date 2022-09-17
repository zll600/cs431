//! Elimination-backoff stack.

mod base;
mod elim;
mod treiber_stack;

pub use base::Stack;

/// Elimination-backoff stack based on Treiber's stack.
pub type ElimStack<T> = base::ElimStack<T, treiber_stack::TreiberStack<T>>;

#[cfg(test)]
mod test {
    use super::*;
    use std::thread::scope;

    #[test]
    fn push() {
        let stack = ElimStack::default();

        scope(|scope| {
            for _ in 0..10 {
                scope.spawn(|| {
                    for i in 0..10_000 {
                        stack.push(i);
                        assert!(stack.pop().is_some());
                    }
                });
            }
        });

        assert!(stack.pop().is_none());
    }
}
