use super::CRDT;

#[derive(Debug, Clone)]
pub struct State<T: std::fmt::Display + Clone + std::fmt::Debug>(
    pub Option<String>,
    pub u64,
    pub T,
);

#[derive(Debug, Clone)]
pub struct LWWRegister<T>
where
    T: std::fmt::Display + Clone + std::fmt::Debug,
{
    id: String,
    state: State<T>,
}

impl<T> LWWRegister<T>
where
    T: std::fmt::Display + Clone + std::fmt::Debug,
{
    pub fn new(state: State<T>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state,
        }
    }

    pub fn state(&self) -> &State<T> {
        return &self.state;
    }

    pub fn value(&self) -> &T {
        &self.state.2
    }

    pub fn set(&mut self, value: T) {
        self.state = State(Some(self.id.clone()), self.state.1 + 1, value);
    }
}

impl<T> CRDT<T> for LWWRegister<T>
where
    T: std::fmt::Display + Clone + std::fmt::Debug,
{
    fn merge(&mut self, state: State<T>) {
        let State(peer_id, peer_timestamp, ..) = &state;
        let State(local_id, local_timestamp, ..) = &self.state;

        if *local_timestamp >= *peer_timestamp {
            return;
        }

        if *local_timestamp == *peer_timestamp && *local_id > *peer_id {
            return;
        }

        self.state = state;
    }
}
