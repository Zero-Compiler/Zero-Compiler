use crate::lexer::token::Token;

// 类型系统定义
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Char,    // 字符类型
    Void,
    Null,
    Array(Box<Type>),  // 数组类型
    Function(FunctionType),
    Struct(StructType),  // 结构体类型
    Named(String),  // 类型别名引用
    Unknown,  // 用于类型推导
}

// 结构体字段定义
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
}

// 结构体类型定义
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructType {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
}

// 函数参数定义
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<Type>,
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }
    
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match (self, other) {
            // 相同类型
            (a, b) if a == b => true,
            // 数字类型之间兼容
            (a, b) if a.is_numeric() && b.is_numeric() => true,
            // Unknown类型与任何类型兼容
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            // 数组类型需要元素类型兼容
            (Type::Array(a), Type::Array(b)) => a.is_compatible_with(b),
            // 结构体类型需要名称和字段匹配
            (Type::Struct(a), Type::Struct(b)) => a == b,
            _ => false,
        }
    }
    
    pub fn get_element_type(&self) -> Option<&Type> {
        match self {
            Type::Array(element_type) => Some(element_type),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // 字面量
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Char(char),      // 字符字面量
    Identifier(String),

    // 路径表达式 (module::item 或 module::submodule::item)
    Path {
        segments: Vec<String>,  // ["math", "geometry", "area"]
    },

    // 数组字面量
    Array {
        elements: Vec<Expr>,
    },
    
    // 结构体字面量
    StructLiteral {
        struct_name: String,
        fields: Vec<(String, Expr)>,  // (字段名, 字段值)
    },
    
    // 二元运算
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    
    // 一元运算
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },
    
    // 函数调用
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    
    // 数组/索引访问
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    
    // 索引赋值
    IndexAssign {
        object: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
    },
    
    // 赋值
    Assign {
        name: String,
        value: Box<Expr>,
    },
    
    // 字段访问 (object.field)
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },

    // 字段赋值
    FieldAssign {
        object: Box<Expr>,
        field: String,
        value: Box<Expr>,
    },

    // 方法调用 (object.method(args))
    MethodCall {
        object: Box<Expr>,
        method: String,
        arguments: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // 算术运算符
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    
    // 比较运算符
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    
    // 逻辑运算符
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // 表达式语句
    Expression(Expr),
    
    // 变量声明
    VarDeclaration {
        name: String,
        mutable: bool,
        type_annotation: Option<Type>,
        initializer: Option<Expr>,
    },
    
    // 函数声明
    FnDeclaration {
        visibility: Visibility,  // 新增：可见性
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
    },
    
    // 结构体声明
    StructDeclaration {
        visibility: Visibility,  // 新增：可见性
        name: String,
        fields: Vec<StructField>,
    },
    
    // 类型别名声明
    TypeAlias {
        visibility: Visibility,  // 新增：可见性
        name: String,
        target_type: Type,
    },
    
    // 返回语句
    Return {
        value: Option<Expr>,
    },
    
    // if 语句
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    
    // while 循环
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    
    // for 循环
    For {
        variable: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
    },
    
    // 打印语句
    Print {
        value: Expr,
    },
    
    // 代码块
    Block {
        statements: Vec<Stmt>,
    },

    // Break 语句（仅在循环中有效）
    Break,

    // Continue 语句（仅在循环中有效）
    Continue,

    // Impl 块（方法实现）
    ImplBlock {
        type_name: String,
        methods: Vec<MethodDeclaration>,
    },

    // 模块声明
    ModuleDeclaration {
        name: String,
        statements: Vec<Stmt>,
        is_public: bool,
    },

    // 导入语句
    UseStatement {
        path: Vec<String>,  // 模块路径，如 ["math", "geometry"]
        items: UseItems,
    },

    // 模块引用（从文件加载）
    ModuleReference {
        name: String,       // 模块名（对应文件名）
        is_public: bool,    // 是否公开
    },
}

/// 导入项类型
#[derive(Debug, Clone, PartialEq)]
pub enum UseItems {
    All,                              // use math::*
    Single(String),                   // use math::add
    Multiple(Vec<String>),            // use math::{add, sub}
    Renamed(String, String),          // use math::add as plus
}

/// 可见性修饰符
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,    // pub
    Private,   // 默认（无修饰符）
}

/// 方法声明（与函数类似，但有隐式的 self 参数）
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDeclaration {
    pub name: String,
    pub parameters: Vec<Parameter>,  // 不包含 self
    pub return_type: Option<Type>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            statements: Vec::new(),
        }
    }
    
    pub fn add_statement(&mut self, stmt: Stmt) {
        self.statements.push(stmt);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

// 辅助函数用于创建表达式
impl Expr {
    pub fn integer(value: i64) -> Self {
        Expr::Integer(value)
    }
    
    pub fn float(value: f64) -> Self {
        Expr::Float(value)
    }
    
    pub fn string(value: String) -> Self {
        Expr::String(value)
    }
    
    pub fn boolean(value: bool) -> Self {
        Expr::Boolean(value)
    }
    
    pub fn identifier(name: String) -> Self {
        Expr::Identifier(name)
    }
    
    pub fn array(elements: Vec<Expr>) -> Self {
        Expr::Array { elements }
    }
    
    pub fn binary(left: Expr, operator: BinaryOp, right: Expr) -> Self {
        Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
    
    pub fn unary(operator: UnaryOp, operand: Expr) -> Self {
        Expr::Unary {
            operator,
            operand: Box::new(operand),
        }
    }
    
    pub fn call(callee: Expr, arguments: Vec<Expr>) -> Self {
        Expr::Call {
            callee: Box::new(callee),
            arguments,
        }
    }
    
    pub fn index(object: Expr, index: Expr) -> Self {
        Expr::Index {
            object: Box::new(object),
            index: Box::new(index),
        }
    }
    
    pub fn index_assign(object: Expr, index: Expr, value: Expr) -> Self {
        Expr::IndexAssign {
            object: Box::new(object),
            index: Box::new(index),
            value: Box::new(value),
        }
    }
    
    pub fn assign(name: String, value: Expr) -> Self {
        Expr::Assign {
            name,
            value: Box::new(value),
        }
    }
    
    pub fn struct_literal(struct_name: String, fields: Vec<(String, Expr)>) -> Self {
        Expr::StructLiteral {
            struct_name,
            fields,
        }
    }
    
    pub fn field_access(object: Expr, field: String) -> Self {
        Expr::FieldAccess {
            object: Box::new(object),
            field,
        }
    }
    
    pub fn field_assign(object: Expr, field: String, value: Expr) -> Self {
        Expr::FieldAssign {
            object: Box::new(object),
            field,
            value: Box::new(value),
        }
    }

    pub fn method_call(object: Expr, method: String, arguments: Vec<Expr>) -> Self {
        Expr::MethodCall {
            object: Box::new(object),
            method,
            arguments,
        }
    }
}