//! TCB manager
use super::{ThreadControlBlock};
use spin::Mutex;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;

pub struct Manager {
    ready_queue: VecDeque<Arc<ThreadControlBlock>>,
}

/// A simple FIFO scheduler.
impl Manager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    ///Add a task to `TaskManager`
    pub fn add(&mut self, task: Arc<ThreadControlBlock>) {
        self.ready_queue.push_back(task);
    }
    ///Remove the first task and return it,or `None` if `TaskManager` is empty
    pub fn fetch(&mut self) -> Option<Arc<ThreadControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref MANAGER: Mutex<Manager> = Mutex::new(Manager::new());
}
///Interface offered to add task
pub fn add_task(task: Arc<ThreadControlBlock>) {
    MANAGER.lock().add(task);
}
///Interface offered to pop the first task
pub fn fetch_task() -> Option<Arc<ThreadControlBlock>> {
    MANAGER.lock().fetch()
}
