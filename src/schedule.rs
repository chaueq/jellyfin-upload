
use std::{sync::mpsc::{channel, Sender}, thread::{self, sleep}, time::{Duration, Instant}};

use crate::module::{Module, ModuleMgmtSignal};

#[allow(unstable_name_collisions)]
pub fn start(http: Sender<ModuleMgmtSignal>) -> Module {
    let (mgmt_sender, mgmt_receiver) = channel::<ModuleMgmtSignal>();
    
    let handle = thread::spawn(move || {
        let mut mgmt_timers: Vec<Timer<ModuleMgmtSignal>> = Vec::new();

        mgmt_timers.push(Timer::new(
            http, 
            Duration::from_minutes(30), 
            None,
            ModuleMgmtSignal::Refresh
        ));

        loop {
            if let Ok(req) = mgmt_receiver.try_recv() {
                match req {
                    ModuleMgmtSignal::Stop => {
                        break;
                    }
                    _ => {}
                }
            }
            else {
                for timer in &mut mgmt_timers {
                    timer.manage();
                }
                sleep(Duration::from_millis(100));
            }

        }
    });

    Module::new(handle, mgmt_sender)
}

struct Timer<ReqType> {
    sender: Sender<ReqType>,
    interval: Duration,
    offset: Option<Duration>,
    signal: ReqType,
    last_executed: Instant
}

impl<ReqType: Clone> Timer<ReqType> {
    pub fn new(sender: Sender<ReqType>, interval: Duration, offset: Option<Duration> , signal: ReqType) -> Self {
        Self {
            sender,
            interval,
            offset,
            signal,
            last_executed: Instant::now()
        }
    }

    pub fn manage(&mut self) {
        match self.offset {
            Some(offset) => {
                if self.last_executed.elapsed() > offset {
                    self.last_executed = Instant::now();
                    self.offset = None;
                }
            }
            None => {
                if self.last_executed.elapsed() > self.interval {
                    self.execute();
                }
            }
        }
    }

    fn execute(&mut self) {
        let _ = self.sender.send(self.signal.clone());
        self.last_executed = Instant::now();
    }
}

pub trait DurationPlus {
    fn from_minutes(mins: u64) -> Duration;
}

impl DurationPlus for Duration {
    fn from_minutes(mins: u64) -> Duration {
        Duration::from_secs(mins * 60)
    }
}