mod lexer;
mod parser;
mod ast;
mod bytecode;
mod compiler;
mod vm;
mod type_checker;
mod error;
mod module_loader;

// 保留旧的解释器用于对比
mod interpreter;

use lexer::Lexer;
use parser::Parser;
use compiler::Compiler;
use vm::VM;
use type_checker::TypeChecker;
use bytecode::serializer::{BytecodeSerializer, BytecodeDeserializer};
use error::{ErrorMode, ErrorDisplayer};
use module_loader::ModuleLoader;
use ast::{Program, Stmt};
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::process;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <source_file.zero> [--dtl]", args[0]);
        eprintln!("       {} --old <source_file.zero> [--dtl]  (use old interpreter)", args[0]);
        eprintln!("       {} --compile <source_file.zero> <output.zbc> [--dtl]  (compile to bytecode)", args[0]);
        eprintln!("       {} --run <bytecode_file.zbc>  (run bytecode file)", args[0]);
        eprintln!("");
        eprintln!("Options:");
        eprintln!("  --dtl    显示详细的错误信息（包含源码片段和修复建议）");
        process::exit(1);
    }

    // 检查是否有 --dtl 标志
    let error_mode = if args.contains(&"--dtl".to_string()) {
        ErrorMode::Detailed
    } else {
        ErrorMode::Simple
    };

    match args[1].as_str() {
        "--old" => {
            if args.len() < 3 {
                eprintln!("Usage: {} --old <source_file.zero> [--dtl]", args[0]);
                process::exit(1);
            }
            let source = read_source_file(&args[2]);
            println!("Using old tree-walking interpreter...");
            run_old(&source, error_mode);
        }
        "--compile" => {
            if args.len() < 4 {
                eprintln!("Usage: {} --compile <source_file.zero> <output.zbc> [--dtl]", args[0]);
                process::exit(1);
            }
            let source = read_source_file(&args[2]);
            compile_to_bytecode(&source, &args[3], error_mode);
        }
        "--run" => {
            if args.len() < 3 {
                eprintln!("Usage: {} --run <bytecode_file.zbc>", args[0]);
                process::exit(1);
            }
            run_bytecode_file(&args[2]);
        }
        _ => {
            let filename = &args[1];
            let source = read_source_file(filename);
            println!("Using bytecode compiler + VM...");
            run(&source, filename, error_mode);
        }
    }
}

fn read_source_file(filename: &str) -> String {
    match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    }
}

/// 编译源代码到字节码文件
fn compile_to_bytecode(source: &str, output_file: &str, error_mode: ErrorMode) {
    println!("Compiling {} to {}...", "source", output_file);

    // 词法分析
    let mut lexer = Lexer::new(source.to_string());
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(err) => {
            let displayer = ErrorDisplayer::new(error_mode);
            eprintln!("{}", displayer.format_error(&err, Some(source)));
            process::exit(1);
        }
    };

    // 预处理tokens（处理科学计数法等）
    let tokens = lexer::TokenPreprocessor::preprocess(tokens);

    // 语法分析
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(prog) => prog,
        Err(err) => {
            eprintln!("Parse error: {:?}", err);
            process::exit(1);
        }
    };

    // 类型检查
    let mut type_checker = TypeChecker::new();
    if let Err(err) = type_checker.check(&program) {
        eprintln!("Type error: {:?}", err);
        process::exit(1);
    }

    // 获取导入符号映射
    let imported_symbols = type_checker.get_imported_symbols();

    // 编译为字节码
    let mut compiler = Compiler::new();
    compiler.set_imported_symbols(imported_symbols);
    let chunk = match compiler.compile(program) {
        Ok(chunk) => chunk,
        Err(err) => {
            eprintln!("Compile error: {:?}", err);
            process::exit(1);
        }
    };

    // 序列化并保存
    let file = match File::create(output_file) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error creating output file: {}", err);
            process::exit(1);
        }
    };

    let mut writer = BufWriter::new(file);
    if let Err(err) = BytecodeSerializer::serialize(&chunk, &mut writer) {
        eprintln!("Error serializing bytecode: {}", err);
        process::exit(1);
    }

    println!("Successfully compiled to {}", output_file);
}

/// 从字节码文件运行
fn run_bytecode_file(filename: &str) {
    println!("Loading bytecode from {}...", filename);
    
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error opening bytecode file: {}", err);
            process::exit(1);
        }
    };

    let mut reader = BufReader::new(file);
    let chunk = match BytecodeDeserializer::deserialize(&mut reader) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error deserializing bytecode: {}", err);
            process::exit(1);
        }
    };

    println!("Running bytecode...");
    
    // 调试：打印反汇编代码
    if env::var("ZERO_DEBUG").is_ok() {
        chunk.disassemble("loaded");
    }

    // VM执行
    let mut vm = VM::new();
    if let Err(err) = vm.execute(chunk) {
        eprintln!("Runtime error: {:?}", err);
        process::exit(1);
    }
}

/// 解析程序中的模块引用，将 ModuleReference 转换为 ModuleDeclaration
fn resolve_module_references(program: Program, source_file_path: &str) -> Result<Program, String> {
    let mut loader = ModuleLoader::new();

    // 添加搜索路径：源文件所在目录和当前工作目录
    if let Some(parent) = PathBuf::from(source_file_path).parent() {
        loader.add_search_path(parent);
    }
    loader.add_search_path(".");

    let mut resolved_statements = Vec::new();

    for stmt in program.statements {
        match stmt {
            Stmt::ModuleReference { name, is_public } => {
                // 加载模块文件
                match loader.load_module(&name) {
                    Ok(module_program) => {
                        // 将加载的模块转换为内联模块声明
                        resolved_statements.push(Stmt::ModuleDeclaration {
                            name,
                            statements: module_program.statements,
                            is_public,
                        });
                    }
                    Err(err) => {
                        return Err(format!("Failed to load module '{}': {:?}", name, err));
                    }
                }
            }
            _ => {
                resolved_statements.push(stmt);
            }
        }
    }

    Ok(Program {
        statements: resolved_statements,
    })
}

/// 新的字节码编译器 + VM执行
fn run(source: &str, source_file: &str, error_mode: ErrorMode) {
    // 词法分析
    let mut lexer = Lexer::new(source.to_string());
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(err) => {
            let displayer = ErrorDisplayer::new(error_mode);
            eprintln!("{}", displayer.format_error(&err, Some(source)));
            process::exit(1);
        }
    };

    // 预处理tokens（处理科学计数法等）
    let tokens = lexer::TokenPreprocessor::preprocess(tokens);

    // 语法分析
    let mut parser = Parser::new(tokens);
    let mut program = match parser.parse() {
        Ok(prog) => prog,
        Err(err) => {
            eprintln!("Parse error: {:?}", err);
            process::exit(1);
        }
    };

    // 解析模块引用（将 mod name; 转换为实际加载的模块）
    program = match resolve_module_references(program, source_file) {
        Ok(prog) => prog,
        Err(err) => {
            eprintln!("Module resolution error: {}", err);
            process::exit(1);
        }
    };

    // 类型检查
    let mut type_checker = TypeChecker::new();
    if let Err(err) = type_checker.check(&program) {
        eprintln!("Type error: {:?}", err);
        process::exit(1);
    }

    // 获取导入符号映射
    let imported_symbols = type_checker.get_imported_symbols();

    // 编译为字节码
    let mut compiler = Compiler::new();
    compiler.set_imported_symbols(imported_symbols);
    let chunk = match compiler.compile(program) {
        Ok(chunk) => chunk,
        Err(err) => {
            eprintln!("Compile error: {:?}", err);
            process::exit(1);
        }
    };

    // 调试：打印反汇编代码
    if env::var("ZERO_DEBUG").is_ok() {
        chunk.disassemble("main");
    }

    // VM执行
    let mut vm = VM::new();
    if let Err(err) = vm.execute(chunk) {
        eprintln!("Runtime error: {:?}", err);
        process::exit(1);
    }
}

/// 旧的树遍历解释器（用于对比）
fn run_old(source: &str, error_mode: ErrorMode) {
    // 词法分析
    let mut lexer = Lexer::new(source.to_string());
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(err) => {
            let displayer = ErrorDisplayer::new(error_mode);
            eprintln!("{}", displayer.format_error(&err, Some(source)));
            process::exit(1);
        }
    };

    // 预处理tokens（处理科学计数法等）
    let tokens = lexer::TokenPreprocessor::preprocess(tokens);

    // 语法分析
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(prog) => prog,
        Err(err) => {
            eprintln!("Parse error: {:?}", err);
            process::exit(1);
        }
    };

    // 解释执行
    let mut interpreter = interpreter::Interpreter::new();
    if let Err(err) = interpreter.interpret(program) {
        eprintln!("Runtime error: {:?}", err);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let source = r#"
            let x = 10;
            let y = 20;
            print(x + y);
        "#;
        run(source, "test.zero", ErrorMode::Simple);
    }

    #[test]
    fn test_function() {
        let source = r#"
            fn add(a, b) {
                return a + b;
            }

            let result = add(5, 3);
            print(result);
        "#;
        run(source, "test.zero", ErrorMode::Simple);
    }

    #[test]
    fn test_bytecode_vs_interpreter() {
        let source = r#"
            let x = 100;
            let y = 200;
            print(x + y);
        "#;

        println!("\n=== Bytecode VM ===");
        run(source, "test.zero", ErrorMode::Simple);

        println!("\n=== Old Interpreter ===");
        run_old(source, ErrorMode::Simple);
    }

    #[test]
    fn test_control_flow() {
        let source = r#"
            let x = 15;
            if x > 10 {
                print(x);
            }

            let i = 0;
            while i < 3 {
                print(i);
                i = i + 1;
            }
        "#;
        run(source, "test.zero", ErrorMode::Simple);
    }

    #[test]
    fn test_functions() {
        let source = r#"
            fn multiply(a, b) {
                return a * b;
            }

            fn factorial(n) {
                if n <= 1 {
                    return 1;
                }
                return n * factorial(n - 1);
            }

            print(multiply(6, 7));
            print(factorial(5));
        "#;
        run(source, "test.zero", ErrorMode::Simple);
    }

    #[test]
    fn test_type_annotations() {
        let source = r#"
            let x: int = 42;
            let y: float = 3.14;
            let s: string = "hello";
            let b: bool = true;
            print(x);
            print(y);
            print(s);
            print(b);
        "#;
        run(source, "test.zero", ErrorMode::Simple);
    }

    #[test]
    fn test_typed_function() {
        let source = r#"
            fn add(a: int, b: int) {
                return a + b;
            }

            let result = add(10, 20);
            print(result);
        "#;
        run(source, "test.zero", ErrorMode::Simple);
    }

    #[test]
    fn test_mixed_type_annotations() {
        let source = r#"
            fn multiply(a, b: int) {
                return a * b;
            }

            let x = 5;
            let result = multiply(x, 10);
            print(result);
        "#;
        run(source, "test.zero", ErrorMode::Simple);
    }

}
