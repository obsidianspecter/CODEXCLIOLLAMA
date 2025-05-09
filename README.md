
```markdown
# CodexCLI

A powerful AI-powered command-line interface that combines the capabilities of OpenAI's Codex with local code execution and development tools.

---

## 🚀 Features

- 🤖 **AI-powered code generation and execution**
- ⚡ **Local code execution with automatic dependency management**
- 🛠️ **Interactive development environment**
- 🔄 **Self-healing error handling**
- 🎨 **Beautiful terminal UI with animations**
- 🌐 **Support for multiple programming languages**:
  - Python (with virtual environment)
  - JavaScript/Node.js
  - TypeScript
  - Rust
  - HTML
  - Bash
- 🧰 **Development tools**:
  - React application creation and management
  - Local server hosting
  - Automatic package installation
  - Interactive code execution

---

## 📋 Prerequisites

- **Rust and Cargo** (latest stable version)
- **Python 3.x** (for Python code execution)
- **Node.js and npm** (for JavaScript/TypeScript/React)
- **WSL (Windows Subsystem for Linux)** for Windows users (for bash scripts)

---

## 🔧 Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/codex_cli.git
   cd codex_cli
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Install the binary**:
   - **Linux/macOS**
     ```bash
     sudo cp target/release/codex_cli /usr/local/bin/
     ```
   - **Windows (PowerShell)**
     ```powershell
     Copy-Item target/release/codex_cli.exe $env:USERPROFILE\AppData\Local\Microsoft\WindowsApps\
     ```

---

## 💻 Usage

### 🧑‍💻 Basic Usage

```bash
codex_cli
```

This starts the interactive CLI interface. You can:
- Type your questions or prompts directly
- Execute system commands by prefixing with `!` (e.g., `!ls`)
- Create and manage React applications
- Start local servers
- Execute code blocks from AI responses

---

### ⚙️ Command Line Options

```bash
codex_cli --help
```

Options:
- `--raw`: Disable fancy UI and animations
- `--workdir <DIR>`: Set the working directory for code execution

---

### 🧪 Examples

1. Ask the AI a question:
   ```
   > How do I create a React component?
   ```

2. Execute a system command:
   ```
   > !ls
   ```

3. Create a React application:
   ```
   > create-react-app
   ```

4. Start a local server:
   ```
   > start-server 8000
   ```

5. Run with a specific working directory:
   ```bash
   codex_cli --workdir ./my_project
   ```

---

## 🛠 Development

### 🔨 Building from Source

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone and build:
   ```bash
   git clone https://github.com/yourusername/codex_cli.git
   cd codex_cli
   cargo build
   ```

---

### ✅ Running Tests

```bash
cargo test
```

---

## 🤝 Contributing

1. Fork the repository  
2. Create your feature branch  
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. Commit your changes  
   ```bash
   git commit -m 'Add some amazing feature'
   ```
4. Push to the branch  
   ```bash
   git push origin feature/amazing-feature
   ```
5. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments
```

