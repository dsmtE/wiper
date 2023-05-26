use crate::app::Arguments;

pub mod handler;
// For this dummy application we only need two IO event
#[derive(Debug, Clone)]
pub enum IoEvent {
    InitializeFromArgs(Arguments),
    DeleteEntries(Vec<walkdir::DirEntry>),
}
