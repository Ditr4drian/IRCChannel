//! Side effects requested by `update`. The runtime is the only place that
//! touches IO while applying these.

use crate::irc::IrcCommand;

#[derive(Debug, Clone)]
pub enum AppCommand {
    Quit,
    Irc(IrcCommand),
    /// Batch helper.
    Batch(Vec<AppCommand>),
}

impl AppCommand {
    pub fn none() -> Vec<AppCommand> {
        Vec::new()
    }

    pub fn one(cmd: AppCommand) -> Vec<AppCommand> {
        vec![cmd]
    }
}
