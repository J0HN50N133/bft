# Debug Guide - bash-fzf-tab-completion

## 快速开始

本项目的调试通过 `logforth` 日志库和 `RUST_LOG` 环境变量控制。

**重要**：日志输出到 stderr，补全输出到 stdout，两者互不干扰。这样 `eval "$output"` 不会尝试执行日志内容。

### 基本用法

```bash
# 启用 INFO 级别日志（推荐用于日常使用）
RUST_LOG=info ./target/release/bash-fzf-tab-completion

# 启用 DEBUG 级别日志（推荐用于开发调试）
RUST_LOG=debug ./target/release/bash-fzf-tab-completion

# 启用 TRACE 级别日志（非常详细）
RUST_LOG=trace ./target/release/bash-fzf-tab-completion

# 不显示日志（生产环境推荐）
unset RUST_LOG
./target/release/bash-fzf-tab-completion
```

### 与 bash 结合使用

```bash
# 在测试时使用
RUST_LOG=debug READLINE_LINE="ls " READLINE_POINT=3 \
    ./target/release/bash-fzf-tab-completion

# 添加到 .bashrc
export RUST_LOG=info
```

## 日志级别

**日志输出到 stderr，补全输出到 stdout**，两者完全分离。

- **error**: 只显示错误
- **warn**: 显示警告和错误
- **info**: 显示一般信息（默认推荐）
- **debug**: 显示详细调试信息（开发时使用）
- **trace**: 显示所有详细信息（非常详细）

## 示例输出

### INFO 级别
```bash
$ RUST_LOG=info READLINE_LINE="cd /t" READLINE_POINT=5 \
    ./target/release/bash-fzf-tab-completion
```

输出：
```
2026-01-14T02:14:55.888718+08:00[Asia/Shanghai]  INFO bash_fzf_tab_completion: main.rs:23 Starting bash-fzf-tab-completion
2026-01-14T02:14:55.897288+08:00[Asia/Shanghai]  INFO bash_fzf_tab_completion: main.rs:53 Generated 1 completion candidates
2026-01-14T02:14:55.897422+08:00[Asia/Shanghai]  INFO bash_fzf_tab_completion: main.rs:100 Completion finished
READLINE_LINE='cd /tmp/'
READLINE_POINT=8
```

### DEBUG 级别
```bash
$ RUST_LOG=debug READLINE_LINE="git che" READLINE_POINT=7 \
    ./target/release/bash-fzf-tab-completion
```

输出：
```
2026-01-14T02:14:59.406438+08:00[Asia/Shanghai]  INFO bash_fzf_tab_completion: main.rs:23 Starting bash-fzf-tab-completion
2026-01-14T02:14:59.408798+08:00[Asia/Shanghai] DEBUG bash_fzf_tab_completion: main.rs:26 Configuration: Config { ... }
2026-01-14T02:14:59.408852+08/00[Asia/Shanghai] DEBUG bash_fzf_tab_completion: main.rs:33 Input: line='git che', point=7
2026-01-14T02:14:59.408978+08:00[Asia/Shanghai] DEBUG bash_fzf_tab_completion: main.rs:43 Parsed command: ParsedLine { ... }
2026-01-14T02:14:59.409028+08:00[Asia/Shanghai] DEBUG bash_fzf_tab_completion: main.rs:46 Command: 'git', current_word: 'che', current_word_idx: 1
2026-01-14T02:14:59.411947+08:00[Asia/Shanghai] DEBUG bash_fzf_tab_completion: main.rs:50 Completion spec: CompletionSpec { ... }
2026-01-14T02:14:59.414675+08:00[Asia/Shanghai]  INFO bash_fzf_tab_completion: main.rs:53 Generated 8 completion candidates
2026-01-14T02:14:59.414765+08:00[Asia/Shanghai] DEBUG bash_fzf_tab_completion: main.rs:67 After filtering: 8 candidates
2026-01-14T02:14:59.414815+08:00[Asia/Shanghai] DEBUG bash_fzf_tab_completion: main.rs:83 Opening FZF with 8 candidates
2026-01-14T02:14:59.414873+08:00[Asia/Shanghai] DEBUG bash_fzf_tab_completion: main.rs:89 Selected completion: 'checkout'
2026-01-14T02:14:59.414961+08:00[Asia/Shanghai]  INFO bash_fzf_tab_completion: main.rs:100 Completion finished
```

## 常见调试场景

### 1. 补全不工作

```bash
RUST_LOG=debug READLINE_LINE="your-command" READLINE_POINT=10 \
    ./target/release/bash-fzf-tab-completion
```

查看：
- 命令是否正确解析
- 是否有补全规范（compspec）
- 是否生成了候选

### 2. FZF 不显示

```bash
RUST_LOG=info READLINE_LINE="ls " READLINE_POINT=3 \
    ./target/release/bash-fzf-tab-completion
```

查看日志中的 "Opening FZF" 信息。

### 3. 只查看补全输出（忽略日志）

```bash
# 重定向 stderr 到 /dev/null，只看补全结果
RUST_LOG=debug READLINE_LINE="cd /t" READLINE_POINT=5 \
    ./target/release/bash-fzf-tab-completion 2>/dev/null

# 输出（只有补全结果）：
# READLINE_LINE='cd /tmp/'
# READLINE_POINT=8
```

### 4. 只查看日志（忽略补全输出）

```bash
# 重定向 stdout 到 /dev/null，只看日志
RUST_LOG=debug READLINE_LINE="cd /t" READLINE_POINT=5 \
    ./target/release/bash-fzf-tab-completion >/dev/null

# 输出（只有日志）：
# 2026-01-14T02:14:59.406438+08:00[Asia/Shanghai]  INFO ...
```

### 5. 分别查看 stdout 和 stderr

```bash
# 将它们分别保存到不同文件
RUST_LOG=debug READLINE_LINE="cd /t" READLINE_POINT=5 \
    ./target/release/bash-fzf-tab-completion 1>output.txt 2>log.txt

# 查看补全输出
cat output.txt
# READLINE_LINE='cd /tmp/'
# READLINE_POINT=8

# 查看日志
cat log.txt
# 2026-01-14T02:14:59.406438+08:00[Asia/Shanghai]  INFO ...
```

### 6. 查看配置

```bash
RUST_LOG=debug ./target/release/bash-fzf-tab-completion
```

查看 Configuration 日志条目。

## 单元测试调试

```bash
# 运行测试并查看日志
RUST_LOG=debug cargo test -- --nocapture --show-output

# 运行特定测试
RUST_LOG=debug cargo test parser::tests::test_parse_simple -- --nocapture
```

## 其他调试方法

### 使用 dbg! 宏

对于快速调试，可以在代码中添加 `dbg!` 宏：

```rust
let config = Config::from_env();
dbg!(&config);

let parsed = parser::parse_shell_line(&readline_line, readline_point)?;
dbg!(&parsed);
```

### 使用断点调试器

在 VSCode 中创建 `.vscode/launch.json`:

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
        "RUST_LOG": "debug",
        "READLINE_LINE": "ls ",
        "READLINE_POINT": "3"
      },
      "args": []
    }
  ]
}
```

## 测试脚本

运行预定义的测试：

```bash
# 边缘情况测试
./test_edge_cases.sh

# 日志功能测试
./test_logging.sh
```
