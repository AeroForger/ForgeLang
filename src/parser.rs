use pest::iterators::Pair;
use pest_derive::Parser;
use pest::Parser;

use crate::ast::*;
use crate::errors::{ForgeError, ForgeResult};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ForgeParser;

pub fn parse_program(source: &str) -> ForgeResult<Program> {
    let pair = ForgeParser::parse(Rule::program, source)
        .map_err(|e| ForgeError::parse(format!("{}", e)))?
        .next()
        .ok_or_else(|| ForgeError::parse("empty parse tree"))?;
    build_program(pair)
}

fn build_program(pair: Pair<Rule>) -> ForgeResult<Program> {
    debug_assert_eq!(pair.as_rule(), Rule::program);
    let mut statements = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::EOI { continue; }
        statements.push(build_statement(inner)?);
    }
    Ok(Program { statements })
}

fn build_statement(pair: Pair<Rule>) -> ForgeResult<Statement> {
    let inner = pair.into_inner().next()
        .ok_or_else(|| ForgeError::parse("empty statement"))?;
    Ok(match inner.as_rule() {
        Rule::var_decl       => Statement::VarDecl(build_var_decl(inner)?),
        Rule::print_stmt     => Statement::Print(build_print(inner)?),
        Rule::assignment     => Statement::Assignment(build_assignment(inner)?),
        Rule::expr_stmt      => {
            let e = build_expr(inner.into_inner().next().unwrap())?;
            Statement::ExprStmt(e)
        }
        Rule::return_stmt    => {
            let expr = inner.into_inner().next().map(build_expr).transpose()?;
            Statement::Return(expr)
        }
        Rule::input_stmt     => Statement::Input(build_input(inner)?),
        Rule::function_decl  => Statement::FunctionDecl(build_function_decl(inner)?),
        Rule::if_stmt        => Statement::If(build_if_stmt(inner)?),
        Rule::while_stmt     => Statement::While(build_while_stmt(inner)?),
        Rule::data_decl      => Statement::DataDecl(build_data_decl(inner)?),
        Rule::object_decl    => Statement::ObjectDecl(build_object_decl(inner)?),
        Rule::use_stmt       => Statement::Use(UseNode { path: vec![], item: None }),
        r => return Err(ForgeError::parse(format!("unexpected rule: {:?}", r))),
    })
}

fn nyi(name: &str) -> ForgeError {
    ForgeError::parse(format!("not yet implemented: {}", name))
}

fn build_function_decl(pair: Pair<Rule>) -> ForgeResult<FunctionDecl> {
    let mut it = pair.into_inner();
    let mut modifier = None;
    let mut next = it.next().unwrap();
    
    if next.as_rule() == Rule::modifier {
        modifier = Some(parse_modifier(next.as_str()));
        next = it.next().unwrap();
    }
    
    if next.as_rule() != Rule::ret_kind {
        return Err(ForgeError::parse(format!("expected ret_kind, found {:?}", next.as_rule())));
    }
    
    let ret_kind = match next.as_str() {
        "Int" => RetKind::Int,
        "Float" => RetKind::Float,
        "Generic" => RetKind::Generic,
        "Weld" => RetKind::Weld,
        "Ore" => RetKind::Ore,
        "Materials" => RetKind::Materials,
        "function" => RetKind::Function,
        "Nunction" => RetKind::Nunction,
        _ => unreachable!(),
    };
    
    let name = it.next().unwrap().as_str().to_string();
    
    let params = if let Some(p) = it.next() {
        if p.as_rule() == Rule::param_list {
            p.into_inner().map(build_param).collect::<ForgeResult<Vec<_>>>()?
        } else {
            let body = build_block(p)?;
            return Ok(FunctionDecl { modifier, ret_kind, name, params: Vec::new(), body });
        }
    } else {
        Vec::new()
    };
    
    let block_pair = it.next().unwrap();
    let body = build_block(block_pair)?;
    
    Ok(FunctionDecl { modifier, ret_kind, name, params, body })
}

fn build_param(pair: Pair<Rule>) -> ForgeResult<Param> {
    let mut it = pair.into_inner();
    let type_decl = build_type_decl(it.next().unwrap())?;
    let name = it.next().unwrap().as_str().to_string();
    Ok(Param { type_decl, name })
}

fn build_block(pair: Pair<Rule>) -> ForgeResult<Vec<Statement>> {
    pair.into_inner().map(build_statement).collect()
}

fn build_var_decl(pair: Pair<Rule>) -> ForgeResult<VarDecl> {
    let mut it = pair.into_inner();
    let mut modifier = None;
    let mut next = it.next().unwrap();
    if next.as_rule() == Rule::modifier {
        modifier = Some(parse_modifier(next.as_str()));
        next = it.next().unwrap();
    }
    if next.as_rule() != Rule::type_decl {
        return Err(ForgeError::parse(format!("expected type_decl, found {:?}", next.as_rule())));
    }
    let type_decl = build_type_decl(next)?;
    let name = it.next().unwrap().as_str().to_string();
    let initializer = it.next().map(build_expr).transpose()?;
    Ok(VarDecl { modifier, type_decl, name, initializer })
}

fn build_data_decl(pair: Pair<Rule>) -> ForgeResult<DataDecl> {
    let mut it = pair.into_inner();
    let mut modifier = None;
    let mut next = it.next().unwrap();
    if next.as_rule() == Rule::modifier {
        modifier = Some(parse_modifier(next.as_str()));
        next = it.next().unwrap();
    }
    // next is kw_data
    let name = it.next().unwrap().as_str().to_string();
    let mut members = Vec::new();
    for member_pair in it {
        members.push(build_var_decl(member_pair)?);
    }
    Ok(DataDecl { modifier, name, members })
}

fn build_object_decl(pair: Pair<Rule>) -> ForgeResult<ObjectDecl> {
    let mut it = pair.into_inner();
    let type_name = it.next().unwrap().as_str().to_string();
    let name = it.next().unwrap().as_str().to_string();
    let mut inits = Vec::new();
    
    // The rest are member_init blocks
    for init_pair in it {
        let mut init_it = init_pair.into_inner();
        let mut path = vec![init_it.next().unwrap().as_str().to_string()];
        let mut next = init_it.next().unwrap();
        while next.as_rule() == Rule::ident {
            path.push(next.as_str().to_string());
            next = init_it.next().unwrap();
        }
        let value = build_expr(next)?;
        inits.push((path, value));
    }
    
    Ok(ObjectDecl { type_name, name, inits })
}

fn build_if_stmt(pair: Pair<Rule>) -> ForgeResult<IfNode> {
    let mut it = pair.into_inner();
    let mut branches = Vec::new();
    let mut else_body = None;
    
    // Skip kw_if
    it.next().unwrap();
    
    let cond = build_expr(it.next().unwrap())?;
    let block = build_block(it.next().unwrap())?;
    branches.push((cond, block));
    
    while let Some(_kw_else) = it.next() {
        let next = it.next().unwrap();
        if next.as_rule() == Rule::kw_if {
            let cond = build_expr(it.next().unwrap())?;
            let block = build_block(it.next().unwrap())?;
            branches.push((cond, block));
        } else {
            else_body = Some(build_block(next)?);
            break;
        }
    }
    
    Ok(IfNode { branches, else_body })
}

fn build_while_stmt(pair: Pair<Rule>) -> ForgeResult<WhileNode> {
    let mut it = pair.into_inner();
    it.next().unwrap();
    let condition = build_expr(it.next().unwrap())?;
    let body = build_block(it.next().unwrap())?;
    Ok(WhileNode { condition, body })
}

fn build_type_decl(pair: Pair<Rule>) -> ForgeResult<TypeDecl> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::number_type => {
            let st = inner.clone().into_inner()
                .find(|p| p.as_rule() == Rule::subtype)
                .unwrap()
                .as_str();
            Ok(TypeDecl::Number(parse_subtype(st)))
        }
        Rule::weld_type => Ok(TypeDecl::Weld),
        Rule::ore_type => {
            let size = inner.clone().into_inner()
                .find(|p| p.as_rule() == Rule::integer)
                .map(|p| p.as_str().parse::<i64>().unwrap());
            Ok(TypeDecl::Ore(size))
        }
        Rule::materials_type => {
            let mut children = inner.into_inner();
            let st = children.find(|p| p.as_rule() == Rule::subtype).unwrap().as_str();
            let has_new = children.any(|p| p.as_rule() == Rule::kw_new);
            Ok(TypeDecl::Materials(parse_subtype(st), has_new))
        }
        r => Err(ForgeError::parse(format!("unexpected type_decl rule: {:?}", r))),
    }
}

fn build_print(pair: Pair<Rule>) -> ForgeResult<PrintNode> {
    let expr = pair.into_inner()
        .find(|p| p.as_rule() == Rule::expr)
        .unwrap();
    Ok(PrintNode { expr: build_expr(expr)? })
}

fn build_input(pair: Pair<Rule>) -> ForgeResult<InputNode> {
    let subtype = pair.into_inner()
        .find(|p| p.as_rule() == Rule::subtype)
        .map(|p| parse_subtype(p.as_str()));
    Ok(InputNode { subtype })
}

fn build_assignment(pair: Pair<Rule>) -> ForgeResult<AssignmentNode> {
    let mut it = pair.into_inner();
    let mut path = vec![it.next().unwrap().as_str().to_string()];
    let mut next = it.next().unwrap();
    while next.as_rule() == Rule::ident {
        path.push(next.as_str().to_string());
        next = it.next().unwrap();
    }
    let value = build_expr(next)?;
    Ok(AssignmentNode { target_path: path, value })
}

fn build_string_parts(pair: Pair<Rule>) -> ForgeResult<Vec<StringPart>> {
    let mut parts = Vec::new();
    let mut buf = String::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::interpolation => {
                if !buf.is_empty() {
                    parts.push(StringPart::Literal(std::mem::take(&mut buf)));
                }
                let path: Vec<String> = inner.into_inner()
                    .filter(|p| p.as_rule() == Rule::ident)
                    .map(|p| p.as_str().to_string())
                    .collect();
                parts.push(StringPart::Interp(path.join(".")));
            }
            Rule::escape => {
                let s = inner.as_str();
                let ch = match s.chars().nth(1).unwrap() {
                    'n'  => '\n',
                    't'  => '\t',
                    'r'  => '\r',
                    '0'  => '\0',
                    '\\' => '\\',
                    '"'  => '"',
                    other => other,
                };
                buf.push(ch);
            }
            _ => {
                buf.push_str(inner.as_str());
            }
        }
    }
    if !buf.is_empty() {
        parts.push(StringPart::Literal(buf));
    }
    Ok(parts)
}

fn build_expr(pair: Pair<Rule>) -> ForgeResult<Expr> {
    match pair.as_rule() {
        Rule::expr             => build_expr(pair.into_inner().next().unwrap()),
        Rule::xor_expr         => build_binary_chain(pair, BinOp::Xor),
        Rule::or_expr          => build_binary_chain(pair, BinOp::Or),
        Rule::and_expr         => build_binary_chain(pair, BinOp::And),
        Rule::comparison       => build_comparison(pair),
        Rule::additive         => build_additive(pair),
        Rule::multiplicative   => build_multiplicative(pair),
        Rule::unary            => build_unary(pair),
        Rule::power            => build_power(pair),
        Rule::postfix          => build_postfix(pair),
        Rule::primary          => build_primary(pair),
        Rule::input_expr       => {
            let st = pair.into_inner().find(|p| p.as_rule() == Rule::subtype).map(|p| p.as_str());
            Ok(Expr::Input(InputNode { subtype: st.map(parse_subtype) }))
        }
        Rule::number           => build_number(pair),
        Rule::string           => {
            let parts = build_string_parts(pair)?;
            Ok(Expr::Str(parts))
        }
        Rule::interpolated_string => {
            let parts = build_string_parts(pair)?;
            Ok(Expr::Str(parts))
        }
        Rule::ident            => Ok(Expr::Identifier(pair.as_str().to_string())),
        r => Err(ForgeError::parse(format!("unexpected expr rule: {:?}", r))),
    }
}

fn build_binary_chain(pair: Pair<Rule>, op: BinOp) -> ForgeResult<Expr> {
    let mut it = pair.into_inner();
    let mut acc = build_expr(it.next().unwrap())?;
    while let Some(_op_pair) = it.next() {
        let rhs = build_expr(it.next().unwrap())?;
        acc = Expr::BinaryOp { op: op.clone(), lhs: Box::new(acc), rhs: Box::new(rhs) };
    }
    Ok(acc)
}

fn build_comparison(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let mut it = pair.into_inner();
    let mut acc = build_expr(it.next().unwrap())?;
    while let Some(op_pair) = it.next() {
        let op = match op_pair.as_str() {
            "==" => BinOp::Eq, "!=" => BinOp::Ne,
            "<"  => BinOp::Lt, ">"  => BinOp::Gt,
            "<=" => BinOp::Le, ">=" => BinOp::Ge,
            s => return Err(ForgeError::parse(format!("bad comp_op: {}", s))),
        };
        let rhs = build_expr(it.next().unwrap())?;
        acc = Expr::BinaryOp { op, lhs: Box::new(acc), rhs: Box::new(rhs) };
    }
    Ok(acc)
}

fn build_additive(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let mut it = pair.into_inner();
    let mut acc = build_expr(it.next().unwrap())?;
    while let Some(op_pair) = it.next() {
        let op = match op_pair.as_str() { "+" => BinOp::Add, "-" => BinOp::Sub, _ => unreachable!() };
        let rhs = build_expr(it.next().unwrap())?;
        acc = Expr::BinaryOp { op, lhs: Box::new(acc), rhs: Box::new(rhs) };
    }
    Ok(acc)
}

fn build_multiplicative(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let mut it = pair.into_inner();
    let mut acc = build_expr(it.next().unwrap())?;
    while let Some(op_pair) = it.next() {
        let op = match op_pair.as_str() { "*" => BinOp::Mul, "/" => BinOp::Div, _ => unreachable!() };
        let rhs = build_expr(it.next().unwrap())?;
        acc = Expr::BinaryOp { op, lhs: Box::new(acc), rhs: Box::new(rhs) };
    }
    Ok(acc)
}

fn build_unary(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let mut it = pair.into_inner();
    let mut ops: Vec<UnOp> = Vec::new();
    let mut next = it.next().unwrap();
    while next.as_rule() == Rule::unary_op {
        ops.push(match next.as_str() { "+" => UnOp::Plus, "-" => UnOp::Neg, _ => unreachable!() });
        next = it.next().unwrap();
    }
    let mut inner = build_expr(next)?;
    for op in ops.into_iter().rev() {
        inner = Expr::UnaryOp { op, operand: Box::new(inner) };
    }
    Ok(inner)
}

fn build_power(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let mut it = pair.into_inner();
    let base = build_expr(it.next().unwrap())?;
    if let Some(_pow_op) = it.next() {
        let rhs = build_expr(it.next().unwrap())?;
        Ok(Expr::BinaryOp { op: BinOp::Pow, lhs: Box::new(base), rhs: Box::new(rhs) })
    } else {
        Ok(base)
    }
}

fn build_postfix(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let mut it = pair.into_inner();
    let mut acc = build_expr(it.next().unwrap())?;
    while let Some(tail) = it.next() {
        match tail.as_rule() {
            Rule::ident => {
                acc = Expr::MemberAccess { object: Box::new(acc), member: tail.as_str().to_string() };
            }
            Rule::call_tail => {
                let args: Vec<Expr> = tail.into_inner()
                    .filter(|p| p.as_rule() == Rule::arg_list)
                    .flat_map(|p| p.into_inner())
                    .map(build_expr)
                    .collect::<ForgeResult<Vec<_>>>()?;
                if let Expr::Identifier(name) = acc {
                    acc = Expr::Call { callee: name, args };
                } else {
                    return Err(nyi("call on non-identifier"));
                }
            }
            r => return Err(ForgeError::parse(format!("unexpected postfix tail: {:?}", r))),
        }
    }
    Ok(acc)
}

fn build_primary(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let inner = pair.into_inner().next().unwrap();
    build_expr(inner)
}

fn build_number(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let s = pair.as_str();
    if s.contains('.') {
        let v: f64 = s.parse().unwrap();
        Ok(Expr::Number(NumberLiteral { int_val: 0, float_val: v, is_float: true }))
    } else {
        let v: i64 = s.parse().unwrap();
        Ok(Expr::Number(NumberLiteral { int_val: v, float_val: 0.0, is_float: false }))
    }
}

fn parse_modifier(s: &str) -> Modifier {
    match s { "Open" => Modifier::Open, "Closed" => Modifier::Closed, "Showcase" => Modifier::Showcase, _ => unreachable!() }
}

fn parse_subtype(s: &str) -> Subtype {
    match s { "Int" => Subtype::Int, "Float" => Subtype::Float, "Generic" => Subtype::Generic, _ => unreachable!() }
}