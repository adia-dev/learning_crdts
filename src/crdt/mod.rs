use self::lww_register::State;

pub mod lww_register;

pub trait CRDT<T> {
    fn merge(&mut self, state: State<T>)
    where
        T: Clone + std::fmt::Display + std::fmt::Debug;
}
