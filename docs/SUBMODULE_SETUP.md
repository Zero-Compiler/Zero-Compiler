# Zero编译器 Submodule 配置指南

## 概述

Zero编译器使用Git Submodule管理错误消息配置，将错误消息独立维护在 [Zero-Lang-Error-Msg](https://github.com/Zero-Compiler/Zero-Lang-Error-Msg) 仓库中。

## 为什么使用 Submodule？

1. **独立维护**：错误消息可以独立于编译器代码进行更新和维护
2. **多语言支持**：支持多种语言的错误消息，方便国际化
3. **版本管理**：可以独立版本控制错误消息的变更
4. **团队协作**：不同团队可以独立维护代码和文档
5. **复用性**：其他项目可以复用相同的错误消息配置

## 项目结构

```
Zero-compiler/
├── error-msg/                    # Git Submodule
│   ├── locale/
│   │   ├── zh_CN/
│   │   │   └── error_messages.toml
│   │   └── en_US/               # 未来支持
│   │       └── error_messages.toml
│   └── README.md
├── src/
│   └── error/
│       └── mod.rs               # 从 submodule 加载配置
└── docs/
    ├── ERROR_HANDLING.md
    └── SUBMODULE_SETUP.md       # 本文档
```

## 初次设置

### 方式1：克隆时包含 submodule

```bash
git clone --recurse-submodules https://github.com/Zero-Compiler/Zero-compiler.git
cd Zero-compiler
```

### 方式2：已克隆后初始化 submodule

```bash
git clone https://github.com/Zero-Compiler/Zero-compiler.git
cd Zero-compiler
git submodule init
git submodule update
```

### 方式3：添加新的 submodule（仅维护者）

```bash
cd Zero-compiler
git submodule add https://github.com/Zero-Compiler/Zero-Lang-Error-Msg.git error-msg
git add .gitmodules error-msg
git commit -m "chore: Add Zero-Lang-Error-Msg submodule"
git push
```

## 日常使用

### 更新 submodule 到最新版本

```bash
cd error-msg
git pull origin main
cd ..
git add error-msg
git commit -m "chore: Update error-msg to latest version"
git push
```

### 切换到特定版本

```bash
cd error-msg
git checkout v1.0.0  # 切换到特定标签
cd ..
git add error-msg
git commit -m "chore: Pin error-msg to v1.0.0"
```

### 查看 submodule 状态

```bash
git submodule status
```

输出示例：
```
 abc1234 error-msg (heads/main)
```

### 更新所有 submodule

```bash
git submodule update --remote
```

## 开发流程

### 修改错误消息（推荐流程）

1. **Fork error-msg 仓库**
   ```bash
   # 在 GitHub 上 fork Zero-Lang-Error-Msg
   ```

2. **克隆并修改**
   ```bash
   cd error-msg
   git remote add fork https://github.com/YOUR_USERNAME/Zero-Lang-Error-Msg.git
   git checkout -b feature/add-new-error
   
   # 编辑 locale/zh_CN/error_messages.toml
   vim locale/zh_CN/error_messages.toml
   
   git add locale/zh_CN/error_messages.toml
   git commit -m "feat: Add L006 error message"
   git push fork feature/add-new-error
   ```

3. **创建 Pull Request**
   - 在 GitHub 上创建 PR 到 Zero-Lang-Error-Msg 主仓库
   - 等待审核和合并

4. **更新主项目**
   ```bash
   cd ..  # 返回 Zero-compiler 目录
   cd error-msg
   git pull origin main
   cd ..
   git add error-msg
   git commit -m "chore: Update error-msg with new L006 error"
   git push
   ```

### 本地测试（不推送到 error-msg）

如果只想本地测试错误消息：

```bash
cd error-msg
# 直接修改文件
vim locale/zh_CN/error_messages.toml

# 在 Zero-compiler 中测试
cd ..
cargo test
cargo run --bin Zero-compiler lang-spec/examples/error_test.zero --dtl

# 注意：不要提交这些本地修改到 error-msg
cd error-msg
git checkout .  # 撤销本地修改
```

## 常见问题

### Q: 为什么我的 error-msg 目录是空的？

A: 你可能忘记初始化 submodule。运行：
```bash
git submodule init
git submodule update
```

### Q: 如何查看当前使用的 error-msg 版本？

A: 运行：
```bash
cd error-msg
git log -1
```

### Q: submodule 更新后编译失败怎么办？

A: 可能是错误消息格式变更。检查：
1. `error-msg/locale/zh_CN/error_messages.toml` 格式是否正确
2. 是否有新的必需字段
3. 尝试重新编译：`cargo clean && cargo build`

### Q: 能不能不使用 submodule？

A: 技术上可以，但不推荐。如果确实需要：
1. 删除 submodule：`git rm error-msg`
2. 手动复制错误消息文件到项目中
3. 修改 `src/error/mod.rs` 中的路径

### Q: 如何为新语言添加翻译？

A: 
1. Fork error-msg 仓库
2. 创建新的语言目录：
   ```bash
   cd error-msg
   mkdir -p locale/en_US
   cp locale/zh_CN/error_messages.toml locale/en_US/
   # 翻译文件内容
   ```
3. 提交 PR
4. 在代码中使用：
   ```rust
   let registry = ErrorRegistry::from_locale("en_US")?;
   ```

## Submodule 工作原理

### .gitmodules 文件

```ini
[submodule "error-msg"]
    path = error-msg
    url = https://github.com/Zero-Compiler/Zero-Lang-Error-Msg.git
```

这个文件定义了：
- submodule 的本地路径
- submodule 的远程仓库 URL

### Git 如何跟踪 submodule

Git 不会存储 submodule 的完整内容，而是：
1. 在 `.gitmodules` 中记录 submodule 的元数据
2. 在 `error-msg/` 目录中记录一个特定的 commit hash
3. 当你克隆主仓库时，需要显式初始化和更新 submodule

### Submodule 的独立性

- error-msg 有自己的 `.git` 目录
- 可以独立进行 git 操作（commit, push, pull 等）
- 主项目只跟踪 submodule 的 commit hash

## 代码集成

### 在 Rust 中使用

```rust
// src/error/mod.rs

impl ErrorRegistry {
    /// 从 submodule 加载默认配置（中文）
    pub fn default() -> Self {
        const DEFAULT_CONFIG: &str = include_str!("../../error-msg/locale/zh_CN/error_messages.toml");
        Self::from_toml(DEFAULT_CONFIG).expect("Failed to load error messages from submodule")
    }
    
    /// 从指定语言加载
    pub fn from_locale(locale: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = format!("error-msg/locale/{}/error_messages.toml", locale);
        let config_str = std::fs::read_to_string(&path)?;
        Self::from_toml(&config_str)
    }
}
```

### 编译时嵌入

使用 `include_str!` 宏在编译时将配置文件嵌入到二进制文件中：
- ✅ 无需运行时文件系统访问
- ✅ 单个可执行文件包含所有内容
- ✅ 更好的性能

## 最佳实践

1. **定期更新**：定期更新 submodule 以获取最新的错误消息
2. **版本固定**：生产环境建议固定到特定版本（tag）
3. **文档同步**：更新错误消息时同步更新相关文档
4. **测试覆盖**：添加新错误类型时同时添加测试用例
5. **审核流程**：所有错误消息变更都应通过 PR 审核

## 相关链接

- [Zero-Lang-Error-Msg 仓库](https://github.com/Zero-Compiler/Zero-Lang-Error-Msg)
- [错误处理文档](ERROR_HANDLING.md)
- [Git Submodule 官方文档](https://git-scm.com/book/en/v2/Git-Tools-Submodules)

## 维护者信息

如有问题或建议，请联系：
- 提 Issue 到 [Zero-compiler](https://github.com/Zero-Compiler/Zero-compiler/issues)
- 提 Issue 到 [Zero-Lang-Error-Msg](https://github.com/Zero-Compiler/Zero-Lang-Error-Msg/issues)