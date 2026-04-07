use serde::{Deserialize, Serialize};
use crate::input_id::InputId;

pub type VarId = u32;
pub type ConditionId = u32;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Type {
    Bool,
    Byte,
    Word,
    Num,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Value {
    Bool(bool),
    Byte(u8),
    Word(u16),
    Num(usize),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Op {
    // Literals
    PushBool(bool),
    PushByte(u8),
    PushWord(u16),
    PushNum(usize),
    
    // Variable/Input access
    LoadVar(VarId),
    StoreVar(VarId),
    LoadInput(InputId),
    
    // Memory access (address on stack -> value)
    LoadCpuMem,
    LoadWram,
    LoadVram,
    LoadAram,
    LoadOam,
    LoadCgram,
    LoadMmio,
    
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,
    
    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    
    // Logical
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    
    // Control flow
    JumpIfFalse(u32),
    Jump(u32),
    
    // Stack manipulation
    Dup,
    Pop,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogInfo {
    pub parts: Vec<LogPart>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LogPart {
    Literal(String),
    Expression(Vec<Op>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    pub kind: ActionKind,
    pub line: u32,
    pub bytecode_offset: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActionKind {
    Break,
    Log(LogInfo),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Condition {
    pub id: ConditionId,
    pub source_line: u32,
    pub dependencies: Vec<InputId>,
    pub bytecode: Vec<Op>,
    pub actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VarInfo {
    pub id: VarId,
    pub name: String,
    pub ty: Type,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompiledScript {
    pub version: u32,
    pub source_hash: u64,
    pub variables: Vec<VarInfo>,
    pub input_dependencies: std::collections::HashMap<InputId, Vec<ConditionId>>,
    pub init_bytecode: Vec<Op>,
    pub conditions: Vec<Condition>,
}