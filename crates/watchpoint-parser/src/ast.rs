use crate::bytecode::Type;

#[derive(Debug, Clone)]
pub struct Script {
    // pub inputs: Vec<InputPath>,
    pub variables: Vec<VariableDecl>,
    pub init: Vec<Statement>,
    pub logic: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct VariableDecl {
    pub ty: Type,
    pub name: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    Always {
        block: Vec<Statement>
    },
    Assignment {
        var_name: String,
        value: Expression,
    },
    Break,
    Log {
        args: Vec<LogArg>,
    },
}

#[derive(Debug, Clone)]
pub enum LogArg {
    String(String),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    BoolLiteral(bool),
    NumberLiteral(usize),
    Variable(String),
    // Input(InputPath),
    
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    
    ArrayAccess {
        base: Box<Expression>,
        index: Box<Expression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    
    // Bitwise
    BitAnd, BitOr, BitXor,
    Shl, Shr,
    
    // Comparison
    Eq, Ne, Lt, Gt, Le, Ge,
    
    // Logical
    LogicalAnd, LogicalOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    LogicalNot,
    BitNot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputPath {
    // Direct paths
    CpuField(String),
    PpuField(String),
    ApuField(String),
    SysField(String),
    IoField(String),
    
    // Register access
    CpuReg(String),
    PpuReg(String),
    
    // DMA channel access
    DmaField(usize, String),
    DmaReg(usize, String),
    
    // Memory access (with runtime expression)
    CpuMem(Box<Expression>),
    Wram(Box<Expression>),
    Vram(Box<Expression>),
    Aram(Box<Expression>),
    Oam(Box<Expression>),
    Cgram(Box<Expression>),
    Mmio(Box<Expression>),
}