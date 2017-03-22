use co::{Coroutine, CoResult};

pub struct CoWrap<T>(T);

impl<Iter> Coroutine for CoWrap<Iter>
    where Iter: Iterator
{
    type Yield = Iter::Item;
    type Return = Iter;
    type Continue = Self;

    fn next(self) -> CoResult<Self> {
        let mut i = self.0;
        match i.next() {
            Some(item) => CoResult::Yield(item, CoWrap(i)),
            _ => CoResult::Return(i),
        }
    }
}

pub trait Wrap<I>: Sized {
    fn wrap(self) -> CoWrap<Self> {
        CoWrap(self)
    }
}

impl<I> Wrap<I> for I where I: Iterator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_iterator() {
        let mut cnt = 1;
        let message = each!((1..10).wrap() => i in {
            assert_eq!(i, cnt);
            cnt += 1;
        } then with mut iter in {
            assert_eq!(iter.next(), None);
            assert_eq!(cnt, 10);
            42
        });
        assert_eq!(message, 42);
    }
}
