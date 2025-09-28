use crate::runtime::values::Value;

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub main: Block,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Void,
    Array(Box<Type>), // Array with element type
}

impl Type {
    // Function from_string removed as it was unused
    
    pub fn to_string(&self) -> String {
        match self {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::String => "string".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
            Type::Array(element_type) => format!("array[{}]", element_type.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Declaration {
        name: String,
        var_type: Type,
        value: Expression,
    },
    Assignment {
        name: String,
        value: Expression,
    },
    ArrayAssignment {
        name: String,
        index: Expression,
        value: Expression,
    },
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        condition: Expression,
        body: Block,
    },
    For {
        init: Box<Statement>,
        condition: Expression,
        increment: Box<Statement>,
        body: Block,
    },
    Return {
        value: Option<Expression>,
    },
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Value),
    Variable(String),
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    BuiltinCall {
        name: String,
        arguments: Vec<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    ArrayLiteral {
        elements: Vec<Expression>,
    },
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo, // Operador módulo %
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    Minus,
}