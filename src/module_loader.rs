use crate::ast::Program;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// 模块加载错误
#[derive(Debug)]
pub enum LoadError {
    ModuleNotFound(String),
    CircularDependency(String),
    IoError(std::io::Error),
    LexerError(String),
    ParseError(String),
}

impl From<std::io::Error> for LoadError {
    fn from(err: std::io::Error) -> Self {
        LoadError::IoError(err)
    }
}

impl From<crate::lexer::LexerError> for LoadError {
    fn from(err: crate::lexer::LexerError) -> Self {
        LoadError::LexerError(format!("{:?}", err))
    }
}

impl From<crate::parser::ParseError> for LoadError {
    fn from(err: crate::parser::ParseError) -> Self {
        LoadError::ParseError(format!("{:?}", err))
    }
}

pub type LoadResult<T> = Result<T, LoadError>;

/// 模块加载器
///
/// 负责从文件系统加载模块文件，解析为 AST，并检测循环依赖
pub struct ModuleLoader {
    /// 模块搜索路径
    search_paths: Vec<PathBuf>,

    /// 已加载的模块缓存 (模块名 -> Program)
    loaded_modules: HashMap<String, Program>,

    /// 正在加载的模块栈（用于循环依赖检测）
    loading_stack: Vec<String>,

    /// 所有已访问过的模块（用于避免重复加载）
    visited: HashSet<String>,
}

impl ModuleLoader {
    /// 创建新的模块加载器
    pub fn new() -> Self {
        ModuleLoader {
            search_paths: Vec::new(),
            loaded_modules: HashMap::new(),
            loading_stack: Vec::new(),
            visited: HashSet::new(),
        }
    }

    /// 添加模块搜索路径
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// 加载模块
    ///
    /// 查找规则:
    /// 1. mod math; → math.zero
    /// 2. mod math; → math/mod.zero
    /// 3. 在所有 search_paths 中查找
    pub fn load_module(&mut self, name: &str) -> LoadResult<Program> {
        // 检查是否已经加载过
        if let Some(program) = self.loaded_modules.get(name) {
            return Ok(program.clone());
        }

        // 检测循环依赖
        if self.loading_stack.contains(&name.to_string()) {
            let cycle = self.build_cycle_message(name);
            return Err(LoadError::CircularDependency(cycle));
        }

        // 标记为正在加载
        self.loading_stack.push(name.to_string());
        self.visited.insert(name.to_string());

        // 查找模块文件
        let file_path = self.find_module_file(name)?;

        // 读取源码
        let source = fs::read_to_string(&file_path)?;

        // 词法分析
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;

        // 预处理 tokens
        let tokens = crate::lexer::TokenPreprocessor::preprocess(tokens);

        // 语法分析
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        // 缓存模块
        self.loaded_modules.insert(name.to_string(), program.clone());

        // 从加载栈中移除
        self.loading_stack.pop();

        Ok(program)
    }

    /// 查找模块文件
    ///
    /// 尝试以下路径（按顺序）：
    /// 1. <search_path>/<name>.zero
    /// 2. <search_path>/<name>/mod.zero
    fn find_module_file(&self, name: &str) -> LoadResult<PathBuf> {
        for search_path in &self.search_paths {
            // 尝试 name.zero
            let mut path = search_path.join(name);
            path.set_extension("zero");
            if path.exists() && path.is_file() {
                return Ok(path);
            }

            // 尝试 name/mod.zero
            let path = search_path.join(name).join("mod.zero");
            if path.exists() && path.is_file() {
                return Ok(path);
            }
        }

        Err(LoadError::ModuleNotFound(format!(
            "Module '{}' not found in search paths: {:?}",
            name, self.search_paths
        )))
    }

    /// 构建循环依赖错误消息
    fn build_cycle_message(&self, current_module: &str) -> String {
        let mut cycle = Vec::new();

        // 找到循环的起点
        let mut found_start = false;
        for module in &self.loading_stack {
            if module == current_module {
                found_start = true;
            }
            if found_start {
                cycle.push(module.clone());
            }
        }

        cycle.push(current_module.to_string());

        format!("Circular dependency detected: {}", cycle.join(" → "))
    }

    /// 获取已加载的模块数量
    pub fn loaded_count(&self) -> usize {
        self.loaded_modules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_loader_creation() {
        let loader = ModuleLoader::new();
        assert_eq!(loader.loaded_count(), 0);
    }

    #[test]
    fn test_add_search_path() {
        let mut loader = ModuleLoader::new();
        loader.add_search_path("./test");
        assert_eq!(loader.search_paths.len(), 1);
    }
}
