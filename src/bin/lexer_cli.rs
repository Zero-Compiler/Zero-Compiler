//! Zero语言词法分析器CLI工具
//! 支持批量处理文件和格式化token输出

use Zero_compiler::lexer::{Lexer, TokenPreprocessor};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "tokenize" => {
            if args.len() < 3 {
                eprintln!("Usage: {} tokenize <file.zero|pattern>", args[0]);
                std::process::exit(1);
            }
            tokenize_files(&args[2]);
        }
        "batch" => {
            if args.len() < 4 {
                eprintln!("Usage: {} batch <input_pattern> <output_dir>", args[0]);
                std::process::exit(1);
            }
            batch_process(&args[2], &args[3]);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    println!("Zero Lexer CLI Tool");
    println!("Usage:");
    println!("  {} tokenize <file.zero>        - Tokenize a single file", program);
    println!("  {} tokenize <pattern>          - Tokenize files matching pattern", program);
    println!("  {} batch <pattern> <out_dir>   - Batch process and save tokens", program);
    println!();
    println!("Examples:");
    println!("  {} tokenize lang-spec/examples/hello.zero", program);
    println!("  {} tokenize 'lang-spec/examples/*.zero'", program);
    println!("  {} batch 'src/**/*.zero' output/tokens", program);
}

fn tokenize_files(pattern: &str) {
    let paths = match find_files(pattern) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error finding files: {}", e);
            std::process::exit(1);
        }
    };

    if paths.is_empty() {
        eprintln!("No files found matching pattern: {}", pattern);
        std::process::exit(1);
    }

    for path in paths {
        println!("\n{}", "=".repeat(60));
        println!("File: {}", path.display());
        println!("{}", "=".repeat(60));
        
        if let Err(e) = tokenize_file(&path) {
            eprintln!("Error processing {}: {}", path.display(), e);
        }
    }
}

fn tokenize_file(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()
        .map_err(|e| format!("Lexer error: {}", e))?;

    // 应用预处理器
    let tokens = TokenPreprocessor::preprocess(tokens);

    println!("\nTokens ({} total):", tokens.len());
    println!("{:-<60}", "");
    
    for (i, token) in tokens.iter().enumerate() {
        println!("{:4} | {:20} | {:15?} | {}:{}",
            i + 1,
            format!("'{}'", token.value),
            token.token_type,
            token.start_pos.line,
            token.start_pos.column
        );
    }

    Ok(())
}

fn batch_process(pattern: &str, output_dir: &str) {
    let paths = match find_files(pattern) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error finding files: {}", e);
            std::process::exit(1);
        }
    };

    if paths.is_empty() {
        eprintln!("No files found matching pattern: {}", pattern);
        std::process::exit(1);
    }

    // 创建输出目录
    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Error creating output directory: {}", e);
        std::process::exit(1);
    }

    println!("Processing {} files...", paths.len());
    
    let mut success_count = 0;
    let mut error_count = 0;

    for path in paths {
        let output_path = Path::new(output_dir)
            .join(path.file_name().unwrap())
            .with_extension("tokens");

        match process_and_save(&path, &output_path) {
            Ok(_) => {
                println!("✓ {} -> {}", path.display(), output_path.display());
                success_count += 1;
            }
            Err(e) => {
                eprintln!("✗ {}: {}", path.display(), e);
                error_count += 1;
            }
        }
    }

    println!("\nCompleted: {} successful, {} errors", success_count, error_count);
}

fn process_and_save(input_path: &Path, output_path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()
        .map_err(|e| format!("Lexer error: {}", e))?;

    // 应用预处理器
    let tokens = TokenPreprocessor::preprocess(tokens);

    // 写入格式化的token文件
    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    writeln!(output, "# Token Analysis for: {}", input_path.display())
        .map_err(|e| format!("Write error: {}", e))?;
    writeln!(output, "# Total tokens: {}", tokens.len())
        .map_err(|e| format!("Write error: {}", e))?;
    writeln!(output, "# {}", "=".repeat(70))
        .map_err(|e| format!("Write error: {}", e))?;
    writeln!(output)
        .map_err(|e| format!("Write error: {}", e))?;

    writeln!(output, "{:<6} {:<25} {:<20} {:<15}", "Index", "Value", "Type", "Position")
        .map_err(|e| format!("Write error: {}", e))?;
    writeln!(output, "{}", "-".repeat(70))
        .map_err(|e| format!("Write error: {}", e))?;

    for (i, token) in tokens.iter().enumerate() {
        writeln!(output, "{:<6} {:<25} {:<20?} {}:{}",
            i + 1,
            format!("'{}'", token.value),
            token.token_type,
            token.start_pos.line,
            token.start_pos.column
        ).map_err(|e| format!("Write error: {}", e))?;
    }

    Ok(())
}

fn find_files(pattern: &str) -> Result<Vec<PathBuf>, String> {
    // 如果是单个文件，直接返回
    let path = Path::new(pattern);
    if path.exists() && path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    // 否则作为glob模式处理
    let mut paths = Vec::new();
    
    // 在Unix系统上需要使用shell展开通配符
    // 在Windows上glob可以直接工作
    #[cfg(unix)]
    {
        use std::process::Command;
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("ls {}", pattern))
            .output()
            .map_err(|e| format!("Failed to expand pattern: {}", e))?;
        
        if output.status.success() {
            let files = String::from_utf8_lossy(&output.stdout);
            for line in files.lines() {
                let p = PathBuf::from(line.trim());
                if p.exists() && p.is_file() {
                    paths.push(p);
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        for entry in glob::glob(pattern)
            .map_err(|e| format!("Invalid pattern: {}", e))? {
            match entry {
                Ok(p) => {
                    if p.is_file() {
                        paths.push(p);
                    }
                }
                Err(e) => eprintln!("Error reading entry: {}", e),
            }
        }
    }

    Ok(paths)
}