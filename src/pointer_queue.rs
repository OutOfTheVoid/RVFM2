use std::sync::mpsc;
pub struct PointerQueue {
    pub rx: Option<mpsc::Receiver<(u32, u32)>>,
    pub tx: mpsc::Sender<(u32, u32)>,
}

impl PointerQueue {
    pub fn take_rx(&mut self) -> mpsc::Receiver<(u32, u32)> {
        self.rx.take().unwrap()
    }

    pub fn make_tx(&self) -> mpsc::Sender<(u32, u32)>{
        self.tx.clone()
    }

    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            rx: Some(rx),
            tx
        }
    }
}

unsafe impl Send for PointerQueue {}
