# Zero字节码导入导出功能

## 概述

Zero编译器现在支持将编译后的字节码保存到文件（`.zbc`格式），并可以从文件中加载字节码直接执行，无需重新编译源代码。

## 使用方法

### 1. 编译源代码到字节码文件

```bash
cargo run -- --compile <source_file.zero> <output.zbc>
```

示例：
```bash
cargo run -- --compile lang-spec/examples/array_test.zero output.zbc
```

### 2. 运行字节码文件

```bash
cargo run -- --run <bytecode_file.zbc>
```

示例：
```bash
cargo run -- --run output.zbc
```

### 3. 直接运行源代码（默认）

```bash
cargo run -- <source_file.zero>
```

### 4. 使用旧的解释器

```bash
cargo run -- --old <source_file.zero>
```

## 文件格式

详见 [`BYTECODE_FORMAT.md`](BYTECODE_FORMAT.md)

字节码文件（`.zbc`）采用二进制格式，包含：
- 文件头（魔数、版本信息）
- 常量池
- 指令序列
- 行号信息

## 优势

1. **更快的启动时间**：跳过词法分析、语法分析、类型检查和编译步骤
2. **分发便利**：可以只分发编译后的字节码，无需源代码
3. **调试支持**：保留行号信息用于错误报告
4. **版本管理**：文件格式包含版本信息，支持向后兼容性检查

## 实现细节

### 序列化器 ([`src/bytecode/serializer.rs`](../src/bytecode/serializer.rs))

- `BytecodeSerializer::serialize()` - 将Chunk序列化为字节流
- 支持所有Value类型：Integer, Float, String, Boolean, Array, Function, Null
- 使用小端序存储多字节整数

### 反序列化器

- `BytecodeDeserializer::deserialize()` - 从字节流重建Chunk
- 验证魔数和版本号
- 完整的错误处理

## 测试覆盖

见 [`tests/bytecode_io_tests.rs`](../tests/bytecode_io_tests.rs)

测试用例包括：
- ✅ 简单程序的序列化/反序列化
- ✅ 数组的序列化/反序列化
- ✅ 函数的序列化/反序列化
- ✅ 嵌套数组的处理
- ✅ 控制流的处理
- ✅ 所有Value类型的处理
- ✅ 无效魔数的错误处理

所有测试全部通过！

## 示例

### 完整工作流程

```bash
# 1. 编写Zero源代码
cat > hello.zero << 'EOF'
let arr: [int] = [1, 2, 3];
print(arr);
EOF

# 2. 编译到字节码
cargo run -- --compile hello.zero hello.zbc

# 3. 运行字节码
cargo run -- --run hello.zbc
# 输出: [1, 2, 3]
```

### 字节码文件大小

```bash
$ ls -lh test_output.zbc
-rw-rw-r-- 1 dev dev 128 Oct 31 03:29 test_output.zbc
```

简单的数组程序编译后仅128字节！

## 未来改进

- [ ] 添加字节码优化pass
- [ ] 支持增量编译
- [ ] 添加CRC32校验和
- [ ] 压缩支持
- [ ] 符号表信息保留（用于反编译）