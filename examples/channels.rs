use std::sync::{Arc, Mutex};

use learning_crdts::crdt::{
    lww_register::{LWWRegister, State},
    CRDT,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(Debug, Clone)]
pub struct TextEditor {
    pub content: String,
}

impl std::fmt::Display for TextEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TextEditor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn insert(&mut self, position: usize, content: &str) {
        self.content.insert_str(position, content);
    }

    pub fn append(&mut self, content: &str) {
        self.content.push_str(content);
    }

    pub fn erase(&mut self, from: usize, to: isize) {
        if to >= 0 && to > (from as isize) {
            self.content.replace_range(from..(to as usize), "");
        } else if to == -1 {
            self.content.replace_range(from.., "");
        } else {
            println!("Could not erase the content at the given position.");
        }
    }
}

type Responder<T> = oneshot::Sender<StateFrame<T>>;

#[derive(Debug)]
pub struct StateFrame<T>
where
    T: std::fmt::Display + Clone + std::fmt::Debug,
{
    pub state: State<T>,
    pub resp: Option<Responder<T>>,
}

impl<T> StateFrame<T>
where
    T: std::fmt::Display + Clone + std::fmt::Debug,
{
    pub fn new(state: State<T>, resp: Option<Responder<T>>) -> Self {
        Self { state, resp }
    }
}

fn generate_random_string(len: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(len)
        .collect()
}

#[tokio::main]
async fn main() {
    let mut text_editor = TextEditor::new();
    text_editor.append("Hello World !");
    let main_lww = Arc::new(Mutex::new(LWWRegister::new(State(None, 0, text_editor))));
    let tx_main_lww = main_lww.clone();
    let count = Arc::new(Mutex::new(0));

    let (tx, mut rx) = mpsc::channel::<State<TextEditor>>(100);
    let (b_tx, _) = broadcast::channel::<State<TextEditor>>(200);
    let b_tx1 = b_tx.clone();

    tokio::spawn(async move {
        for _ in 0..10 {
            let text_editor = tx_main_lww.lock().unwrap().state().2.clone();
            let tx = tx.clone();
            let b_tx = b_tx1.clone();

            tokio::spawn(async move {
                let state = State(Some(uuid::Uuid::new_v4().to_string()), 0, text_editor);
                let mut lww = LWWRegister::new(state);

                for _ in 0..100 {
                    let mut value = lww.value().clone();
                    let mut b_rx = b_tx.subscribe();

                    let random_string = generate_random_string(1);
                    value.append(random_string.as_str());

                    lww.set(value);

                    _ = tx.send(lww.state().clone()).await;

                    match b_rx.recv().await {
                        Ok(state) => {
                            lww.merge(state);
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to receive a response from the channel; err = {:?}",
                                e
                            );
                        }
                    }
                }
            });
        }
    });

    let manager = tokio::spawn(async move {
        let b_tx = b_tx.clone();

        while let Some(state) = rx.recv().await {
            let mut lww = main_lww.lock().unwrap();

            let mut lock = count.lock().unwrap();
            *lock += 1;

            lww.merge(state);

            _ = b_tx.send(lww.state().clone());
        }

        println!("LWW: {:?}", main_lww);
    });

    manager.await.unwrap();
}
