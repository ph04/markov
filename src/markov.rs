use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use indexmap::set::IndexSet;
use rand::Rng;

pub trait State: Hash + Eq + IntoIterator {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NGram<const N: usize> {
    ngram: [u8; N],
}

impl<const N: usize> State for NGram<N> {}

impl<const N: usize> IntoIterator for NGram<N> {
    type Item = u8;
    type IntoIter = std::array::IntoIter<u8, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.ngram.into_iter()
    }
}

impl<const N: usize> ToString for NGram<N> {
    fn to_string(&self) -> String {
        String::from_utf8(self.ngram.to_vec()).unwrap()
    }
}

impl<const N: usize> FromIterator<NGram<N>> for String {
    fn from_iter<T: IntoIterator<Item = NGram<N>>>(iter: T) -> Self {
        iter.into_iter().collect::<String>()
    }
}

impl<const N: usize> From<[u8; N]> for NGram<N> {
    fn from(ngram: [u8; N]) -> Self {
        Self { ngram }
    }
}

pub struct MarkovIter<'m, S: State, const N: usize, R: Rng + ?Sized + Clone> {
    states: &'m HashMap<S, IndexSet<u8>>,
    prev_state: Option<S>,
    rng: R
}

impl<'m, S: State, const N: usize, R: Rng + ?Sized + Clone> MarkovIter<'m, S, N, R> {
    pub fn new(states: &'m HashMap<S, IndexSet<u8>>, first_state: Option<S>, rng: R) -> Self {
        Self { states, prev_state: first_state, rng }
    }
}

impl<'m, S, const N: usize, R> Iterator for MarkovIter<'m, S, N, R>
where
    S: State + Clone + Copy,
    R: Rng + ?Sized + Clone,
    [u8; N]: Into<S>,
    <S as IntoIterator>::Item: Into<u8>,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(prev) = self.prev_state {
            if let Some(next_states) = self.states.get(&prev) {
                let random_idx = self.rng.gen_range(0..next_states.len());

                let next_char = next_states[random_idx];

                let mut new_prev_state = [0; N];

                new_prev_state
                    .iter_mut()
                    .zip(prev.into_iter().skip(1))
                    .for_each(|(new_c, old_c)| *new_c = old_c.into());

                new_prev_state[N - 1] = next_char;

                self.prev_state = Some(new_prev_state.into());
                
                Some(next_char)
            } else {
                self.prev_state = None;

                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct MarkovStates<'m, S: State, const N: usize, R: Rng + ?Sized + Clone> {
    states: HashMap<S, IndexSet<u8>>,
    _marker: PhantomData<&'m S>,
    rng: R,
}

impl<'m, S, const N: usize, R> IntoIterator for &'m MarkovStates<'m, S, N, R>
where
    S: State + Clone + Copy,
    R: Rng + ?Sized + Clone,
    [u8; N]: Into<S>,
    <S as IntoIterator>::Item: Into<u8>,
{
    type Item = u8;
    type IntoIter = MarkovIter<'m, S, N, R>;

    fn into_iter(self) -> MarkovIter<'m, S, N, R> {
        MarkovIter::new(&self.states, self.states.keys().next().copied(), self.rng.clone())
    }
}

impl<'m, S: State + From<[u8; N]>, const N: usize, R: Rng + ?Sized + Clone> MarkovStates<'m, S, N, R> {
    pub fn from_random(chars: &[u8], rng: R) -> Self {
        let mut states: HashMap<S, IndexSet<u8>> = HashMap::new();

        chars
            .windows(N + 1)
            .for_each(|slice| {
                let mut curr = [0; N];

                let mut bytes = slice.iter();

                curr
                    .iter_mut()
                    .for_each(|byte| *byte = *bytes.next().unwrap());

                let curr_state = S::from(curr);
                let next_char = slice[N];

                if let Some(next_chars) = states.get_mut(&curr_state) {
                    next_chars.insert(next_char);
                } else {
                    states.insert(curr_state, IndexSet::from([next_char]));
                }
            });

        Self { states, _marker: PhantomData::default(), rng }
    }
}
