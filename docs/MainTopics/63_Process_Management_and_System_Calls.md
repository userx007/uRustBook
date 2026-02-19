# Process Management and System Calls in Rust

1. **`std::process`** — exit, abort, and PID
2. **`Command` builder** — execution, output capture, background spawning
3. **Piping** — stdin/stdout chaining, `null`, `inherit`, and Unix-style pipe chains
4. **Killing processes** — `child.kill()` + `wait`
5. **Environment variables** — reading, setting, clearing, compile-time macros
6. **CLI arguments** — `env::args()`
7. **Signal handling** — `signal-hook` iterator and atomic flag patterns
8. **`libc` syscall wrappers** — `fork`/`exec`, `kill`, `nice`, `dup2`, and raw `syscall`
9. **Working directory** — `current_dir`, `set_current_dir`, per-child `current_dir`
10. **Putting it all together** — a streaming process runner example


Rust provides robust facilities for process management through `std::process`, and for lower-level system interaction via signal handling, environment variables, and libc wrappers. This document covers all these areas with practical examples.

---

## 1. `std::process` — The Core Module

The `std::process` module exposes the current process's lifecycle and controls, plus the `Command` builder for spawning child processes.

### 1.1 Exiting the Current Process

```rust
use std::process;

fn main() {
    println!("About to exit");
    process::exit(0); // Exit code 0 = success; non-zero = failure
}
```

> `process::exit` terminates immediately — destructors are **not** run. For graceful teardown, prefer returning from `main` or using `drop` guards.

### 1.2 Aborting the Process

```rust
fn main() {
    // Triggers an immediate abnormal termination (SIGABRT on Unix).
    std::process::abort();
}
```

### 1.3 Accessing the Current Process ID

```rust
fn main() {
    let pid = std::process::id();
    println!("Current PID: {}", pid);
}
```

---

## 2. Spawning Child Processes with `Command`

`std::process::Command` is a builder pattern for configuring and launching child processes.

### 2.1 Simple Execution

```rust
use std::process::Command;

fn main() {
    let status = Command::new("echo")
        .arg("Hello from child process!")
        .status()
        .expect("Failed to execute command");

    println!("Exited with: {}", status);
}
```

### 2.2 Capturing Output

```rust
use std::process::Command;

fn main() {
    let output = Command::new("ls")
        .arg("-la")
        .output()
        .expect("Failed to execute ls");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    println!("Exit status: {}", output.status);
}
```

### 2.3 Spawning a Background Process (`spawn`)

`spawn` launches the child and returns a `Child` handle immediately without waiting.

```rust
use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    let mut child = Command::new("sleep")
        .arg("5")
        .spawn()
        .expect("Failed to spawn sleep");

    println!("Child PID: {}", child.id());

    // Do other work...
    thread::sleep(Duration::from_secs(1));

    // Wait for the child to finish
    let status = child.wait().expect("Failed to wait on child");
    println!("Child exited with: {}", status);
}
```

### 2.4 Checking Exit Status

```rust
use std::process::Command;

fn main() {
    let status = Command::new("false") // always exits with code 1
        .status()
        .expect("Failed to run");

    if status.success() {
        println!("Success");
    } else {
        eprintln!("Failed with code: {:?}", status.code());
    }
}
```

---

## 3. Piping: Stdin, Stdout, Stderr

Use `std::process::Stdio` to configure how streams are handled.

### 3.1 Writing to a Child's Stdin

```rust
use std::process::{Command, Stdio};
use std::io::Write;

fn main() {
    let mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn cat");

    // Write to the child's stdin
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(b"Hello, piped world!\n").expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait");
    println!("Got back: {}", String::from_utf8_lossy(&output.stdout));
}
```

### 3.2 Chaining Processes (Unix Pipe Equivalent)

```rust
use std::process::{Command, Stdio};

fn main() {
    // Equivalent to: echo "hello world foo" | grep "foo"
    let echo = Command::new("echo")
        .arg("hello world foo")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn echo");

    let grep = Command::new("grep")
        .arg("foo")
        .stdin(echo.stdout.expect("Failed to get echo stdout"))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn grep");

    let output = grep.wait_with_output().expect("Failed to get grep output");
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
```

### 3.3 Inheriting Parent Streams

```rust
use std::process::{Command, Stdio};

fn main() {
    // Child writes directly to the terminal
    Command::new("ls")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed");
}
```

### 3.4 Discarding Output

```rust
use std::process::{Command, Stdio};

fn main() {
    Command::new("noisy-program")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok();
}
```

---

## 4. Killing a Child Process

```rust
use std::process::Command;

fn main() {
    let mut child = Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("Failed to spawn");

    println!("Spawned PID: {}", child.id());

    // Terminate the child
    child.kill().expect("Failed to kill child");

    let status = child.wait().expect("Failed to wait");
    println!("Status after kill: {}", status);
}
```

---

## 5. Environment Variables

### 5.1 Reading Environment Variables

```rust
use std::env;

fn main() {
    // Read a specific variable
    match env::var("HOME") {
        Ok(val) => println!("HOME = {}", val),
        Err(e)  => println!("Error reading HOME: {}", e),
    }

    // Iterate over all environment variables
    for (key, value) in env::vars() {
        println!("{} = {}", key, value);
    }
}
```

### 5.2 Setting Environment Variables for a Child Process

```rust
use std::process::Command;

fn main() {
    let output = Command::new("printenv")
        .arg("MY_VAR")
        .env("MY_VAR", "hello_rust")
        .output()
        .expect("Failed to execute");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}
```

### 5.3 Clearing the Environment for a Child

```rust
use std::process::Command;

fn main() {
    // Start with a clean environment, add only what you need
    let output = Command::new("env")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "en_US.UTF-8")
        .output()
        .expect("Failed");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}
```

### 5.4 Removing Specific Variables

```rust
use std::process::Command;

fn main() {
    Command::new("my-app")
        .env_remove("SECRET_TOKEN") // don't pass this to child
        .status()
        .ok();
}
```

### 5.5 Reading at Compile Time

```rust
fn main() {
    // These are resolved at compile time, not runtime
    let profile = env!("CARGO_PKG_VERSION");
    println!("Built as version: {}", profile);

    // Optional compile-time env var
    let maybe = option_env!("CI");
    println!("Running in CI: {}", maybe.is_some());
}
```

---

## 6. Command-Line Arguments

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Program: {}", args[0]);

    for (i, arg) in args.iter().enumerate().skip(1) {
        println!("Arg {}: {}", i, arg);
    }
}
```

For argument parsing in production, use crates like `clap` or `structopt`.

---

## 7. Signal Handling (Unix)

Rust's standard library has no native signal API beyond the basics. The idiomatic approach on Unix is to use the `signal-hook` crate.

### 7.1 Basic Signal Handling with `signal-hook`

Add to `Cargo.toml`:
```toml
[dependencies]
signal-hook = "0.3"
```

```rust
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::thread;
use std::time::Duration;

fn main() {
    let mut signals = Signals::new([SIGINT, SIGTERM]).expect("Failed to register signals");

    // Spawn a thread to handle signals
    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGINT  => println!("\nCaught SIGINT (Ctrl+C), shutting down gracefully..."),
                SIGTERM => println!("Caught SIGTERM, shutting down gracefully..."),
                _       => println!("Received unknown signal: {}", sig),
            }
            std::process::exit(0);
        }
    });

    println!("Running... Press Ctrl+C to stop.");
    loop {
        thread::sleep(Duration::from_secs(1));
        println!("tick");
    }
}
```

### 7.2 Using an Atomic Flag (signal-safe pattern)

```rust
use signal_hook::consts::SIGINT;
use signal_hook::flag;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

fn main() {
    let running = Arc::new(AtomicBool::new(true));

    // Register SIGINT to flip `running` to false
    flag::register(SIGINT, Arc::clone(&running)).expect("Failed to register SIGINT handler");

    println!("Running. Press Ctrl+C to stop.");
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(500));
        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }
    println!("\nShutdown complete.");
}
```

---

## 8. `libc` and Raw Syscall Wrappers

For lower-level POSIX interaction not covered by `std`, use the `libc` crate.

Add to `Cargo.toml`:
```toml
[dependencies]
libc = "0.2"
```

### 8.1 `fork` and `exec`

```rust
use libc::{fork, execvp, waitpid, WIFEXITED, WEXITSTATUS};
use std::ffi::CString;
use std::ptr;

fn main() {
    let pid = unsafe { fork() };

    match pid {
        -1 => eprintln!("fork failed"),
        0  => {
            // Child process
            let program = CString::new("ls").unwrap();
            let args = vec![
                CString::new("ls").unwrap(),
                CString::new("-la").unwrap(),
            ];
            let c_args: Vec<*const libc::c_char> = args.iter()
                .map(|s| s.as_ptr())
                .chain(std::iter::once(ptr::null()))
                .collect();

            unsafe { execvp(program.as_ptr(), c_args.as_ptr()) };
            eprintln!("execvp failed");
            unsafe { libc::_exit(1) };
        }
        child_pid => {
            // Parent process
            let mut status = 0;
            unsafe { waitpid(child_pid, &mut status, 0) };

            if unsafe { WIFEXITED(status) } {
                println!("Child exited with code: {}", unsafe { WEXITSTATUS(status) });
            }
        }
    }
}
```

### 8.2 Sending Signals with `kill`

```rust
use libc::{kill, SIGTERM};

fn terminate_process(pid: i32) {
    let result = unsafe { kill(pid, SIGTERM) };
    if result == -1 {
        eprintln!("Failed to send signal");
    }
}

fn main() {
    let target_pid = 12345; // Replace with real PID
    terminate_process(target_pid);
}
```

### 8.3 Getting/Setting Process Priority (`nice`)

```rust
use libc::{getpriority, setpriority, PRIO_PROCESS};

fn main() {
    let pid = 0; // 0 = current process

    let current = unsafe { getpriority(PRIO_PROCESS, pid as u32) };
    println!("Current priority (nice): {}", current);

    // Lower nice value = higher priority (requires root for negative values)
    let result = unsafe { setpriority(PRIO_PROCESS, pid as u32, 10) };
    if result == 0 {
        println!("Priority set to 10");
    } else {
        eprintln!("Failed to set priority (may need root)");
    }
}
```

### 8.4 File Descriptor Duplication (`dup2`)

```rust
use libc::{dup2, open, O_WRONLY, O_CREAT, O_TRUNC, S_IRUSR, S_IWUSR};
use std::ffi::CString;

fn redirect_stdout_to_file(path: &str) {
    let c_path = CString::new(path).unwrap();
    let fd = unsafe {
        open(c_path.as_ptr(), O_WRONLY | O_CREAT | O_TRUNC, S_IRUSR | S_IWUSR)
    };
    if fd == -1 {
        eprintln!("Failed to open file");
        return;
    }
    // Redirect stdout (fd 1) to the file
    unsafe { dup2(fd, 1) };
    unsafe { libc::close(fd) };
}

fn main() {
    redirect_stdout_to_file("/tmp/output.txt");
    println!("This goes to /tmp/output.txt");
}
```

### 8.5 Making Direct Syscalls with `libc::syscall`

For syscalls not wrapped by `libc`:

```rust
use libc::{syscall, SYS_getpid};

fn main() {
    let pid = unsafe { syscall(SYS_getpid) };
    println!("PID via raw syscall: {}", pid);
}
```

---

## 9. Working Directory

```rust
use std::env;

fn main() {
    // Get current working directory
    let cwd = env::current_dir().expect("Failed to get cwd");
    println!("CWD: {}", cwd.display());

    // Change directory
    env::set_current_dir("/tmp").expect("Failed to change dir");
    println!("New CWD: {}", env::current_dir().unwrap().display());
}
```

Setting the working directory for a child process:

```rust
use std::process::Command;

fn main() {
    let output = Command::new("pwd")
        .current_dir("/tmp")
        .output()
        .expect("Failed");

    println!("Child CWD: {}", String::from_utf8_lossy(&output.stdout).trim());
}
```

---

## 10. Putting It All Together: A Process Runner

```rust
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::env;

fn run_command(program: &str, args: &[&str]) -> i32 {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear()
        .env("PATH", env::var("PATH").unwrap_or_default())
        .spawn()
        .unwrap_or_else(|e| panic!("Failed to spawn '{}': {}", program, e));

    // Stream stdout line by line
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            println!("[stdout] {}", line);
        }
    }

    let status = child.wait().expect("Failed to wait");
    status.code().unwrap_or(-1)
}

fn main() {
    let code = run_command("ls", &["-la", "/tmp"]);
    println!("Exited with code: {}", code);
}
```

---

## Summary

| Concern | Tool |
|---|---|
| Spawn & manage processes | `std::process::Command` |
| Capture/pipe I/O | `std::process::Stdio` + `Child.stdin/stdout/stderr` |
| Environment variables | `std::env::var`, `Command::env` |
| Signal handling | `signal-hook` crate |
| POSIX syscalls (`fork`, `exec`, `kill`, etc.) | `libc` crate |
| Raw syscalls | `libc::syscall` |
| Current directory | `std::env::current_dir` / `set_current_dir` |

Rust's process management ergonomics balance safety and expressiveness — the standard library covers most use cases, while `libc` bridges the gap when POSIX primitives are needed directly.