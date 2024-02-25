use std::{mem, sync::Mutex};

use crate::{screens::GUIScreen, world::{ChunkBlockMetadata, World}, GameConfig};

pub enum Task {
    ExitGame,
    // Custom(Box<dyn Fn() -> () + Send>),
    OpenScreenCentered(Box<dyn GUIScreen>),
    CloseScreen,
    WorldUpdateBlock(&'static (dyn Fn(ChunkBlockMetadata, &mut World) -> () + Sync), ChunkBlockMetadata),
    CloseWorld,
    OpenWorld(String),
    CreateWorld,
    __OpnWrld(World, GameConfig),
}

static TASKS: Mutex<Vec<Task>> = Mutex::new(Vec::new());

pub fn get_tasks() -> Vec<Task> {
    mem::replace(&mut TASKS.lock().unwrap(), Vec::new())
}

// pub fn schedule_function(task: Box<dyn Fn() -> () + Send>) {
//     TASKS.lock().unwrap().push(Task::Custom(task));
// }

pub fn schedule_task(task: Task) {
    TASKS.lock().unwrap().push(task);
}