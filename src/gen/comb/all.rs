use std::marker::PhantomData;

use gen::Returns;
use gen::either::GenEither;
use gen::comb::race::{GenRace, Race};
use gen::comb::chain::{GenChain, Chain};
use gen::map::ret::{GenMapReturn, MapReturn};

pub struct Prepend<F>(F);

impl<F> Prepend<F> {
    pub fn new(f: F) -> Self {
        Prepend(f)
    }
}

impl<I, F> FnOnce<(I,)> for Prepend<F> {
    type Output = (F, I);

    extern "rust-call" fn call_once(self, (i,): (I,)) -> Self::Output {
        (self.0, i)
    }
}

pub struct Append<F>(F);

impl<F> Append<F> {
    pub fn new(f: F) -> Self {
        Append(f)
    }
}

impl<I, F> FnOnce<(I,)> for Append<F> {
    type Output = (I, F);

    extern "rust-call" fn call_once(self, (i,): (I,)) -> Self::Output {
        (i, self.0)
    }
}

pub struct ContinueRemaining<F, L>(PhantomData<(F, L)>);

impl<F, L> ContinueRemaining<F, L>
{
    fn new() -> Self {
        ContinueRemaining(PhantomData)
    }
}

impl<F, L> FnOnce<(GenEither<(F::Return, L), (F, L::Return)>,)> for ContinueRemaining<F, L>
    where F: Returns,
          L: Returns
{
    type Output = GenEither<GenMapReturn<L, Prepend<F::Return>>, GenMapReturn<F, Append<L::Return>>>;

    extern "rust-call" fn call_once(self,
                                    (results,): (GenEither<(F::Return, L), (F, L::Return)>,))
                                    -> Self::Output {
        match results {
            GenEither::Former((f, l)) => GenEither::Former(l.map_return(Prepend::new(f))),
            GenEither::Latter((f, l)) => GenEither::Latter(f.map_return(Append::new(l))),
        }
    }
}

pub type GenAll<F, L>= GenChain<GenRace<F, L>, ContinueRemaining<F, L>>;

pub trait All {
    fn all<L>(self, l: L) -> GenAll<Self, L> where Self: Sized + Returns, L: Returns
    {
        self.race(l).chain(ContinueRemaining::new())
    }
}

impl<F> All for F {}


#[cfg(test)]
mod tests {

    use gen::iter::wrap::Wrap;
    use gen::comb::all::All;
    use gen::map::ret::MapReturn;

    #[test]
    fn all() {
        let first = (0..5).wrap().map_return(|_| "first");
        let second = (0..10).wrap().map_return(|_| "second");

        let mut trace = vec![];
        let both = each!(first.all(second) => i in {
            trace.push(i);
        });

        assert_eq!(trace, [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 6, 7, 8, 9]);
        assert_eq!(both, ("first", "second"));
    }
}
