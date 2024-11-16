use std::{sync::mpsc::Sender, thread};

pub struct Module {
    handle: thread::JoinHandle<()>,
    sender_mgmt: Sender<ModuleMgmtSignal>,
}

impl Module {
    pub fn new(handle: thread::JoinHandle<()>, sender_mgmt: Sender<ModuleMgmtSignal>) -> Self {
        Self {
            handle,
            sender_mgmt
        }
    }

    pub fn join(self) {
        let _ = self.handle.join();
    }

    pub fn stop(&self) {
        let _ = self.sender_mgmt.send(ModuleMgmtSignal::Stop);
    }

    pub fn clone_mgmt_sender(&self) -> Sender<ModuleMgmtSignal> {
        self.sender_mgmt.clone()
    }
}

#[derive(PartialEq, Clone)]
pub enum ModuleMgmtSignal {
    Stop,
    Refresh
}