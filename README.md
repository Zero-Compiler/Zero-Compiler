# Zero 编程语言

Zero是一门使用Rust实现的现代编程语言，采用字节码编译器 + 虚拟机架构，支持静态类型检查。

## 特性

- **字节码编译**：源代码编译为高效的字节码
- **虚拟机执行**：基于栈的虚拟机，快速执行字节码
- **静态类型系统**：编译期类型检查，提高代码安全性
- **类型推导**：智能的局部类型推导
- **数组支持**：完整的数组字面量、索引访问和类型检查
- **字节码序列化**：支持保存和加载编译后的字节码文件
- **简洁语法**：易于学习和使用
- **变量声明**：支持 [`let`](src/parser/mod.rs:92) 和 [`var`](src/parser/mod.rs:92) 关键字
- **基本数据类型**：[`int`](src/ast/mod.rs:6), [`float`](src/ast/mod.rs:7), [`string`](src/ast/mod.rs:8), [`bool`](src/ast/mod.rs:9), [`数组`](src/ast/mod.rs:12)
- **控制流**：[if/else](src/parser/mod.rs:249), [while](src/parser/mod.rs:282), [for循环](src/parser/mod.rs:297)
- **函数定义**：支持函数声明、调用和递归
- **运算符**：算术、比较、逻辑运算符

## 架构

Zero编译器采用现代编译器架构：

```
源代码 → Lexer → Tokens → Parser → AST → Compiler → Bytecode → VM → 执行
```

- **词法分析器（Lexer）**：将源代码转换为Token流
- **语法分析器（Parser）**：构建抽象语法树（AST）
- **编译器（Compiler）**：将AST编译为字节码
- **虚拟机（VM）**：执行字节码指令

详细架构文档请查看 [ARCHITECTURE.md](docs/ARCHITECTURE.md)

## 语法示例

### Hello World

```zero
print("Hello, Zero!");
```

### 变量声明（支持类型注解）

```zero
// 不可变变量
let x: int = 42;
let y: float = 3.14;
let name: string = "Zero";
let flag: bool = true;

// 类型推导
let auto_int = 100;        // 推导为 int
let auto_float = 2.718;    // 推导为 float
let auto_string = "hello"; // 推导为 string

// 可变变量
var count = 0;
count = count + 1;
```

### 数组

```zero
// 数组字面量
let numbers: [int] = [1, 2, 3, 4, 5];
let names = ["Alice", "Bob", "Charlie"];

// 数组索引访问
print(numbers[0]);        // 1
numbers[0] = 10;
print(numbers[0]);        // 10

// 嵌套数组
let matrix: [[int]] = [[1, 2, 3], [4, 5, 6]];
print(matrix[0][1]);      // 2
```

### 函数定义（支持类型注解）

```zero
// 带类型注解的函数
fn add(a: int, b: int) -> int {
    return a + b;
}

// 无类型注解的函数（类型推导）
fn factorial(n) {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

// 混合类型注解
fn multiply(x, y: int) {
    return x * y;
}

let result = add(10, 20);
print(result);  // 输出: 30
```

### 控制流

```zero
// If-else 语句
let x = 15;
if x > 10 {
    print("x is greater than 10");
} else {
    print("x is less than or equal to 10");
}

// While 循环
let counter = 0;
while counter < 5 {
    print(counter);
    counter = counter + 1;
}

// For 循环
for i in 0..10 {
    print(i);
}
```

## 项目结构

```
Zero-compiler/
├── src/
│   ├── main.rs              # 主程序入口
│   ├── lib.rs               # 库接口
│   ├── lexer/               # 词法分析器
│   │   ├── mod.rs           # Lexer实现
│   │   └── token.rs         # Token定义
│   ├── parser/              # 语法分析器
│   │   └── mod.rs           # Parser实现（递归下降）
│   ├── ast/                 # 抽象语法树
│   │   └── mod.rs           # AST节点定义 + 类型系统
│   ├── type_checker/        # 类型检查器
│   │   └── mod.rs           # 静态类型检查和推导
│   ├── bytecode/            # 字节码系统
│   │   ├── mod.rs           # OpCode、Value、Chunk定义
│   │   └── serializer.rs    # 字节码序列化/反序列化
│   ├── compiler/            # 字节码编译器
│   │   └── mod.rs           # AST → 字节码编译
│   ├── vm/                  # 虚拟机
│   │   └── mod.rs           # 基于栈的VM实现
│   └── interpreter/         # 解释器（保留用于对比）
│       └── mod.rs           # 树遍历解释器
├── lang-spec/               # Zero语言规范 (子模块)
│   ├── examples/            # 示例程序
│   │   ├── hello.zero       # Hello World
│   │   ├── variables.zero   # 变量示例
│   │   ├── functions.zero   # 函数示例
│   │   └── ...              # 更多示例
│   └── spec/                # 语言规范文档
├── docs/                    # 文档
│   ├── ARCHITECTURE.md      # 架构文档
│   ├── LANGUAGE_SPEC.md     # 语言规范
│   ├── TYPE_SYSTEM.md       # 类型系统设计
│   ├── ARRAYS.md            # 数组系统设计
│   ├── BYTECODE_FORMAT.md   # 字节码文件格式
│   └── BYTECODE_IO.md       # 字节码导入导出
```

## 构建和运行

### 安装依赖

确保已安装 [Rust](https://www.rust-lang.org/)（推荐使用最新稳定版）。

### 构建项目

```bash
# 构建项目
cargo build

# 构建发布版本（优化）
cargo build --release
```

### 运行程序

```bash
# 运行示例程序（使用字节码VM）
cargo run lang-spec/examples/hello.zero
cargo run lang-spec/examples/functions.zero
cargo run lang-spec/examples/control_flow.zero
cargo run lang-spec/examples/arrays.zero
cargo run lang-spec/examples/types.zero

# 运行自定义程序
cargo run -- <source_file.zero>

# 编译到字节码文件
cargo run -- --compile <source_file.zero> <output.zbc>

# 运行字节码文件
cargo run -- --run <bytecode_file.zbc>

# 使用旧的树遍历解释器（用于对比）
cargo run -- --old <source_file.zero>
```

### 调试模式

查看生成的字节码和VM执行过程：

```bash
# 设置环境变量启用字节码反汇编
ZERO_DEBUG=1 cargo run lang-spec/examples/functions.zero

# 在debug构建中自动显示栈状态
cargo run --debug lang-spec/examples/functions.zero
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_functions

# 显示测试输出
cargo test -- --nocapture
```

## 性能

基于fibonacci(30)的性能测试：

- **字节码VM**: ~1.8秒
- **树遍历解释器**: ~2.5秒
- **性能提升**: ~28%

## 示例程序

项目包含以下示例程序：

1. **[hello.zero](lang-spec/examples/hello.zero)** - 基本的Hello World程序
2. **[variables.zero](lang-spec/examples/variables.zero)** - 变量声明和使用
3. **[functions.zero](lang-spec/examples/functions.zero)** - 函数定义和调用
4. **[control_flow.zero](lang-spec/examples/control_flow.zero)** - 控制流结构（if/while/for）
5. **[types.zero](lang-spec/examples/types.zero)** - 类型系统和类型注解
6. **[arrays.zero](lang-spec/examples/arrays.zero)** - 数组操作和索引访问
7. **[array_test.zero](lang-spec/examples/array_test.zero)** - 数组功能完整测试

运行示例：

```bash
# 运行所有示例
for example in lang-spec/examples/*.zero; do
    echo "=== Running $example ==="
    cargo run --quiet "$example"
    echo
done
```

## 开发指南

### 添加新特性

详细步骤请查看 [ARCHITECTURE.md](docs/ARCHITECTURE.md#扩展性)。

基本流程：
1. 在Lexer中添加新Token
2. 在AST中添加新节点
3. 在Parser中添加解析逻辑
4. 在Compiler中添加编译逻辑
5. 在VM中添加执行逻辑（如需要）
6. 添加测试

### 代码风格

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 遵循 Rust 官方编码规范

### 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 文档

- [语言规范](docs/LANGUAGE_SPEC.md) - Zero语言的语法和语义
- [架构文档](docs/ARCHITECTURE.md) - 编译器内部架构
- [类型系统](docs/TYPE_SYSTEM.md) - 静态类型系统设计
- [数组系统](docs/ARRAYS.md) - 数组类型和操作
- [字节码格式](docs/BYTECODE_FORMAT.md) - 字节码文件格式规范
- [字节码IO](docs/BYTECODE_IO.md) - 字节码导入导出功能
- [贡献指南](CONTRIBUTING.md) - 如何为项目做贡献

## 开发状态

✅ **已完成**：
- ✅ 词法分析器（完整Token支持）
- ✅ 语法分析器（递归下降）
- ✅ 抽象语法树（完整的AST节点）
- ✅ 静态类型系统（基础类型 + 数组类型）
- ✅ 类型检查器（编译期类型检查）
- ✅ 类型推导（局部类型推导）
- ✅ 数组支持（字面量、索引访问、类型检查）
- ✅ 字节码编译器（AST → 字节码）
- ✅ 虚拟机（基于栈的VM）
- ✅ 字节码序列化/反序列化（.zbc文件格式）
- ✅ 基本数据类型和运算
- ✅ 控制流（if/while/for）
- ✅ 函数定义和调用
- ✅ 递归函数支持
- ✅ 短路求值优化

🚧 **进行中**：
- 🚧 数组方法（push、pop、insert等）
- 🚧 函数返回类型注解完整支持

📋 **计划中**：
- 📋 模块系统（import/export）
- 📋 标准库（字符串、数组、IO等）
- 📋 错误处理（try-catch）
- 📋 闭包（捕获外部变量）
- 📋 垃圾回收（标记-清除GC）
- 📋 泛型支持
- 📋 结构体/类
- 📋 REPL（交互式解释器）
- 📋 调试器（断点、单步执行）
- 📋 LSP支持（IDE集成）
- 📋 包管理器

## 技术栈

- **语言**: Rust (Edition 2024)
- **依赖**: 最小化依赖，仅使用Rust标准库
- **测试**: Cargo内置测试框架
- **架构**: 编译器前端 + 字节码VM

## 许可证

MIT License

## 致谢

灵感来源：
- [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom
- [Writing An Interpreter In Go](https://interpreterbook.com/) by Thorsten Ball
- Lua VM

## 联系方式

- 问题反馈：[GitHub Issues](https://github.com/YUZHEthefool/Zero-compiler/issues)
- 讨论交流：[GitHub Discussions](https://github.com/YUZHEthefool/Zero-compiler/discussions)