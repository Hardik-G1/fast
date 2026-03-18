// Author : Hardik
// Package Name: fast
// Package Version: 0.1.0
mod info;
mod fastcd;
mod hist;
mod alias;
fn print_verbose_help(){
    print_help();
    eprintln!("Install:");
    eprintln!("  PowerShell:  .\\install.ps1       (from source, needs Cargo)");
    eprintln!("               .\\install-bin.ps1   (pre-built exe, no Cargo)");
    eprintln!("  Bash/Zsh:    bash install.sh     (from source, needs Cargo)");
}
fn print_help() {
    eprintln!("fast-tools v{}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  fast              Open file browser (use fcd to cd on Enter)");
    eprintln!("  fast hist         Search & run commands from folder history (fh)");
    eprintln!("  fast top          System monitor (ftop)");
    eprintln!("  fast alias        TUI alias picker");
    eprintln!("  fast alias add <name> <cmd>   Save an alias");
    eprintln!("  fast alias rm <name>          Remove an alias");
    eprintln!("  fast alias list               List all aliases");
    eprintln!("  fast alias run <name>         Run alias (used by f <name>)");
    eprintln!("  fast help / --help            Show this help");
    eprintln!();
    eprintln!("Shell shortcuts (after install):");
    eprintln!("  fcd              File browser, cd into selected directory");
    eprintln!("  fh               Folder-specific command history picker");
    eprintln!("  ftop             System monitor");
    eprintln!("  f <alias>        Run a saved alias");
    eprintln!();
    
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(|s| s.as_str()) {
        Some("help") | Some("--help") | Some("-h") => { print_help(); Ok(()) },
        Some("hist")  => hist::run(&args[1..]),
        Some("alias") => alias::run(&args[1..]),
        Some("top")   => info::get::ftop().map_err(|e| e.into()),
        Some("cd")    => fastcd::run(),
        _             => { print_verbose_help(); Ok(()) } 
    };
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
