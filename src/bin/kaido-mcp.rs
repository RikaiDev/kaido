// Kaido MCP Server
// Binary entry point for MCP (Model Context Protocol) server
//
// Usage:
//   kaido-mcp              # Start MCP server (stdio mode)
//   kaido-mcp --help       # Show help
//
// Claude Code Configuration (~/.claude.json):
// {
//   "mcpServers": {
//     "kaido": {
//       "command": "kaido-mcp"
//     }
//   }
// }

use clap::Parser;
use kaido::mcp::McpServer;

#[derive(Parser, Debug)]
#[command(name = "kaido-mcp")]
#[command(author = "RikaiDev")]
#[command(version)]
#[command(about = "Kaido MCP Server - Expose Kaido tools via Model Context Protocol")]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    }

    eprintln!("Kaido MCP Server v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("Waiting for MCP client connection...");

    let mut server = McpServer::new();

    if let Err(e) = server.run() {
        eprintln!("Server error: {e}");
        std::process::exit(1);
    }
}
