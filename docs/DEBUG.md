# Debug Configuration

## 使用 logforth 日志库

本项目使用 `logforth` 日志库，支持通过环境变量控制日志级别。

## 环境变量控制

设置 `LOGFORTH_FILTER` 环境变量来控制日志级别：

```bash
# 只显示错误
export LOGFORTH_FILTER=error

# 显示警告和错误
export LOGFORTH_FILTER=warn

# 显示信息、警告和错误（默认）
export LOGFORTH_FILTER=info

# 显示所有日志
export LOGFORTH_FILTER=debug

# 显示所有日志包括 trace
export LOGFORTH_FILTER=trace
```

### 快速开始

```bash
# 启用调试日志
LOGFORTH_FILTER=debug ./target/release/bash-fzf-tab-completion

# 在测试时使用
LOGFORTH_FILTER=debug cargo test

# 与其他环境变量一起使用
LOGFORTH_FILTER=debug READLINE_LINE="ls " READLINE_POINT=3 ./target/release/bash-fzf-tab-completion
```

## 使用 cargo 的调试工具

### 使用 println! 宏

在代码中添加调试输出：

```rust
// src/main.rs
fn main() -> Result<()> {
    println!("DEBUG: READLINE_LINE = {:?}", readline_line);
    println!("DEBUG: READLINE_POINT = {}", readline_point);
    
    let parsed = parser::parse_shell_line(&readline_line, readline_point)?;
    println!("DEBUG: Parsed result: {:?}", parsed);
    
    let spec = completion::resolve_compspec(&ctx.command)?;
    println!("DEBUG: Completion spec: {:?}", spec);
    
    let candidates = completion::execute_completion(&spec, &ctx)?;
    println!("DEBUG: Candidates count: {}", candidates.len());
    println!("DEBUG: First 5 candidates: {:?}", &candidates.iter().take(5).collect::<Vec<_>>());
    
    // ...
}
```

### 使用 eprintln! 输出到 stderr

```rust
// stderr 不会干扰正常的 bash 输出
eprintln!("DEBUG: Config: {:?}", config);
eprintln!("DEBUG: Current word: '{}'", ctx.current_word);
```

### 使用 dbg! 宏（推荐）

`dbg!` 会自动打印文件名、行号和表达式的值：

```rust
fn main() -> Result<()> {
    let config = Config::from_env();
    dbg!(&config);  // 打印配置
    
    let parsed = parser::parse_shell_line(&readline_line, readline_point)?;
    dbg!(&parsed);  // 打印解析结果
    
    let ctx = CompletionContext::from_parsed(&parsed, readline_line.clone(), readline_point);
    dbg!(&ctx);  // 打印上下文
    
    // ...
}
```

## 使用 dbg! 宏（快速调试）

对于快速调试，可以使用 Rust 的 `dbg!` 宏：

```rust
fn main() -> Result<()> {
    let config = Config::from_env();
    dbg!(&config);  // 打印配置

    let parsed = parser::parse_shell_line(&readline_line, readline_point)?;
    dbg!(&parsed);  // 打印解析结果

    let ctx = CompletionContext::from_parsed(&parsed, readline_line.clone(), readline_point);
    dbg!(&ctx);  // 打印上下文
}
```

## 使用 Rust Debugger (lldb/gdb)

### 1. 编译 debug 版本

```bash
cargo build
```

### 2. 使用 lldb (Linux/macOS)

```bash
lldb target/debug/bash-fzf-tab-completion
(lldb) env READLINE_LINE="ls "
(lldb) env READLINE_POINT=3
(lldb) run
```

### 3. 使用 gdb (Linux)

```bash
gdb target/debug/bash-fzf-tab-completion
(gdb) set environment READLINE_LINE="ls "
(gdb) set environment READLINE_POINT=3
(gdb) run
```

### 4. 在 VSCode 中调试

创建 `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug bash-fzf-tab-completion",
      "cargo": {
        "args": ["build"],
        "filter": {
          "name": "bash-fzf-tab-completion",
          "kind": "bin"
        }
      },
      "env": {
        "READLINE_LINE": "ls ",
        "READLINE_POINT": "3"
      },
      "args": []
    }
  ]
}
```

## 测试特定场景

### 测试文件补全

```bash
# 手动设置环境变量测试
READLINE_LINE="cat " READLINE_POINT=4 \
    cargo run -- --help 2>&1 || \
    READLINE_LINE="cat " READLINE_POINT=4 \
    target/debug/bash-fzf-tab-completion
```

### 测试 Git 补全

```bash
READLINE_LINE="git che" READLINE_POINT=7 \
    target/debug/bash-fzf-tab-completion
```

### 测试目录补全

```bash
READLINE_LINE="cd /tmp" READLINE_POINT=7 \
    target/debug/bash-fzf-tab-completion
```

## 单元测试调试

### 运行单个测试并打印输出

```bash
cargo test parser::tests::test_parse_simple -- --nocapture --show-output
```

### 打印测试中的变量

```rust
#[test]
fn test_parse_simple() {
    let result = parser::parse_shell_line("ls -la", 5).unwrap();
    dbg!(&result);  // 测试时也会打印
    assert_eq!(result.words, vec!["ls", "-la"]);
}
```

## 常见调试场景

### 1. 查看输入和解析

```bash
LOGFORTH_FILTER=debug READLINE_LINE="ls -la" READLINE_POINT=6 \
    ./target/release/bash-fzf-tab-completion
```

输出：
```
INFO Starting bash-fzf-tab-completion
DEBUG Configuration: Config { ... }
DEBUG Input: line='ls -la', point=6
DEBUG Parsed command: ParsedResult { ... }
DEBUG Command: 'ls', current_word='-la', arg_index: 1
INFO Generated 45 completion candidates
```

### 2. 补全候选为空

代码中已添加日志：
```rust
info!("Generated {} completion candidates", candidates.len());
```

查看日志：
```bash
LOGFORTH_FILTER=info ./target/release/bash-fzf-tab-completion
```

### 3. FZF 不显示或不工作

代码中已添加日志：
```rust
info!("Opening FZF with {} candidates", candidates.len());
```

查看是否正确触发 FZF：
```bash
LOGFORTH_FILTER=info READLINE_LINE="git " READLINE_POINT=4 \
    ./target/release/bash-fzf-tab-completion
```

## 日志级别说明

- `error`: 只显示错误信息
- `warn`: 显示警告和错误
- `info`: 显示一般信息（推荐用于生产）
- `debug`: 显示详细的调试信息（推荐用于开发）
- `trace`: 显示所有详细信息（包括非常详细的跟踪）

## 性能分析

### 1. 使用 criterion 基准测试

```bash
cargo bench
```

### 2. 使用 flamegraph

```toml
[dependencies]
flamegraph = "0.6"
```

```bash
cargo flamegraph
```

## 推荐的调试流程

1. **日常调试**: 使用 `LOGFORTH_FILTER=debug` 环境变量
2. **快速调试**: 使用 `dbg!` 宏
3. **复杂问题**: 使用 `lldb`/`gdb` 或 VSCode 调试器
4. **性能问题**: 使用 `cargo bench` 或 `cargo flamegraph`
5. **测试失败**: 使用 `--nocapture` 和 `LOGFORTH_FILTER=debug`

## 日志输出示例

```bash
# 启用调试日志查看完整流程
LOGFORTH_FILTER=debug READLINE_LINE="git che" READLINE_POINT=7 \
    ./target/release/bash-fzf-tab-completion
```

示例输出：
```
INFO Starting bash-fzf-tab-completion
DEBUG Configuration: Config { fzf_tmux_height: Some("40%"), ... }
DEBUG Input: line='git che', point=7
DEBUG Parsed command: ParsedResult { words: ["git", "che"], ... }
DEBUG Command: 'git', current_word='che', arg_index: 1
DEBUG Completion spec: CompletionSpec { ... }
INFO Generated 8 completion candidates
DEBUG After filtering: 8 candidates
DEBUG Selected completion: 'checkout'
INFO Completion finished
READLINE_LINE='git checkout '
READLINE_POINT=13
```
