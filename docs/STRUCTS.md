# 结构体系统文档

本文档描述了Zero编译器中结构体系统的实现。

## 概述

结构体是一种复合数据类型，允许将多个不同类型的值组合在一起。Zero语言支持：

1. 结构体声明和定义
2. 结构体字面量创建
3. 字段访问和赋值
4. 类型别名（包括匿名结构体）

## 语法

### 结构体声明

```rust
struct Person {
    name: String,
    age: Integer,
    active: Boolean
};
```

### 类型别名

```rust
// 简单类型别名
type User = Person;

// 匿名结构体别名
type Point = struct {
    x: Integer,
    y: Integer
};
```

### 结构体实例化

```rust
let person = Person {
    name: "Alice",
    age: 30,
    active: true
};
```

### 字段访问

```rust
let name = person.name;
let age = person.age;
```

### 字段赋值

```rust
person.age = 31;
person.name = "Bob";
```

## 实现细节

### 词法分析 (Lexer)

在 [`src/lexer/token.rs`](../src/lexer/token.rs) 中添加了以下关键字：
- `struct` - 结构体声明
- `type` - 类型别名
- `impl`, `pub`, `use`, `mod` - 为未来扩展预留

### 抽象语法树 (AST)

在 [`src/ast/mod.rs`](../src/ast/mod.rs) 中定义了：

#### 类型定义
```rust
pub struct StructField {
    pub name: String,
    pub field_type: Type,
}

pub struct StructType {
    pub name: String,
    pub fields: Vec<StructField>,
}

pub enum Type {
    // ... 其他类型
    Struct(StructType),
    Named(String),  // 用户定义的类型名
}
```

#### 语句类型
```rust
pub enum Stmt {
    // ... 其他语句
    StructDeclaration {
        name: String,
        fields: Vec<StructField>,
    },
    TypeAlias {
        name: String,
        target_type: Type,
    },
}
```

#### 表达式类型
```rust
pub enum Expr {
    // ... 其他表达式
    StructLiteral {
        struct_name: String,
        fields: Vec<(String, Expr)>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    FieldAssign {
        object: Box<Expr>,
        field: String,
        value: Box<Expr>,
    },
}
```

### 语法解析器 (Parser)

在 [`src/parser/mod.rs`](../src/parser/mod.rs) 中实现了：

1. **`struct_declaration()`** - 解析结构体声明
   - 支持带类型注解的字段
   - 支持可选的尾随逗号

2. **`type_alias_declaration()`** - 解析类型别名
   - 支持简单类型别名
   - 支持匿名结构体别名

3. **结构体字面量解析** - 在 `primary()` 中
   - 识别 `Identifier { ... }` 模式
   - 解析字段名和值对

4. **字段访问解析** - 在 `call()` 中
   - 支持点号语法 `object.field`
   - 可以链式访问

5. **字段赋值解析** - 在 `assignment()` 中
   - 支持 `object.field = value` 语法

### 类型检查器 (Type Checker)

在 [`src/type_checker/mod.rs`](../src/type_checker/mod.rs) 中实现了：

1. **结构体类型注册** - 在 `check_statement()` 中
   - 将结构体类型添加到符号表
   - 存储字段信息以供后续检查

2. **字段访问类型推断** - 在 `infer_type()` 中
   - 验证对象是结构体类型
   - 检查字段是否存在
   - 返回正确的字段类型

3. **字段赋值类型检查**
   - 验证对象和字段
   - 检查赋值类型兼容性

### 字节码系统 (Bytecode)

在 [`src/bytecode/mod.rs`](../src/bytecode/mod.rs) 中定义了：

#### 操作码
```rust
pub enum OpCode {
    // ... 其他操作码
    NewStruct(usize),   // 创建结构体（参数：字段数量）
    FieldGet(usize),    // 获取字段（参数：字段索引）
    FieldSet(usize),    // 设置字段（参数：字段索引）
}
```

#### 值类型
```rust
pub struct StructValue {
    pub struct_name: String,
    pub fields: Vec<Value>,  // 按字段定义顺序存储
}

pub enum Value {
    // ... 其他值类型
    Struct(StructValue),
}
```

### 序列化器 (Serializer)

在 [`src/bytecode/serializer.rs`](../src/bytecode/serializer.rs) 中：

1. **结构体值序列化** - 类型ID: 0x08
   - 序列化结构体名称
   - 序列化字段数量
   - 递归序列化每个字段值

2. **操作码序列化**
   - `NewStruct`: 0x64
   - `FieldGet`: 0x65
   - `FieldSet`: 0x66

### 虚拟机 (VM)

在 [`src/vm/mod.rs`](../src/vm/mod.rs) 中实现了：

1. **`NewStruct`** - 创建结构体实例
   - 从栈中弹出字段值
   - 弹出结构体名称
   - 创建 `StructValue` 并压栈

2. **`FieldGet`** - 获取字段值
   - 从栈中弹出结构体
   - 根据索引获取字段值
   - 将字段值压栈

3. **`FieldSet`** - 设置字段值
   - 从栈中弹出新值和结构体
   - 更新指定字段
   - 将更新后的结构体和值压栈

## 编译器集成

在 [`src/compiler/mod.rs`](../src/compiler/mod.rs) 中添加了结构体语句的占位符处理，为完整的字节码生成做准备。

## 解释器集成

在 [`src/interpreter/mod.rs`](../src/interpreter/mod.rs) 中添加了结构体表达式的占位符处理。

## 使用示例

参见 [`lang-spec/examples/structs.zero`](../lang-spec/examples/structs.zero) 获取完整的使用示例。

```rust
// 定义结构体
struct Person {
    name: String,
    age: Integer
};

// 创建实例
let alice = Person {
    name: "Alice",
    age: 30
};

// 访问字段
print(alice.name);  // 输出: Alice

// 修改字段
alice.age = 31;
print(alice.age);   // 输出: 31
```

## 限制和未来工作

当前实现的限制：

1. **编译器生成** - 结构体的完整字节码生成尚未实现
2. **方法支持** - 尚不支持结构体方法（impl块）
3. **继承** - 不支持结构体继承或trait
4. **泛型** - 不支持泛型结构体

未来计划：

1. 完善编译器中的结构体字节码生成
2. 实现 `impl` 块支持结构体方法
3. 添加构造函数和析构函数
4. 支持嵌套结构体
5. 实现结构体的序列化和反序列化
6. 添加更多的结构体操作符重载

## 技术细节

### 内存布局

结构体在运行时表示为：
- 结构体名称（String）
- 字段值数组（Vec<Value>），按声明顺序存储

### 类型系统

结构体类型包括：
- 命名结构体：`Type::Struct(StructType)`
- 类型别名：`Type::Named(String)`

类型检查器在符号表中存储结构体定义，用于：
- 验证字段访问
- 类型推断
- 赋值类型检查

### 字节码生成

结构体相关操作的字节码生成流程：

1. **创建结构体**：
   - 为每个字段生成表达式代码
   - 生成结构体名称常量加载
   - 生成 `NewStruct` 指令

2. **字段访问**：
   - 生成对象表达式代码
   - 确定字段索引
   - 生成 `FieldGet` 指令

3. **字段赋值**：
   - 生成对象和值表达式代码
   - 确定字段索引
   - 生成 `FieldSet` 指令

## 测试

建议的测试用例：

1. 基本结构体声明和实例化
2. 字段访问和赋值
3. 嵌套结构体
4. 类型别名
5. 匿名结构体
6. 错误情况（未定义字段、类型不匹配等）

## 参考

- [ARCHITECTURE.md](ARCHITECTURE.md) - 编译器整体架构
- [TYPE_SYSTEM.md](TYPE_SYSTEM.md) - 类型系统文档
- [BYTECODE_FORMAT.md](BYTECODE_FORMAT.md) - 字节码格式