#![allow(dead_code)]  // silence unused‐function warnings

use clap::Parser;
use figlet_rs::FIGfont;
use owo_colors::OwoColorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
    env,
    time::Duration,
    thread,
};
use console::style;
use duct::cmd;

/// CodexCLI - AI at your terminal's service
#[derive(Parser)]
#[command(name = "codexcli", version = "1.0", author = "Anvin", about = "Ask AI anything")]
struct Args {
    /// Disable fancy UI and animations
    #[arg(long)]
    raw: bool,
    
    /// Set the working directory for code execution
    #[arg(long)]
    workdir: Option<String>,
}

fn print_banner() {
    let standard_font = FIGfont::standard()
        .unwrap_or_else(|_| FIGfont::from_content("").unwrap());
    let figure = standard_font.convert("CodexCLI").unwrap();
    println!("\n{}", figure.to_string().bright_blue().bold());
    println!("{}", style("AI at your terminal's service").dim());
    println!("{}", style("─────────────────────────────").dim());
    println!();
}

fn show_animated_message(message: &str, duration: Duration) {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    thread::sleep(duration);
    spinner.finish_and_clear();
}

fn show_spinner() -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Thinking...".to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    spinner
}

fn show_error_recovery(message: &str) {
    println!("\n{} {}", style("🔄 Attempting to recover:").bold().yellow(), style(message).white());
    show_animated_message("Recovering...", Duration::from_secs(1));
}

fn show_success(message: &str) {
    println!("\n{} {}", style("✅ Success:").bold().green(), style(message).white());
}

fn show_warning(message: &str) {
    println!("\n{} {}", style("⚠️ Warning:").bold().yellow(), style(message).white());
}

fn show_error(message: &str) {
    println!("\n{} {}", style("❌ Error:").bold().red(), style(message).red());
}

fn format_response(response: &str) -> String {
    let mut formatted = String::new();
    for line in response.lines() {
        if line.trim().is_empty() {
            formatted.push_str("\n");
        } else if line.trim().starts_with("```") {
            formatted.push_str(&format!("{}\n", style(line).cyan()));
        } else if line.trim().starts_with('#') {
            formatted.push_str(&format!("{}\n", style(line).yellow().bold()));
        } else if line.trim().starts_with('-') {
            formatted.push_str(&format!("{}\n", style(line).green()));
        } else {
            formatted.push_str(&format!("{}\n", style(line).white()));
        }
    }
    formatted
}

fn get_user_input() -> String {
    print!("{} ", style(">").bold().cyan());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim_end().to_string()
}

fn execute_command(command: &str) -> Result<String, String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn extract_code_blocks(response: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut in_block = false;
    let mut lang = String::new();

    for line in response.lines() {
        if line.trim().starts_with("```") {
            if in_block {
                blocks.push((lang.clone(), current.clone()));
                current.clear();
                lang.clear();
                in_block = false;
            } else {
                in_block = true;
                lang = line.trim().trim_start_matches("```").to_string();
            }
        } else if in_block {
            current.push_str(line);
            current.push('\n');
        }
    }

    blocks
}

fn setup_python_environment() -> Result<(), String> {
    show_animated_message("Setting up Python environment...", Duration::from_secs(1));
    
    if !Path::new("venv").exists() {
        let result = Command::new("python")
            .args(&["-m", "venv", "venv"])
            .output();

        match result {
            Ok(_output) => (),
            Err(_) => {
                show_error_recovery("Python not found, attempting to install...");
                // Try to install Python
                if cfg!(windows) {
                    Command::new("winget")
                        .args(&["install", "Python.Python"])
                        .output()
                        .map_err(|e| e.to_string())?;
                } else {
                    Command::new("sudo")
                        .args(&["apt-get", "install", "python3"])
                        .output()
                        .map_err(|e| e.to_string())?;
                }
                // Retry venv creation
                Command::new("python")
                    .args(&["-m", "venv", "venv"])
                    .output()
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    let python_path = if cfg!(windows) {
        "venv\\Scripts\\python.exe"
    } else {
        "venv/bin/python"
    };

    // Install common packages with retry logic
    let packages = ["pip", "setuptools", "wheel"];
    for package in packages.iter() {
        let mut attempts = 0;
        while attempts < 3 {
            let result = Command::new(python_path)
                .args(&["-m", "pip", "install", "--upgrade", package])
                .output();

            match result {
                Ok(_output) if _output.status.success() => break,
                Ok(_output) => {
                    show_warning(&format!("Failed to install {}, retrying...", package));
                    attempts += 1;
                    if attempts == 3 {
                        return Err(format!("Failed to install {} after 3 attempts", package));
                    }
                    thread::sleep(Duration::from_secs(1));
                }
                Err(e) => return Err(e.to_string()),
            }
        }
    }

    show_success("Python environment setup complete");
    Ok(())
}

fn install_python_package(package: &str) -> Result<(), String> {
    println!("{} {}", style("Installing Python package:").bold().yellow(), style(package).white());
    let python_path = if cfg!(windows) {
        "venv\\Scripts\\python.exe"
    } else {
        "venv/bin/python"
    };
    
    Command::new(python_path)
        .args(&["-m", "pip", "install", package])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn setup_node_environment() -> Result<(), String> {
    // Create package.json if it doesn't exist
    if !Path::new("package.json").exists() {
        println!("{}", style("Setting up Node.js environment...").bold().yellow());
        Command::new("npm")
            .args(&["init", "-y"])
            .output()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn install_node_package(package: &str) -> Result<(), String> {
    println!("{} {}", style("Installing Node package:").bold().yellow(), style(package).white());
    Command::new("npm")
        .args(&["install", package])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn handle_python_error(error: &str, code: &str) -> Result<String, String> {
    if error.contains("ModuleNotFoundError") {
        let pkg = error
            .split("No module named '")
            .nth(1)
            .and_then(|s| s.split('\'').next())
            .ok_or_else(|| "Could not extract package name".to_string())?;
        install_python_package(pkg)?;
        execute_code_block(code, "python", None)
    } else {
        Err(error.to_string())
    }
}

fn handle_node_error(error: &str, code: &str) -> Result<String, String> {
    if error.contains("Cannot find module") {
        let pkg = error
            .split("Cannot find module '")
            .nth(1)
            .and_then(|s| s.split('\'').next())
            .ok_or_else(|| "Could not extract package name".to_string())?;
        install_node_package(pkg)?;
        execute_code_block(code, "javascript", None)
    } else {
        Err(error.to_string())
    }
}

fn setup_react_environment(workdir: Option<&str>) -> Result<(), String> {
    println!("{}", style("Setting up React environment...").bold().yellow());
    
    // Create React app using create-react-app
    let mut cmd = Command::new("npx");
    cmd.args(&["create-react-app", "react-app"]);
    
    if let Some(dir) = workdir {
        cmd.current_dir(dir);
    }
    
    let status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        return Err("Failed to create React application".to_string());
    }

    Ok(())
}

fn start_react_server(workdir: Option<&str>) -> Result<String, String> {
    println!("{}", style("Starting React development server...").bold().yellow());
    
    let mut cmd = Command::new("npm");
    cmd.args(&["start"]);
    
    if let Some(dir) = workdir {
        cmd.current_dir(Path::new(dir).join("react-app"));
    } else {
        cmd.current_dir("react-app");
    }
    
    let _status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok("React development server started. Press Ctrl+C to stop.".to_string())
}

fn start_local_server(port: u16, workdir: Option<&str>) -> Result<String, String> {
    println!("{}", style("Starting local server...").bold().yellow());
    
    // Try Python's http.server first
    let mut cmd = Command::new("python");
    cmd.args(&["-m", "http.server", &port.to_string()]);
    
    if let Some(dir) = workdir {
        cmd.current_dir(dir);
    }
    
    let _status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(format!("Local server started on port {}. Press Ctrl+C to stop.", port))
}

fn execute_code_block(code: &str, language: &str, workdir: Option<&str>) -> Result<String, String> {
    // Check for special commands
    if code.trim() == "create-react-app" {
        return setup_react_environment(workdir)
            .and_then(|_| Ok("React application created successfully. Use 'npm start' to run the development server.".to_string()));
    }
    
    if code.trim() == "npm start" {
        return start_react_server(workdir);
    }
    
    if code.trim().starts_with("start-server") {
        let port = code
            .split_whitespace()
            .nth(1)
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8000);
        return start_local_server(port, workdir);
    }

    let ext = match language.to_lowercase().as_str() {
        "python" | "py" => "py",
        "javascript" | "js" => "js",
        "typescript" | "ts" => "ts",
        "rust" | "rs" => "rs",
        "bash" | "sh" => "sh",
        "html" => "html",
        _ => return Err(format!("Unsupported language: {}", language)),
    };

    // Create working directory if specified
    if let Some(dir) = workdir {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        env::set_current_dir(dir).map_err(|e| e.to_string())?;
    }

    let fname = format!("temp_code.{}", ext);
    File::create(&fname)
        .and_then(|mut f| f.write_all(code.as_bytes()))
        .map_err(|e| e.to_string())?;

    let result = || -> Result<String, String> {
        match ext {
            "py" => {
                // Setup Python environment
                setup_python_environment()?;
                
                let python_path = if cfg!(windows) {
                    "venv\\Scripts\\python.exe"
                } else {
                    "venv/bin/python"
                };

                // First try non-interactive mode
                let out = Command::new(python_path)
                    .arg(&fname)
                    .output()
                    .map_err(|e| e.to_string())?;

                if out.status.success() {
                    return Ok(String::from_utf8_lossy(&out.stdout).to_string());
                }

                let err = String::from_utf8_lossy(&out.stderr).to_string();

                // Handle missing modules
                if err.contains("ModuleNotFoundError") {
                    return handle_python_error(&err, code);
                }

                // For any input-related errors, switch to interactive mode
                if err.contains("input(") || err.contains("EOF") || err.contains("EOFError") {
                    println!(
                        "{}",
                        style("\nSwitching to interactive mode. Press Ctrl+C when done.").bold().yellow()
                    );
                    
                    let mut child = Command::new(python_path)
                        .arg(&fname)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()
                        .map_err(|e| e.to_string())?;

                    let status = child.wait().map_err(|e| e.to_string())?;
                    if status.success() {
                        Ok(String::new())
                    } else {
                        Err(format!("Python exited with status: {}", status))
                    }
                } else {
                    Err(err)
                }
            }
            "js" => {
                // Setup Node.js environment
                setup_node_environment()?;
                
                let out = Command::new("node")
                    .arg(&fname)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .map_err(|e| e.to_string())?;
                
                if out.success() {
                    Ok(String::new())
                } else {
                    let err = handle_node_error("", code)?;
                    if err.is_empty() {
                        Ok(String::new())
                    } else {
                        Err(format!("Node.js exited with status: {}", out))
                    }
                }
            }
            "ts" => {
                setup_node_environment()?;
                install_node_package("typescript")?;
                install_node_package("ts-node")?;
                
                let out = Command::new("npx")
                    .args(&["ts-node", &fname])
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .map_err(|e| e.to_string())?;

                if out.success() {
                    Ok(String::new())
                } else {
                    Err(format!("TypeScript execution failed with status: {}", out))
                }
            }
            "rs" => {
                let out = Command::new("rustc")
                    .arg(&fname)
                    .output()
                    .map_err(|e| e.to_string())?;
                
                if !out.status.success() {
                    return Err(String::from_utf8_lossy(&out.stderr).to_string());
                }

                let binary = if cfg!(windows) {
                    "temp_code.exe"
                } else {
                    "./temp_code"
                };

                let status = Command::new(binary)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .map_err(|e| e.to_string())?;

                if status.success() {
                    Ok(String::new())
                } else {
                    Err(format!("Rust program exited with status: {}", status))
                }
            }
            "sh" => {
                let mut cmd = if cfg!(windows) {
                    let mut c = Command::new("wsl");
                    c.args(&["bash", "-c", &format!("bash {}", fname)]);
                    c
                } else {
                    let mut c = Command::new("bash");
                    c.arg(&fname);
                    c
                };

                let status = cmd
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .map_err(|e| e.to_string())?;

                if status.success() {
                    Ok(String::new())
                } else {
                    Err(format!("Bash script exited with status: {}", status))
                }
            }
            "html" => {
                println!("{}", style("Opening HTML in default browser...").bold().yellow());
                let browser_cmd = if cfg!(windows) {
                    Command::new("cmd")
                        .args(&["/C", "start", &fname])
                        .status()
                        .map_err(|e| e.to_string())?
                } else if cfg!(target_os = "macos") {
                    Command::new("open")
                        .arg(&fname)
                        .status()
                        .map_err(|e| e.to_string())?
                } else {
                    Command::new("xdg-open")
                        .arg(&fname)
                        .status()
                        .map_err(|e| e.to_string())?
                };

                if browser_cmd.success() {
                    Ok(format!("HTML file opened in browser: {}", fname))
                } else {
                    Err("Failed to open HTML file in browser".to_string())
                }
            }
            _ => Err(format!("Unsupported language: {}", ext)),
        }
    }();

    // Clean up
    if ext != "html" {  // Don't delete HTML files immediately as they're being viewed
        let _ = std::fs::remove_file(&fname);
    }
    if ext == "rs" {
        let _ = std::fs::remove_file(if cfg!(windows) { "temp_code.exe" } else { "temp_code" });
    }

    // Reset working directory
    if workdir.is_some() {
        env::set_current_dir("..").map_err(|e| e.to_string())?;
    }

    result
}

fn process_prompt(prompt: &str, raw: bool, workdir: Option<&str>) {
    if prompt.starts_with('!') {
        let c = &prompt[1..].trim();
        if !raw {
            println!("{} {}", style("Executing command:").bold().yellow(), style(c).white());
        }
        match execute_command(c) {
            Ok(o) => {
                if !raw {
                    println!("\n{}{}", style("Command output:\n").bold().green(), style("─────────────────────────────\n").dim());
                    println!("{}", o);
                    println!("{}", style("─────────────────────────────").dim());
                } else {
                    println!("{}", o);
                }
            }
            Err(e) => {
                show_error(&e);
                show_error_recovery("Attempting to fix the command...");
                // Try to fix common command issues
                if let Ok(fixed) = fix_command(c) {
                    show_warning(&format!("Trying fixed command: {}", fixed));
                    match execute_command(&fixed) {
                        Ok(o) => {
                            show_success("Command fixed and executed successfully");
                            println!("{}", o);
                        }
                        Err(e) => show_error(&e),
                    }
                }
            }
        }
        return;
    }

    // Check for special commands in the prompt
    if prompt.trim() == "create-react-app" {
        match setup_react_environment(workdir) {
            Ok(_) => println!("\n{}", style("React application created successfully. Use 'npm start' to run the development server.").bold().green()),
            Err(e) => println!("\n{} {}", style("Error:").bold().red(), style(e).red()),
        }
        return;
    }

    if prompt.trim() == "npm start" {
        match start_react_server(workdir) {
            Ok(msg) => println!("\n{}", style(msg).bold().green()),
            Err(e) => println!("\n{} {}", style("Error:").bold().red(), style(e).red()),
        }
        return;
    }

    if prompt.trim().starts_with("start-server") {
        let port = prompt
            .split_whitespace()
            .nth(1)
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8000);
        match start_local_server(port, workdir) {
            Ok(msg) => println!("\n{}", style(msg).bold().green()),
            Err(e) => println!("\n{} {}", style("Error:").bold().red(), style(e).red()),
        }
        return;
    }

    if !raw {
        println!("{} {}", style("🤖 Prompt:").bold().cyan(), style(prompt).white());
        println!();
    }

    let spinner = if raw { None } else { Some(show_spinner()) };
    let ai = cmd!("ollama", "run", "llama3.2")
        .stdin_bytes(prompt)
        .read();
    if let Some(sp) = spinner {
        sp.finish_and_clear();
    }

    match ai {
        Ok(output) => {
            if !raw {
                println!("\n{}{}", style("🧠 AI Response:\n").bold().cyan(), style("─────────────────────────────\n").dim());
                println!("{}", format_response(&output));
                println!("{}", style("─────────────────────────────").dim());

                let blocks = extract_code_blocks(&output);
                if !blocks.is_empty() {
                    println!("\n{} (y/n)", style("Found code blocks. Execute them?").bold().yellow());
                    let mut ans = String::new();
                    io::stdin().read_line(&mut ans).unwrap();
                    if ans.trim().eq_ignore_ascii_case("y") {
                        for (lang, code) in blocks {
                            println!(
                                "\n{} {} {}",
                                style("Executing").bold().green(),
                                style(&lang).bold().cyan(),
                                style("code block:").bold().green()
                            );
                            match execute_code_block(&code, &lang, workdir) {
                                Ok(res) => {
                                    if !res.is_empty() {
                                        println!(
                                            "\n{}{}",
                                            style("Execution result:\n").bold().green(),
                                            style("─────────────────────────────").dim()
                                        );
                                        println!("{}", res);
                                        println!("{}", style("─────────────────────────────").dim());
                                    }
                                }
                                Err(err) => println!("\n{} {}", style("Execution error:").bold().red(), style(err).red()),
                            }
                        }
                    }
                }
            } else {
                println!("{}", output);
            }
        }
        Err(e) => {
            println!("\n{} {}", style("Error:").bold().red(), style(e).red());
            println!("{}", style("Please try again or Ctrl+C to exit").dim());
        }
    }
}

fn fix_command(command: &str) -> Result<String, String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    // Common command fixes
    let fixed = match parts[0] {
        "python" if cfg!(windows) => "py",
        "python3" if cfg!(windows) => "py",
        "pip" if cfg!(windows) => "py -m pip",
        "npm" if !Command::new("npm").output().is_ok() => "npx",
        _ => parts[0],
    };

    let mut fixed_parts = vec![fixed];
    fixed_parts.extend_from_slice(&parts[1..]);
    Ok(fixed_parts.join(" "))
}

fn main() {
    let args = Args::parse();
    
    if !args.raw {
        print_banner();
        println!("{}", style("Type your prompt and hit Enter; Ctrl+C to exit.").dim());
        println!("{}", style("For system commands, prefix with ! (e.g. !ls)").dim());
        println!("{}", style("─────────────────────────────").dim());
        
        // Show initial setup animation
        show_animated_message("Initializing CodexCLI...", Duration::from_secs(1));
    }

    loop {
        let prompt = get_user_input();
        if prompt.is_empty() {
            continue;
        }
        process_prompt(&prompt, args.raw, args.workdir.as_deref());
    }
}
