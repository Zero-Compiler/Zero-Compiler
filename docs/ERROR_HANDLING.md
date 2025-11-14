# Zero编译器错误处理系统

## 概述

Zero编译器实现了一个完善的两级错误处理机制，设计灵感来自Rust编译器的错误报告系统。

错误消息配置使用Git Submodule管理，位于 `error-msg/` 目录，支持多语言国际化。

## 错误模式

### 1. 简易模式（Simple Mode）

简易模式仅显示错误的基本信息：
- 错误代码
- 行号和列号
- 简短的错误描述

**示例输出：**
```
错误 [L001] 在 2:19: 未闭合的字符串字面量
```

### 2. 详细模式（Detailed Mode，使用 --dtl 标志）

详细模式提供完整的错误上下文：
- 错误代码和标题
- 源码位置（文件、行、列）
- 带高亮的源码片段
- 详细的错误描述
- 修复建议

**示例输出：**
```
error[L001]: 未闭合的字符串字面量
  --> <input>:2:19
   1 | // 测试错误处理 - 未闭合的字符串
   2 | let message = "Hello World
     |               ^~~~~~~~~~~~
   3 | 

字符串字面量必须以双引号(")开始和结束。

帮助: 在字符串末尾添加闭合的双引号 "
```

## 使用方法

### 命令行标志

```bash
# 简易模式（默认）
cargo run --bin Zero-compiler lang-spec/examples/error_test.zero

# 详细模式
cargo run --bin Zero-compiler lang-spec/examples/error_test.zero --dtl

# 编译模式
cargo run --bin Zero-compiler -- --compile lang-spec/examples/error_test.zero output.zbc --dtl

# 旧解释器模式
cargo run --bin Zero-compiler -- --old lang-spec/examples/error_test.zero --dtl

# 或者先编译后运行
cargo build --release
./target/release/Zero-compiler lang-spec/examples/error_test.zero --dtl
```

## 错误类型

### 词法分析错误（Lexer Errors）

#### L001: 未闭合的字符串字面量
**触发条件：** 字符串没有结束引号

**示例：**
```zero
let s = "Hello World
```

**修复建议：** 在字符串末尾添加闭合的双引号 "

---

#### L002: 无效的转义序列
**触发条件：** 使用了不支持的转义字符

**示例：**
```zero
let s = "test\q"
```

**有效转义序列：**
- `\n` - 换行符
- `\t` - 制表符
- `\r` - 回车符
- `\\` - 反斜杠
- `\"` - 双引号
- `\'` - 单引号
- `\0` - 空字符
- `\xHH` - 十六进制转义（两位）
- `\u{XXXX}` - Unicode转义（1-6位）

**修复建议：** 检查转义序列的拼写，或使用raw字符串 r"..."

---

#### L003: 意外的字符
**触发条件：** 遇到无效的字符

**示例：**
```zero
let x = 10 @ 20;
```

**修复建议：** 检查是否有拼写错误或多余的字符

---

#### L004: 无效的数字格式
**触发条件：** 数字字面量格式不正确

**示例：**
```zero
let x = 0x;      // 十六进制缺少数字
let y = 0b;      // 二进制缺少数字
let z = 1.2e;    // 科学计数法缺少指数
```

**支持的数字格式：**
- 十进制：`123`, `45.67`, `1.2e10`
- 十六进制：`0xFF`, `0x1A`
- 二进制：`0b1010`, `0b11`
- 八进制：`0o755`, `0o17`

**修复建议：** 根据具体错误类型提供相应建议

---

#### L005: 无效的Unicode转义序列
**触发条件：** Unicode转义序列格式错误

**示例：**
```zero
let s = "\u123";     // 固定格式需要4位
let t = "\u{GGGG}";  // 包含非十六进制字符
```

**正确格式：**
- `\uXXXX` - 固定4位十六进制
- `\u{X}` 到 `\u{XXXXXX}` - 1-6位十六进制，需要花括号

**修复建议：** 确保Unicode码点是有效的十六进制数字，并且在有效范围内（U+0000 到 U+10FFFF）

---

### 语法分析错误（Parser Errors）

#### P001: 意外的token
**触发条件：** 期望某个token但发现了另一个

**示例：**
```zero
let x = ;  // 期望表达式，但发现分号
```

---

#### P002: 意外的文件结束
**触发条件：** 在解析完成前遇到文件结束

**示例：**
```zero
fn test() {
    let x = 10;
// 缺少闭合的大括号
```

---

#### P003: 无效的表达式
**触发条件：** 无法解析为有效的表达式

## 错误代码索引

| 代码 | 类别 | 描述 |
|------|------|------|
| L001 | 词法 | 未闭合的字符串字面量 |
| L002 | 词法 | 无效的转义序列 |
| L003 | 词法 | 意外的字符 |
| L004 | 词法 | 无效的数字格式 |
| L005 | 词法 | 无效的Unicode转义序列 |
| P001 | 语法 | 意外的token |
| P002 | 语法 | 意外的文件结束 |
| P003 | 语法 | 无效的表达式 |

## 实现细节

### 架构

错误处理系统由以下模块组成：

1. **src/error/mod.rs** - 错误类型定义和格式化逻辑
   - `ErrorMode` - 错误显示模式枚举
   - `SourceLocation` - 源码位置信息
   - `CompilerError` trait - 所有错误的共同接口
   - 各种具体错误类型

2. **src/lexer/mod.rs** - 词法分析器错误集成
   - 使用新的错误类型替换旧的错误结构
   - 在所有错误点提供详细的位置信息

3. **src/main.rs** - 命令行接口
   - 解析 `--dtl` 标志
   - 根据错误模式格式化错误输出

### 错误格式化

#### 简易模式格式
```
错误 [{错误代码}] 在 {行}:{列}: {标题}
```

#### 详细模式格式
```
error[{错误代码}]: {标题}
  --> {文件}:{行}:{列}
   {行-1} | {前一行源码}
   {行}   | {出错行源码}
        | {空格}^~~~ {错误指示符}
   {行+1} | {后一行源码}

{详细描述}

帮助: {修复建议}
```

## 测试

### 运行错误测试

```bash
# 简易模式
cargo run --bin Zero-compiler lang-spec/examples/error_test.zero

# 详细模式
cargo run --bin Zero-compiler lang-spec/examples/error_test.zero --dtl
```

### 测试示例

查看 `lang-spec/examples/error_test.zero` 文件以了解各种错误类型的示例。

## 扩展

### 添加新的错误类型

1. 在 `error-msg/locale/zh_CN/error_messages.toml` 中添加错误配置：

```toml
[lexer.L006]
code = "L006"
title = "你的新错误标题"
description = "详细描述"
suggestion = "修复建议"
category = "lexer"
```

2. 在 `src/error/mod.rs` 中的 `ErrorType` 枚举添加新变体：

```rust
pub enum ErrorType {
    // ... 现有错误类型
    LexerYourNewError,
}
```

3. 实现 `code()` 和 `config_key()` 方法：

```rust
impl ErrorType {
    pub fn code(&self) -> &'static str {
        match self {
            // ...
            Self::LexerYourNewError => "L006",
        }
    }
    
    pub fn config_key(&self) -> &'static str {
        match self {
            // ...
            Self::LexerYourNewError => "lexer.L006",
        }
    }
}
```

4. 添加便捷构造函数：

```rust
impl CompilerError {
    pub fn your_new_error(line: usize, column: usize, offset: usize) -> Self {
        Self::new(
            "L006",
            SourceLocation::single(line, column, offset),
            ErrorType::LexerYourNewError,
        )
    }
}
```

5. 在词法分析器或语法分析器中使用：

```rust
return Err(CompilerError::your_new_error(self.line, self.column, self.position));
```

### 添加新语言支持

1. 在error-msg仓库创建新的语言目录：
```bash
cd error-msg
mkdir -p locale/en_US
cp locale/zh_CN/error_messages.toml locale/en_US/
# 翻译 error_messages.toml 中的内容
```

2. 在代码中使用：
```rust
let registry = ErrorRegistry::from_locale("en_US")?;
let displayer = ErrorDisplayer::new(error_mode).with_registry(registry);
```

## 最佳实践

1. **始终提供位置信息** - 每个错误都应包含准确的行号、列号和偏移量
2. **提供清晰的描述** - 错误描述应该简洁但完整
3. **包含修复建议** - 尽可能提供具体的修复步骤
4. **使用适当的错误代码** - 保持错误代码的一致性和可查询性
5. **测试两种模式** - 确保错误在简易和详细模式下都能正确显示

## 未来改进

- [ ] 添加错误恢复机制，继续解析以发现更多错误
- [ ] 支持自定义错误格式化模板
- [ ] 添加错误统计和汇总
- [ ] 支持JSON格式的错误输出（用于IDE集成）
- [ ] 添加更多语言级别的错误类型（类型错误、语义错误等）
- [ ] 支持错误级别（错误、警告、提示）