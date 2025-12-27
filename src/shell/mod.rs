pub mod builtins;
pub mod history;
pub mod kaido_shell;
pub mod prompt;
pub mod pty;
pub mod repl;
pub mod signals;

pub use builtins::{parse_builtin, Builtin, BuiltinResult, ShellEnvironment};
pub use history::{default_history_path, ensure_history_dir, HistoryConfig};
pub use kaido_shell::{KaidoShell, ShellConfig};
pub use prompt::PromptBuilder;
pub use pty::{PtyExecutionResult, PtyExecutor};
pub use repl::run_agent_repl;
pub use signals::{SignalHandler, TerminalSize};
