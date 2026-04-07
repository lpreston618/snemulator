use crate::ast::*;
use crate::bytecode::*;
use crate::input_id::*;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Undefined variable '{0}' at line {1}")]
    UndefinedVariable(String, usize),
    
    #[error("Type mismatch at line {0}: expected {1:?}, got {2:?}")]
    TypeMismatch(usize, Type, Type),
    
    #[error("Unknown input path at line {0}: {1}")]
    UnknownInput(usize, String),
    
    #[error("Invalid DMA channel {0} at line {1} (must be 0-7)")]
    InvalidDmaChannel(usize, usize),
    
    #[error("Assignment to non-variable at line {0}")]
    InvalidAssignment(usize),
    
    #[error("Input '{0}' used in LOGIC but not declared in INPUTS (line {1})")]
    UndeclaredInput(String, usize),
}

pub struct Compiler {
    variables: HashMap<String, (VarId, Type)>,
    next_var_id: VarId,
    next_condition_id: ConditionId,
    // declared_inputs: HashSet<InputId>,
    input_dependencies: HashMap<InputId, Vec<ConditionId>>,
    conditions: Vec<Condition>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            next_var_id: 0,
            next_condition_id: 0,
            // declared_inputs: HashSet::new(),
            input_dependencies: HashMap::new(),
            conditions: Vec::new(),
        }
    }
    
    pub fn compile(mut self, ast: Script) -> Result<CompiledScript, CompileError> {
        // Register declared inputs
        // for input_path in &ast.inputs {
        //     let input_ids = self.resolve_input_path_static(input_path, 0)?;
        //     for id in input_ids {
        //         self.declared_inputs.insert(id);
        //     }
        // }
        
        // Register variables
        let var_info: Vec<VarInfo> = ast.variables.iter().map(|v| {
            let id = self.next_var_id;
            self.next_var_id += 1;
            self.variables.insert(v.name.clone(), (id, v.ty));
            VarInfo {
                id,
                name: v.name.clone(),
                ty: v.ty,
            }
        }).collect();
        
        // Compile INIT block
        let init_bytecode = self.compile_statements(&ast.init, None)?;
        
        // Compile LOGIC block (each top-level if becomes a condition)
        for stmt in &ast.logic {
            self.compile_top_level_statement(stmt)?;
        }
        
        // Calculate source hash (simple for now)
        let source_hash = 0; // TODO: implement proper hashing
        
        Ok(CompiledScript {
            version: 1,
            source_hash,
            variables: var_info,
            input_dependencies: self.input_dependencies,
            init_bytecode,
            conditions: self.conditions,
        })
    }
    
    fn compile_top_level_statement(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match &stmt.kind {
            StatementKind::If { condition, then_block, else_block } => {
                let condition_id = self.next_condition_id;
                self.next_condition_id += 1;
                
                // Collect dependencies from condition expression
                let mut dependencies = HashSet::new();
                self.collect_dependencies(condition, &mut dependencies)?;
                
                // Also collect from all statements in blocks
                for s in then_block {
                    self.collect_statement_dependencies(s, &mut dependencies)?;
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.collect_statement_dependencies(s, &mut dependencies)?;
                    }
                }
                
                // Register dependencies
                for dep in &dependencies {
                    self.input_dependencies
                        .entry(*dep)
                        .or_insert_with(Vec::new)
                        .push(condition_id);
                }
                
                // Compile bytecode
                let mut bytecode = Vec::new();
                let mut actions = Vec::new();
                
                self.compile_if_statement(
                    condition,
                    then_block,
                    else_block.as_deref(),
                    &mut bytecode,
                    &mut actions,
                    stmt.line
                )?;
                
                self.conditions.push(Condition {
                    id: condition_id,
                    source_line: stmt.line as u32,
                    dependencies: dependencies.into_iter().collect(),
                    bytecode,
                    actions,
                });
                
                Ok(())
            }
            _ => {
                // Non-if statements at top level in LOGIC are wrapped in a simple condition
                let condition_id = self.next_condition_id;
                self.next_condition_id += 1;
                
                let mut dependencies = HashSet::new();
                self.collect_statement_dependencies(stmt, &mut dependencies)?;
                
                for dep in &dependencies {
                    self.input_dependencies
                        .entry(*dep)
                        .or_insert_with(Vec::new)
                        .push(condition_id);
                }
                
                let mut bytecode = Vec::new();
                let mut actions = Vec::new();
                
                self.compile_statement(stmt, &mut bytecode, &mut actions)?;
                
                self.conditions.push(Condition {
                    id: condition_id,
                    source_line: stmt.line as u32,
                    dependencies: dependencies.into_iter().collect(),
                    bytecode,
                    actions,
                });
                
                Ok(())
            }
        }
    }
    
    fn compile_statements(
        &mut self,
        statements: &[Statement],
        mut actions: Option<&mut Vec<Action>>
    ) -> Result<Vec<Op>, CompileError> {
        let mut bytecode = Vec::new();
        
        for stmt in statements {
            match &mut actions {
                Some(acts) => self.compile_statement(stmt, &mut bytecode, acts)?,
                None => {
                    let mut dummy_actions = Vec::new();
                    self.compile_statement(stmt, &mut bytecode, &mut dummy_actions)?;
                }
            }
        }
        
        Ok(bytecode)
    }
    
    fn compile_statement(
        &mut self,
        stmt: &Statement,
        bytecode: &mut Vec<Op>,
        actions: &mut Vec<Action>
    ) -> Result<(), CompileError> {
        match &stmt.kind {
            StatementKind::Assignment { var_name, value } => {
                // let (var_id, var_type) = self.variables.get(var_name)
                //     .copied()
                //     .ok_or_else(|| CompileError::UndefinedVariable(var_name.clone(), stmt.line))?;
                
                // let expr_type = self.compile_expression(value, bytecode)?;
                
                // if expr_type != var_type {
                //     return Err(CompileError::TypeMismatch(stmt.line, var_type, expr_type));
                // }
                
                // bytecode.push(Op::StoreVar(var_id));
                // Ok(())
                let (var_id, var_type) = self.variables.get(var_name)
                    .copied()
                    .ok_or_else(|| CompileError::UndefinedVariable(var_name.clone(), stmt.line))?;
                
                // Compile expression with expected type context
                let expr_type = self.compile_expression(value, bytecode, Some(var_type))?;
                
                if expr_type != var_type {
                    return Err(CompileError::TypeMismatch(stmt.line, var_type, expr_type));
                }
                
                bytecode.push(Op::StoreVar(var_id));
                Ok(())
            }
            
            StatementKind::Break => {
                actions.push(Action {
                    kind: ActionKind::Break,
                    line: stmt.line as u32,
                    bytecode_offset: bytecode.len(),
                });
                Ok(())
            }
            
            StatementKind::Log { args } => {
                let mut log_parts = Vec::new();
                
                for arg in args {
                    match arg {
                        LogArg::String(s) => {
                            log_parts.push(LogPart::Literal(s.clone()));
                        }
                        LogArg::Expression(expr) => {
                            let mut expr_bytecode = Vec::new();
                            self.compile_expression(expr, &mut expr_bytecode, None)?;
                            log_parts.push(LogPart::Expression(expr_bytecode));
                        }
                    }
                }
                
                actions.push(Action {
                    kind: ActionKind::Log(LogInfo { parts: log_parts }),
                    line: stmt.line as u32,
                    bytecode_offset: bytecode.len(),
                });
                
                Ok(())
            }
            
            StatementKind::Always { block } => {
                for stmt in block {
                    self.compile_statement(stmt, bytecode, actions)?;
                }
                
                Ok(())
            }
            
            StatementKind::If { condition, then_block, else_block } => {
                self.compile_if_statement(
                    condition,
                    then_block,
                    else_block.as_deref(),
                    bytecode,
                    actions,
                    stmt.line
                )
            }
        }
    }
    
    fn compile_if_statement(
        &mut self,
        condition: &Expression,
        then_block: &[Statement],
        else_block: Option<&[Statement]>,
        bytecode: &mut Vec<Op>,
        actions: &mut Vec<Action>,
        line: usize
    ) -> Result<(), CompileError> {
        // Compile condition
        let cond_type = self.compile_expression(condition, bytecode, None)?;
        if cond_type != Type::Bool {
            return Err(CompileError::TypeMismatch(line, Type::Bool, cond_type));
        }
        
        // Reserve space for jump
        let jump_if_false_pos = bytecode.len();
        bytecode.push(Op::JumpIfFalse(0)); // Placeholder
        
        // Compile then block
        for stmt in then_block {
            self.compile_statement(stmt, bytecode, actions)?;
        }
        
        if let Some(else_stmts) = else_block {
            // Reserve space for jump over else block
            let jump_pos = bytecode.len();
            bytecode.push(Op::Jump(0)); // Placeholder
            
            // Patch jump_if_false to point here
            let else_start = bytecode.len();
            bytecode[jump_if_false_pos] = Op::JumpIfFalse(else_start as u32);
            
            // Compile else block
            for stmt in else_stmts {
                self.compile_statement(stmt, bytecode, actions)?;
            }
            
            // Patch jump to point past else block
            let end_pos = bytecode.len();
            bytecode[jump_pos] = Op::Jump(end_pos as u32);
        } else {
            // Patch jump_if_false to point past then block
            let end_pos = bytecode.len();
            bytecode[jump_if_false_pos] = Op::JumpIfFalse(end_pos as u32);
        }
        
        Ok(())
    }
    
    fn compile_expression(
        &mut self,
        expr: &Expression,
        bytecode: &mut Vec<Op>,
        type_hint: Option<Type>
    ) -> Result<Type, CompileError> {
        match &expr.kind {
            ExpressionKind::BoolLiteral(b) => {
                bytecode.push(Op::PushBool(*b));
                Ok(Type::Bool)
            }
            
            ExpressionKind::NumberLiteral(n) => {
                let ty = type_hint.unwrap_or(Type::Num);
                    
                match ty {
                    Type::Byte if *n <= 0xFF => {
                        bytecode.push(Op::PushByte(*n as u8));
                        Ok(Type::Byte)
                    }
                    Type::Word if *n <= 0xFFFF => {
                        bytecode.push(Op::PushWord(*n as u16));
                        Ok(Type::Word)
                    }
                    Type::Num => {
                        bytecode.push(Op::PushNum(*n));
                        Ok(Type::Num)
                    }
                    _ => {
                        bytecode.push(Op::PushNum(*n));
                        Ok(Type::Num)
                    }
                }
            }
            
            ExpressionKind::Variable(name) => {
                let (var_id, var_type) = self.variables.get(name)
                    .copied()
                    .ok_or_else(|| CompileError::UndefinedVariable(name.clone(), expr.line))?;
                bytecode.push(Op::LoadVar(var_id));
                Ok(var_type)
            }
            
            // ExpressionKind::Input(path) => {
            //     self.compile_input_access(path, bytecode, expr.line)
            // }
            
            ExpressionKind::BinaryOp { op, left, right } => {
                let left_type = self.compile_expression(left, bytecode, type_hint)?;
                let right_type = self.compile_expression(right, bytecode, type_hint)?;
                
                // Type checking for binary operators
                let result_type = match op {
                    BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                        if left_type != Type::Bool || right_type != Type::Bool {
                            return Err(CompileError::TypeMismatch(expr.line, Type::Bool, left_type));
                        }
                        Type::Bool
                    }
                    BinaryOperator::Eq | BinaryOperator::Ne |
                    BinaryOperator::Lt | BinaryOperator::Gt |
                    BinaryOperator::Le | BinaryOperator::Ge => {
                        // Comparison always returns bool
                        Type::Bool
                    }
                    _ => {
                        // Arithmetic/bitwise operations preserve numeric type
                        // Use the "larger" type
                        match (left_type, right_type) {
                            (Type::Num, _) | (_, Type::Num) => Type::Num,
                            (Type::Word, _) | (_, Type::Word) => Type::Word,
                            _ => Type::Byte,
                        }
                    }
                };
                
                let op_code = match op {
                    BinaryOperator::Add => Op::Add,
                    BinaryOperator::Sub => Op::Sub,
                    BinaryOperator::Mul => Op::Mul,
                    BinaryOperator::Div => Op::Div,
                    BinaryOperator::Mod => Op::Mod,
                    BinaryOperator::BitAnd => Op::BitAnd,
                    BinaryOperator::BitOr => Op::BitOr,
                    BinaryOperator::BitXor => Op::BitXor,
                    BinaryOperator::Shl => Op::Shl,
                    BinaryOperator::Shr => Op::Shr,
                    BinaryOperator::Eq => Op::Eq,
                    BinaryOperator::Ne => Op::Ne,
                    BinaryOperator::Lt => Op::Lt,
                    BinaryOperator::Gt => Op::Gt,
                    BinaryOperator::Le => Op::Le,
                    BinaryOperator::Ge => Op::Ge,
                    BinaryOperator::LogicalAnd => Op::LogicalAnd,
                    BinaryOperator::LogicalOr => Op::LogicalOr,
                };
                
                bytecode.push(op_code);
                Ok(result_type)
            }
            
            ExpressionKind::UnaryOp { op, operand } => {
                let operand_type = self.compile_expression(operand, bytecode, type_hint)?;
                
                match op {
                    UnaryOperator::LogicalNot => {
                        if operand_type != Type::Bool {
                            return Err(CompileError::TypeMismatch(expr.line, Type::Bool, operand_type));
                        }
                        bytecode.push(Op::LogicalNot);
                        Ok(Type::Bool)
                    }
                    UnaryOperator::BitNot => {
                        bytecode.push(Op::BitNot);
                        Ok(operand_type)
                    }
                }
            }
            
            ExpressionKind::ArrayAccess { base, index } => {
                // First check if base is an input path that supports indexing
                // if let ExpressionKind::Input(path) = &base.kind {
                //     return self.compile_indexed_input_access(path, index, bytecode, expr.line);
                // }
                
                Err(CompileError::UnknownInput(expr.line, "Invalid array access".to_string()))
            }
        }
    }
    
    fn compile_input_access(
        &mut self,
        path: &InputPath,
        bytecode: &mut Vec<Op>,
        line: usize
    ) -> Result<Type, CompileError> {
        let (input_id, ty) = self.resolve_input_path(path, line)?;
        
        // Check if input was declared
        // if !self.declared_inputs.contains(&input_id) {
        //     return Err(CompileError::UndeclaredInput(format!("{:?}", path), line));
        // }
        
        bytecode.push(Op::LoadInput(input_id));
        Ok(ty)
    }
    
    // fn compile_indexed_input_access(
    //     &mut self,
    //     path: &InputPath,
    //     index: &Expression,
    //     bytecode: &mut Vec<Op>,
    //     line: usize
    // ) -> Result<Type, CompileError> {
    //     // Compile index expression first
    //     self.compile_expression(index, bytecode)?;
        
    //     // Generate appropriate load instruction based on path
    //     match path {
    //         InputPath::CpuField(name) if name == "mem" => {
    //             bytecode.push(Op::LoadCpuMem);
    //             Ok(Type::Byte)
    //         }
    //         InputPath::Wram(_) => {
    //             bytecode.push(Op::LoadWram);
    //             Ok(Type::Byte)
    //         }
    //         InputPath::Vram(_) => {
    //             bytecode.push(Op::LoadVram);
    //             Ok(Type::Byte)
    //         }
    //         InputPath::Aram(_) => {
    //             bytecode.push(Op::LoadAram);
    //             Ok(Type::Byte)
    //         }
    //         InputPath::Oam(_) => {
    //             bytecode.push(Op::LoadOam);
    //             Ok(Type::Byte)
    //         }
    //         InputPath::Cgram(_) => {
    //             bytecode.push(Op::LoadCgram);
    //             Ok(Type::Byte)
    //         }
    //         InputPath::Mmio(_) => {
    //             bytecode.push(Op::LoadMmio);
    //             Ok(Type::Byte)
    //         }
    //         _ => Err(CompileError::UnknownInput(line, format!("{:?} does not support indexing", path)))
    //     }
    // }
    
    fn resolve_input_path(&self, path: &InputPath, line: usize) -> Result<(InputId, Type), CompileError> {
        // This is a simplified version - full implementation would handle all cases
        match path {
            InputPath::CpuField(name) => self.resolve_cpu_field(name, line),
            InputPath::PpuField(name) => self.resolve_ppu_field(name, line),
            InputPath::SysField(name) => self.resolve_sys_field(name, line),
            _ => Err(CompileError::UnknownInput(line, format!("{:?}", path)))
        }
    }
    
    // fn resolve_input_path_static(&self, path: &InputPath, line: usize) -> Result<Vec<InputId>, CompileError> {
    //     // For memory access paths, return the base InputId (not address-specific)
    //     match path {
    //         InputPath::CpuMem(_) => Ok(vec![InputId::CpuMem]),
    //         InputPath::Wram(_) => Ok(vec![InputId::Wram]),
    //         InputPath::Vram(_) => Ok(vec![InputId::Vram]),
    //         InputPath::Aram(_) => Ok(vec![InputId::Aram]),
    //         InputPath::Oam(_) => Ok(vec![InputId::Oam]),
    //         InputPath::Cgram(_) => Ok(vec![InputId::Cgram]),
    //         InputPath::Mmio(_) => Ok(vec![InputId::Mmio]),
    //         _ => {
    //             let (id, _) = self.resolve_input_path(path, line)?;
    //             Ok(vec![id])
    //         }
    //     }
    // }
    
    fn resolve_cpu_field(&self, name: &str, line: usize) -> Result<(InputId, Type), CompileError> {
        match name {
            "a" => Ok((InputId::CpuA, Type::Word)),
            "x" => Ok((InputId::CpuX, Type::Word)),
            "y" => Ok((InputId::CpuY, Type::Word)),
            "sp" => Ok((InputId::CpuSp, Type::Word)),
            "pc" => Ok((InputId::CpuPc, Type::Word)),
            "dp" => Ok((InputId::CpuDp, Type::Word)),
            "pb" => Ok((InputId::CpuPb, Type::Byte)),
            "db" => Ok((InputId::CpuDb, Type::Byte)),
            "p" => Ok((InputId::CpuP, Type::Byte)),
            "prg_byte_0" => Ok((InputId::CpuPrgByte0, Type::Byte)),
            "prg_byte_1" => Ok((InputId::CpuPrgByte1, Type::Byte)),
            "prg_byte_2" => Ok((InputId::CpuPrgByte2, Type::Byte)),
            "flagc" => Ok((InputId::CpuFlagC, Type::Bool)),
            "flagz" => Ok((InputId::CpuFlagZ, Type::Bool)),
            "flagi" => Ok((InputId::CpuFlagI, Type::Bool)),
            "flagd" => Ok((InputId::CpuFlagD, Type::Bool)),
            "flagx" => Ok((InputId::CpuFlagX, Type::Bool)),
            "flagm" => Ok((InputId::CpuFlagM, Type::Bool)),
            "flagv" => Ok((InputId::CpuFlagV, Type::Bool)),
            "flagn" => Ok((InputId::CpuFlagN, Type::Bool)),
            "nmi_pending" => Ok((InputId::CpuNmiPending, Type::Bool)),
            "full_pc" => Ok((InputId::CpuFullPc, Type::Num)),
            _ => Err(CompileError::UnknownInput(line, format!("cpu.{}", name)))
        }
    }
    
    fn resolve_ppu_field(&self, name: &str, line: usize) -> Result<(InputId, Type), CompileError> {
        match name {
            "dot" => Ok((InputId::PpuDot, Type::Num)),
            "scanline" => Ok((InputId::PpuScanline, Type::Num)),
            "screen_x" => Ok((InputId::PpuScreenX, Type::Num)),
            "screen_y" => Ok((InputId::PpuScreenY, Type::Num)),
            "vram_addr" => Ok((InputId::PpuVramAddr, Type::Word)),
            _ => Err(CompileError::UnknownInput(line, format!("ppu.{}", name)))
        }
    }
    
    fn resolve_sys_field(&self, name: &str, line: usize) -> Result<(InputId, Type), CompileError> {
        match name {
            "frame" => Ok((InputId::SysFrame, Type::Num)),
            _ => Err(CompileError::UnknownInput(line, format!("sys.{}", name)))
        }
    }
    
    fn collect_dependencies(
        &self,
        expr: &Expression,
        deps: &mut HashSet<InputId>
    ) -> Result<(), CompileError> {
        match &expr.kind {
            // ExpressionKind::Input(path) => {
            //     let ids = self.resolve_input_path_static(path, expr.line)?;
            //     for id in ids {
            //         deps.insert(id);
            //     }
            // }
            ExpressionKind::BinaryOp { left, right, .. } => {
                self.collect_dependencies(left, deps)?;
                self.collect_dependencies(right, deps)?;
            }
            ExpressionKind::UnaryOp { operand, .. } => {
                self.collect_dependencies(operand, deps)?;
            }
            ExpressionKind::ArrayAccess { base, index } => {
                self.collect_dependencies(base, deps)?;
                self.collect_dependencies(index, deps)?;
            }
            _ => {}
        }
        Ok(())
    }
    
    fn collect_statement_dependencies(
        &self,
        stmt: &Statement,
        deps: &mut HashSet<InputId>
    ) -> Result<(), CompileError> {
        match &stmt.kind {
            StatementKind::Assignment { value, .. } => {
                self.collect_dependencies(value, deps)?;
            }
            StatementKind::If { condition, then_block, else_block } => {
                self.collect_dependencies(condition, deps)?;
                for s in then_block {
                    self.collect_statement_dependencies(s, deps)?;
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.collect_statement_dependencies(s, deps)?;
                    }
                }
            }
            StatementKind::Log { args } => {
                for arg in args {
                    if let LogArg::Expression(expr) = arg {
                        self.collect_dependencies(expr, deps)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}