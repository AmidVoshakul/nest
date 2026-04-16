//! Nest CLI banner

pub fn print_banner() {
    println!("\x1b[38;5;75m");
    println!(" ███╗   ██╗███████╗███████╗████████╗");
    println!(" ████╗  ██║██╔════╝██╔════╝╚══██╔══╝");
    println!(" ██╔██╗ ██║█████╗  ███████╗   ██║   ");
    println!(" ██║╚██╗██║██╔══╝  ╚════██║   ██║   ");
    println!(" ██║ ╚████║███████╗███████║   ██║   ");
    println!(" ╚═╝  ╚═══╝╚══════╝╚══════╝   ╚═╝   ");
    println!("\x1b[0m");
    println!("\x1b[90m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
    println!("  \x1b[1mSecure Hypervisor for Autonomous AI Agents\x1b[0m");
    println!(
        "  \x1b[38;5;240mVersion {}\x1b[0m",
        env!("CARGO_PKG_VERSION")
    );
    println!("\x1b[90m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
    println!();
}
