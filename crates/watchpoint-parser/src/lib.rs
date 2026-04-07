use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

mod ast;
mod bytecode;
mod compiler;
mod input_id;

pub use bytecode::{CompiledScript, Op, Type, Value, Action, ActionKind, LogInfo, LogPart, Condition, VarInfo};
pub use input_id::{InputId, CpuRegister, PpuRegister, DmaRegister};
use ast::*;
use compiler::{Compiler, CompileError};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct WatchpointParser;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("Compile error: {0}")]
    CompileError(#[from] CompileError),
    
    #[error("Internal parser error: {0}")]
    InternalError(String),
}

pub fn parse_watchpoint_script(source: &str) -> Result<CompiledScript, ParseError> {    
    // Parse with pest
    let pairs = WatchpointParser::parse(Rule::script, source)
        .map_err(|e| {
            let (line, column) = match e.line_col {
                pest::error::LineColLocation::Pos((line, col)) => (line, col),
                pest::error::LineColLocation::Span((line, col), _) => (line, col),
            };
            ParseError::SyntaxError {
                line,
                column,
                message: e.variant.message().to_string(),
            }
        })?;
    
    // Build AST
    let ast = build_ast(pairs)?;
    
    // Compile to bytecode
    let compiler = Compiler::new();
    let compiled = compiler.compile(ast)?;
    
    Ok(compiled)
}

pub fn serialize_script(script: &CompiledScript) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(bincode::serialize(script)?)
}

pub fn deserialize_script(data: &[u8]) -> Result<CompiledScript, Box<dyn std::error::Error>> {
    Ok(bincode::deserialize(data)?)
}

// fn print_pair(pair: pest::iterators::Pair<Rule>, indent: usize) {
//     let indent_str = "  ".repeat(indent);
//     println!("{}{:?}: {:?}", indent_str, pair.as_rule(), pair.as_str());
//     for inner in pair.into_inner() {
//         print_pair(inner, indent + 1);
//     }
// }

fn build_ast(pairs: pest::iterators::Pairs<Rule>) -> Result<Script, ParseError> {
    // let mut inputs = None;
    let mut variables = None;
    let mut init = None;
    let mut logic = None;
    
    // for pair in pairs.clone() {
    //     print_pair(pair, 0);
    // }
    
    for pair in pairs {
        println!("Top level rule: {:?}", pair.as_rule());  // DEBUG
        
        if pair.as_rule() != Rule::script {
            continue;
        }
        
        for inner in pair.into_inner() {            
            println!("  Script inner rule: {:?}", inner.as_rule());  // DEBUG
            
            if inner.as_rule() != Rule::section {
                continue;
            }
            
            for section_inner in inner.into_inner() {
                println!("    Section inner: {:?} = {:?}", 
                    section_inner.as_rule(), 
                    section_inner.as_str());  // DEBUG
                
                match section_inner.as_rule() {
                    // Rule::inputs => {
                    //     if inputs.is_some() {
                    //         return Err(ParseError::SyntaxError {
                    //             line: section_inner.as_span().start_pos().line_col().0,
                    //             column: section_inner.as_span().start_pos().line_col().1,
                    //             message: "INPUTS section defined multiple times".to_string()
                    //         });
                    //     }
                    //     inputs = Some(parse_inputs(section_inner)?);
                    // }
                    Rule::variables => {
                        if variables.is_some() {
                            return Err(ParseError::SyntaxError {
                                line: section_inner.as_span().start_pos().line_col().0,
                                column: section_inner.as_span().start_pos().line_col().1,
                                message: "VARIABLES section defined multiple times".to_string()
                            });
                        }
                        variables = Some(parse_variables(section_inner)?);
                    }
                    Rule::init => {
                        if init.is_some() {
                            return Err(ParseError::SyntaxError {
                                line: section_inner.as_span().start_pos().line_col().0,
                                column: section_inner.as_span().start_pos().line_col().1,
                                message: "INIT section defined multiple times".to_string()
                            });
                        }
                        init = Some(parse_statements(section_inner.into_inner())?);
                    }
                    Rule::logic => {                        
                        if logic.is_some() {
                            return Err(ParseError::SyntaxError {
                                line: section_inner.as_span().start_pos().line_col().0,
                                column: section_inner.as_span().start_pos().line_col().1,
                                message: "LOGIC section defined multiple times".to_string()
                            });
                        }

                        let mut logic_statements = Vec::new();
                            
                        for inner in section_inner.into_inner() {
                            if inner.as_rule() != Rule::conditional {
                                continue;
                            }
                            
                            // Each conditional contains either an if_statement or always_statement
                            for cond_inner in inner.into_inner() {
                                match cond_inner.as_rule() {
                                    Rule::if_statement => {
                                        let line = cond_inner.as_span().start_pos().line_col().0;
                                        
                                        logic_statements.push(parse_if_statement(cond_inner, line)?);
                                    }
                                    Rule::always_statement => {
                                        let line = cond_inner.as_span().start_pos().line_col().0;
                                        
                                        logic_statements.push(parse_always_statement(cond_inner, line)?);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        
                        logic = Some(logic_statements);
                    }
                    _ => {}
                }
            }
        }
    }
    
    Ok(Script {
        // inputs: inputs.unwrap_or_default(),
        variables: variables.unwrap_or_default(),
        init: init.unwrap_or_default(),
        logic: logic.unwrap_or_default(),
    })
}

// fn parse_inputs(pair: pest::iterators::Pair<Rule>) -> Result<Vec<InputPath>, ParseError> {
//     let mut inputs = Vec::new();
    
//     for inner in pair.into_inner() {
//         if inner.as_rule() == Rule::input_list {
//             for input_decl in inner.into_inner() {
//                 if input_decl.as_rule() == Rule::input_decl {
//                     for path_pair in input_decl.into_inner() {
//                         if path_pair.as_rule() == Rule::input_path {
//                             inputs.push(parse_input_path(path_pair)?);
//                         }
//                     }
//                 }
//             }
//         }
//     }
    
//     Ok(inputs)
// }

fn parse_variables(pair: pest::iterators::Pair<Rule>) -> Result<Vec<VariableDecl>, ParseError> {
    let mut variables = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::variable_list {
            for var_decl in inner.into_inner() {
                if var_decl.as_rule() == Rule::variable_decl {
                    let line = var_decl.as_span().start_pos().line_col().0;
                    let mut inner_iter = var_decl.into_inner();
                    
                    let type_pair = inner_iter.next().ok_or_else(|| {
                        ParseError::InternalError("Expected type in variable declaration".to_string())
                    })?;
                    let ty = parse_type(type_pair)?;
                    
                    let name_pair = inner_iter.next().ok_or_else(|| {
                        ParseError::InternalError("Expected name in variable declaration".to_string())
                    })?;
                    let name = name_pair.as_str().to_string();
                    
                    variables.push(VariableDecl { ty, name, line });
                }
            }
        }
    }
    
    Ok(variables)
}

fn parse_type(pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
    match pair.as_str() {
        "Bool" => Ok(Type::Bool),
        "Byte" => Ok(Type::Byte),
        "Word" => Ok(Type::Word),
        "Num" => Ok(Type::Num),
        _ => Err(ParseError::InternalError(format!("Unknown type: {}", pair.as_str()))),
    }
}

fn parse_statements(pairs: pest::iterators::Pairs<Rule>) -> Result<Vec<Statement>, ParseError> {
    let mut statements = Vec::new();
    
    for pair in pairs {
        if pair.as_rule() == Rule::conditional {
            statements.push(parse_statement(pair)?);
        }
    }
    
    Ok(statements)
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<Statement, ParseError> {
    let line = pair.as_span().start_pos().line_col().0;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::if_statement => {
                return parse_if_statement(inner, line);
            }
            Rule::assignment => {
                return parse_assignment(inner, line);
            }
            Rule::break_statement => {
                return Ok(Statement {
                    kind: StatementKind::Break,
                    line,
                });
            }
            Rule::log_statement => {
                return parse_log_statement(inner, line);
            }
            _ => {}
        }
    }
    
    Err(ParseError::InternalError("Empty statement".to_string()))
}

fn parse_if_statement(pair: pest::iterators::Pair<Rule>, line: usize) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    
    let condition_pair = inner.next().ok_or_else(|| {
        ParseError::InternalError("Expected condition in if statement".to_string())
    })?;
    let condition = parse_expression(condition_pair)?;
    
    let mut then_block = Vec::new();
    let mut else_block = None;
    
    for part in inner {
        match part.as_rule() {
            Rule::statement => {
                then_block.push(parse_statement(part)?);
            }
            Rule::else_clause => {
                let mut else_statements = Vec::new();
                for stmt in part.into_inner() {
                    if stmt.as_rule() == Rule::statement {
                        else_statements.push(parse_statement(stmt)?);
                    }
                }
                else_block = Some(else_statements);
            }
            _ => {}
        }
    }
    
    Ok(Statement {
        kind: StatementKind::If {
            condition,
            then_block,
            else_block,
        },
        line,
    })
}

fn parse_always_statement(pair: pest::iterators::Pair<Rule>, line: usize) -> Result<Statement, ParseError> {
    let inner = pair.into_inner();
    
    let mut block = Vec::new();
    
    for part in inner {
        match part.as_rule() {
            Rule::statement => {
                block.push(parse_statement(part)?);
            }
            _ => {}
        }
    }
    
    Ok(Statement {
        kind: StatementKind::Always {
            block,
        },
        line,
    })
}

fn parse_assignment(pair: pest::iterators::Pair<Rule>, line: usize) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    
    let var_name = inner.next()
        .ok_or_else(|| ParseError::InternalError("Expected variable name".to_string()))?
        .as_str()
        .to_string();
    
    let value_pair = inner.next()
        .ok_or_else(|| ParseError::InternalError("Expected expression".to_string()))?;
    let value = parse_expression(value_pair)?;
    
    Ok(Statement {
        kind: StatementKind::Assignment { var_name, value },
        line,
    })
}

fn parse_log_statement(pair: pest::iterators::Pair<Rule>, line: usize) -> Result<Statement, ParseError> {
    let mut args = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::log_args {
            for arg in inner.into_inner() {
                if arg.as_rule() == Rule::log_arg {
                    args.push(parse_log_arg(arg)?);
                }
            }
        }
    }
    
    Ok(Statement {
        kind: StatementKind::Log { args },
        line,
    })
}

fn parse_log_arg(pair: pest::iterators::Pair<Rule>) -> Result<LogArg, ParseError> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string_literal => {
                let s = inner.as_str();
                // Remove quotes and handle escape sequences
                let unquoted = &s[1..s.len()-1];
                let unescaped = unquoted
                    .replace("\\n", "\n")
                    .replace("\\r", "\r")
                    .replace("\\t", "\t")
                    .replace("\\\\", "\\")
                    .replace("\\\"", "\"");
                return Ok(LogArg::String(unescaped));
            }
            Rule::expression => {
                return Ok(LogArg::Expression(parse_expression(inner)?));
            }
            _ => {}
        }
    }
    
    Err(ParseError::InternalError("Empty log arg".to_string()))
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
    let line = pair.as_span().start_pos().line_col().0;
    
    match pair.as_rule() {
        Rule::expression | Rule::logical_or | Rule::logical_and | 
        Rule::bitwise_or | Rule::bitwise_xor | Rule::bitwise_and |
        Rule::equality | Rule::comparison | Rule::shift |
        Rule::additive | Rule::multiplicative => {
            parse_binary_expression(pair, line)
        }
        Rule::unary => {
            parse_unary_expression(pair, line)
        }
        Rule::postfix => {
            parse_postfix_expression(pair, line)
        }
        Rule::primary => {
            parse_primary_expression(pair, line)
        }
        _ => Err(ParseError::InternalError(format!("Unexpected expression rule: {:?}", pair.as_rule())))
    }
}

fn parse_binary_expression(pair: pest::iterators::Pair<Rule>, line: usize) -> Result<Expression, ParseError> {
    let inner: Vec<_> = pair.into_inner().collect();
    
    // If we only have one element, just parse it directly
    if inner.len() == 1 {
        return parse_expression(inner.into_iter().next().unwrap());
    }
    
    // Otherwise we have: expr (op expr)+
    let mut iter = inner.into_iter();
    let first = iter.next().ok_or_else(|| {
        ParseError::InternalError("Expected first operand".to_string())
    })?;
    
    let mut left = parse_expression(first)?;
    
    while let Some(op_pair) = iter.next() {
        // Check if this is actually an operator by looking at the string
        let op_str = op_pair.as_str();
        
        // Only parse as binary operator if it's actually an operator string
        if let Ok(op) = parse_binary_operator(op_str) {
            let right_pair = iter.next().ok_or_else(|| {
                ParseError::InternalError(format!("Expected right operand after operator '{}'", op_str))
            })?;
            let right = parse_expression(right_pair)?;
            
            left = Expression {
                kind: ExpressionKind::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                line,
            };
        } else {
            // Not an operator, something's wrong with our grammar understanding
            return Err(ParseError::InternalError(
                format!("Expected operator but got: '{}'", op_str)
            ));
        }
    }
    
    Ok(left)
}

fn parse_binary_operator(s: &str) -> Result<BinaryOperator, ParseError> {
    match s {
        "+" => Ok(BinaryOperator::Add),
        "-" => Ok(BinaryOperator::Sub),
        "*" => Ok(BinaryOperator::Mul),
        "/" => Ok(BinaryOperator::Div),
        "%" => Ok(BinaryOperator::Mod),
        "&" => Ok(BinaryOperator::BitAnd),
        "|" => Ok(BinaryOperator::BitOr),
        "^" => Ok(BinaryOperator::BitXor),
        "<<" => Ok(BinaryOperator::Shl),
        ">>" => Ok(BinaryOperator::Shr),
        "==" => Ok(BinaryOperator::Eq),
        "!=" => Ok(BinaryOperator::Ne),
        "<" => Ok(BinaryOperator::Lt),
        ">" => Ok(BinaryOperator::Gt),
        "<=" => Ok(BinaryOperator::Le),
        ">=" => Ok(BinaryOperator::Ge),
        "&&" => Ok(BinaryOperator::LogicalAnd),
        "||" => Ok(BinaryOperator::LogicalOr),
        _ => Err(ParseError::InternalError(format!("Unknown operator: {}", s))),
    }
}

fn parse_unary_expression(pair: pest::iterators::Pair<Rule>, line: usize) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner.next().ok_or_else(|| {
        ParseError::InternalError("Expected operand".to_string())
    })?;
    
    if first.as_str() == "!" || first.as_str() == "~" {
        let op = if first.as_str() == "!" {
            UnaryOperator::LogicalNot
        } else {
            UnaryOperator::BitNot
        };
        
        let operand_pair = inner.next().ok_or_else(|| {
            ParseError::InternalError("Expected operand after unary operator".to_string())
        })?;
        let operand = parse_expression(operand_pair)?;
        
        Ok(Expression {
            kind: ExpressionKind::UnaryOp {
                op,
                operand: Box::new(operand),
            },
            line,
        })
    } else {
        parse_expression(first)
    }
}

fn parse_postfix_expression(pair: pest::iterators::Pair<Rule>, line: usize) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let primary = inner.next().ok_or_else(|| {
        ParseError::InternalError("Expected primary expression".to_string())
    })?;
    
    let mut expr = parse_expression(primary)?;
    
    // Handle array access
    for index_pair in inner {
        if index_pair.as_rule() == Rule::expression {
            let index = parse_expression(index_pair)?;
            expr = Expression {
                kind: ExpressionKind::ArrayAccess {
                    base: Box::new(expr),
                    index: Box::new(index),
                },
                line,
            };
        }
    }
    
    Ok(expr)
}

fn parse_primary_expression(pair: pest::iterators::Pair<Rule>, line: usize) -> Result<Expression, ParseError> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::bool_literal => {
                let b = inner.as_str() == "true";
                return Ok(Expression {
                    kind: ExpressionKind::BoolLiteral(b),
                    line,
                });
            }
            Rule::number_literal => {
                let n = parse_number(inner.as_str())?;
                return Ok(Expression {
                    kind: ExpressionKind::NumberLiteral(n),
                    line,
                });
            }
            // Rule::input_path => {
            //     let path = parse_input_path(inner)?;
            //     return Ok(Expression {
            //         kind: ExpressionKind::Input(path),
            //         line,
            //     });
            // }
            Rule::ident => {
                return Ok(Expression {
                    kind: ExpressionKind::Variable(inner.as_str().to_string()),
                    line,
                });
            }
            Rule::expression => {
                return parse_expression(inner);
            }
            _ => {}
        }
    }
    
    Err(ParseError::InternalError("Empty primary expression".to_string()))
}

fn parse_number(s: &str) -> Result<usize, ParseError> {
    if s.starts_with("0x") || s.starts_with("$") {
        let hex_str = if s.starts_with("0x") {
            &s[2..]
        } else {
            &s[1..]
        };
        usize::from_str_radix(hex_str, 16)
            .map_err(|e| ParseError::InternalError(format!("Invalid hex number: {}", e)))
    } else {
        s.parse::<usize>()
            .map_err(|e| ParseError::InternalError(format!("Invalid decimal number: {}", e)))
    }
}

fn parse_input_path(pair: pest::iterators::Pair<Rule>) -> Result<InputPath, ParseError> {
    let s = pair.as_str();
    let mut parts: Vec<&str> = Vec::new();
    let mut current_part = String::new();
    let mut in_brackets = false;
    let mut bracket_expr = String::new();
    
    for ch in s.chars() {
        if ch == '[' {
            in_brackets = true;
            if !current_part.is_empty() {
                parts.push(Box::leak(current_part.into_boxed_str()));
                current_part = String::new();
            }
        } else if ch == ']' {
            in_brackets = false;
            // We'll handle bracket expressions separately
            parts.push(Box::leak(format!("[{}]", bracket_expr).into_boxed_str()));
            bracket_expr = String::new();
        } else if ch == '.' && !in_brackets {
            if !current_part.is_empty() {
                parts.push(Box::leak(current_part.into_boxed_str()));
                current_part = String::new();
            }
        } else {
            if in_brackets {
                bracket_expr.push(ch);
            } else {
                current_part.push(ch);
            }
        }
    }
    
    if !current_part.is_empty() {
        parts.push(Box::leak(current_part.into_boxed_str()));
    }
    
    // Parse based on the structure
    if parts.is_empty() {
        return Err(ParseError::InternalError("Empty input path".to_string()));
    }
    
    match parts[0] {
        "cpu" => {
            if parts.len() < 2 {
                return Err(ParseError::InternalError("Incomplete cpu path".to_string()));
            }
            
            if parts[1] == "mem" && parts.len() == 3 && parts[2].starts_with('[') {
                // cpu.mem[expr]
                let expr_str = &parts[2][1..parts[2].len()-1];
                let expr = parse_expression_from_str(expr_str, pair.as_span().start_pos().line_col().0)?;
                return Ok(InputPath::CpuMem(Box::new(expr)));
            } else if parts[1] == "regs" && parts.len() == 3 {
                return Ok(InputPath::CpuReg(parts[2].to_string()));
            } else if parts.len() == 2 {
                return Ok(InputPath::CpuField(parts[1].to_string()));
            }
        }
        "ppu" => {
            if parts.len() < 2 {
                return Err(ParseError::InternalError("Incomplete ppu path".to_string()));
            }
            
            if parts[1] == "regs" && parts.len() == 3 {
                return Ok(InputPath::PpuReg(parts[2].to_string()));
            } else if parts.len() == 2 {
                return Ok(InputPath::PpuField(parts[1].to_string()));
            }
        }
        "apu" => {
            if parts.len() == 2 {
                return Ok(InputPath::ApuField(parts[1].to_string()));
            }
        }
        "sys" => {
            if parts.len() == 2 {
                return Ok(InputPath::SysField(parts[1].to_string()));
            }
        }
        "io" => {
            if parts.len() == 2 {
                return Ok(InputPath::IoField(parts[1].to_string()));
            }
        }
        "dma" => {
            // dma[n].field or dma[n].regs.NAME
            if parts.len() >= 3 && parts[1].starts_with('[') {
                let channel_str = &parts[1][1..parts[1].len()-1];
                let channel = channel_str.parse::<usize>()
                    .map_err(|_| ParseError::InternalError("Invalid DMA channel".to_string()))?;
                
                if parts[2] == "regs" && parts.len() == 4 {
                    return Ok(InputPath::DmaReg(channel, parts[3].to_string()));
                } else if parts.len() == 3 {
                    return Ok(InputPath::DmaField(channel, parts[2].to_string()));
                }
            }
        }
        "wram" | "vram" | "aram" | "oam" | "cgram" | "mmio" => {
            if parts.len() == 2 && parts[1].starts_with('[') {
                let expr_str = &parts[1][1..parts[1].len()-1];
                let expr = parse_expression_from_str(expr_str, pair.as_span().start_pos().line_col().0)?;
                
                return Ok(match parts[0] {
                    "wram" => InputPath::Wram(Box::new(expr)),
                    "vram" => InputPath::Vram(Box::new(expr)),
                    "aram" => InputPath::Aram(Box::new(expr)),
                    "oam" => InputPath::Oam(Box::new(expr)),
                    "cgram" => InputPath::Cgram(Box::new(expr)),
                    "mmio" => InputPath::Mmio(Box::new(expr)),
                    _ => unreachable!(),
                });
            }
        }
        _ => {}
    }
    
    Err(ParseError::InternalError(format!("Unknown input path: {}", s)))
}

fn parse_expression_from_str(s: &str, line: usize) -> Result<Expression, ParseError> {
    let pairs = WatchpointParser::parse(Rule::expression, s)
        .map_err(|e| ParseError::InternalError(format!("Failed to parse expression: {}", e)))?;
    
    for pair in pairs {
        return parse_expression(pair);
    }
    
    Err(ParseError::InternalError("Empty expression".to_string()))
}