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
        Rule::stop_stmt      => Statement::Stop,
        Rule::input_stmt     => Statement::Input(build_input(inner)?),
        Rule::function_decl  => Statement::FunctionDecl(build_function_decl(inner)?),
        Rule::if_stmt        => Statement::If(build_if_stmt(inner)?),
         Rule::while_stmt     => Statement::While(build_while_stmt(inner)?),
         Rule::for_stmt       => Statement::For(build_for_stmt(inner)?),
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
    let type_decl = build_type_decl(next)?;
    let name = it.next().unwrap().as_str().to_string();
    let initializer = it.next().map(build_expr).transpose()?;
    Ok(VarDecl { modifier, type_decl, name, initializer })
}

fn build_data_decl(pair: Pair<Rule>) -> ForgeResult<DataDecl> {
    let mut modifier = None;
    let mut name = String::new();
    let mut members = Vec::new();
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::modifier => modifier = Some(parse_modifier(p.as_str())),
            Rule::ident => name = p.as_str().to_string(),
            Rule::member_decl => members.push(build_member_decl(p)?),
            _ => {}
        }
    }
    Ok(DataDecl { modifier, name, members })
}

fn build_member_decl(pair: Pair<Rule>) -> ForgeResult<VarDecl> {
    let mut modifier = None;
    let mut type_decl = None;
    let mut name = String::new();
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::modifier => modifier = Some(parse_modifier(p.as_str())),
            Rule::type_decl => type_decl = Some(build_type_decl(p)?),
            Rule::ident => name = p.as_str().to_string(),
            _ => {}
        }
    }
    let type_decl = type_decl.ok_or_else(|| ForgeError::parse("missing type_decl in member_decl"))?;
    Ok(VarDecl { modifier, type_decl, name, initializer: None })
}

fn build_object_decl(pair: Pair<Rule>) -> ForgeResult<ObjectDecl> {
    let mut it = pair.into_inner();
    let type_name = it.next().unwrap().as_str().to_string();
    let name = it.next().unwrap().as_str().to_string();
    let mut inits = Vec::new();
    for member_init in it {
        let mut mit = member_init.into_inner();
        let mut path = Vec::new();
        let mut next = mit.next().unwrap();
        while next.as_rule() == Rule::ident {
            path.push(next.as_str().to_string());
            next = mit.next().unwrap();
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

    it.next().unwrap(); // kw_if
    let cond = build_expr(it.next().unwrap())?;
    let body = build_block(it.next().unwrap())?;
    branches.push((cond, body));

    while let Some(next) = it.next() {
        if next.as_rule() == Rule::kw_else {
            if let Some(after_else) = it.next() {
                if after_else.as_rule() == Rule::kw_if {
                    let elif_cond = build_expr(it.next().unwrap())?;
                    let elif_body = build_block(it.next().unwrap())?;
                    branches.push((elif_cond, elif_body));
                } else if after_else.as_rule() == Rule::block {
                    else_body = Some(build_block(after_else)?);
                }
            }
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

fn build_for_stmt(pair: Pair<Rule>) -> ForgeResult<ForNode> {
    let mut it = pair.into_inner();
    it.next().unwrap(); // kw_for
    let for_init = it.next().unwrap();
    let init = build_var_decl(for_init)?;
    let condition = build_expr(it.next().unwrap())?;
    let for_increment = it.next().unwrap();
    let block = it.next().unwrap();

    let mut incr_it = for_increment.into_inner();
    let increment_var = incr_it.next().unwrap().as_str().to_string();
    let op_str = incr_it.next().unwrap().as_str();
    let increment_op = match op_str {
        "++" => IncrOp::Inc,
        "--" => IncrOp::Dec,
        _ => return Err(ForgeError::parse(format!("unknown increment op: {}", op_str))),
    };

    let body = build_block(block)?;
    Ok(ForNode { init, condition, increment_var, increment_op, body })
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
        Rule::bool_type => Ok(TypeDecl::Bool),
        Rule::ore_type => {
            let mut it = inner.into_inner().filter(|p| p.as_rule() != Rule::kw_ore);
            let first = it.next().unwrap();
            match first.as_rule() {
                Rule::integer => {
                    let size: i64 = first.as_str().parse().map_err(|e| ForgeError::parse(format!("invalid integer: {}", e)))?;
                    Ok(TypeDecl::Ore(Some(size)))
                }
                Rule::kw_empty => Ok(TypeDecl::Ore(None)),
                Rule::tuple_field => {
                    let mut fields = Vec::new();
                    let parse_tf = |p: Pair<Rule>| -> ForgeResult<(Subtype, String)> {
                        let mut fit = p.into_inner();
                        let st = fit.next().unwrap().as_str();
                        let name = fit.next().unwrap().as_str().to_string();
                        Ok((parse_subtype(st), name))
                    };
                    fields.push(parse_tf(first)?);
                    for p in it {
                        if p.as_rule() == Rule::tuple_field {
                            fields.push(parse_tf(p)?);
                        }
                    }
                    Ok(TypeDecl::OreTuple(fields))
                }
                r => Err(ForgeError::parse(format!("unexpected ore_type rule: {:?}", r))),
            }
        }
        Rule::materials_type => {
            let mut it = inner.into_inner();
            let st = it.find(|p| p.as_rule() == Rule::subtype).unwrap().as_str();
            let has_new = it.any(|p| p.as_rule() == Rule::kw_new);
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
    let first = it.next().unwrap();
    let target = build_assignment_target(first)?;
    let value = build_expr(it.next().unwrap())?;
    Ok(AssignmentNode { target, value })
}

fn build_assignment_target(pair: Pair<Rule>) -> ForgeResult<AssignmentTarget> {
    let mut it = pair.into_inner();
    let first = it.next().ok_or_else(|| ForgeError::parse("empty assignment target"))?;
    let mut target = AssignmentTarget::Var(first.as_str().to_string());
    
    while let Some(next) = it.next() {
        match next.as_rule() {
            Rule::ident | Rule::member_ident => {
                target = AssignmentTarget::Member {
                    object: Box::new(target),
                    member: next.as_str().to_string(),
                };
            }
            Rule::index_tail => {
                let index_pair = next.into_inner().next().unwrap();
                let index = build_expr(index_pair)?;
                target = AssignmentTarget::Index {
                    object: Box::new(target),
                    index,
                };
            }
            _ => {}
        }
    }
    
    Ok(target)
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
        Rule::array_init       => build_array_init(pair),
        Rule::tuple_init       => build_tuple_init(pair),
        Rule::list_init        => build_list_init(pair),
        Rule::input_expr       => {
            let st = pair.into_inner().find(|p| p.as_rule() == Rule::subtype).map(|p| p.as_str());
            Ok(Expr::Input(InputNode { subtype: st.map(parse_subtype) }))
        }
        Rule::number           => build_number(pair),
        Rule::bool_literal     => match pair.as_str() {
            "true" => Ok(Expr::Bool(true)),
            "false" => Ok(Expr::Bool(false)),
            _ => unreachable!(),
        },
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

fn build_array_init(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let elements = pair.into_inner()
        .map(build_expr)
        .collect::<ForgeResult<Vec<_>>>()?;
    Ok(Expr::ArrayLiteral(elements))
}

fn build_tuple_init(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let elements = pair.into_inner()
        .map(build_expr)
        .collect::<ForgeResult<Vec<_>>>()?;
    Ok(Expr::TupleLiteral(elements))
}

fn build_list_init(pair: Pair<Rule>) -> ForgeResult<Expr> {
    let elements = pair.into_inner()
        .map(build_expr)
        .collect::<ForgeResult<Vec<_>>>()?;
    Ok(Expr::ListLiteral(elements))
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
            Rule::ident | Rule::member_ident => {
                acc = Expr::MemberAccess { object: Box::new(acc), member: tail.as_str().to_string() };
            }
            Rule::index_tail => {
                let index_pair = tail.into_inner().next().unwrap();
                let index = build_expr(index_pair)?;
                acc = Expr::IndexAccess { object: Box::new(acc), index: Box::new(index) };
            }
            Rule::call_tail => {
                let args: Vec<Expr> = tail.into_inner()
                    .filter(|p| p.as_rule() == Rule::arg_list)
                    .flat_map(|p| p.into_inner())
                    .map(build_expr)
                    .collect::<ForgeResult<Vec<_>>>()?;
                match acc {
                    Expr::Identifier(name) => {
                        acc = Expr::Call { callee: name, args };
                    }
                    Expr::MemberAccess { object, member } => {
                        if let Expr::Identifier(namespace) = &*object {
                            if namespace == "Program" {
                                acc = Expr::NamespaceCall {
                                    namespace: namespace.clone(),
                                    method: member,
                                    args,
                                };
                                continue;
                            }
                        }
                        acc = Expr::MethodCall {
                            object,
                            method: member,
                            args,
                        };
                    }
                    _ => return Err(nyi("call on non-identifier")),
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
    match s {
        "Int" => Subtype::Int,
        "Float" => Subtype::Float,
        "Generic" => Subtype::Generic,
        "Weld" => Subtype::Weld,
        _ => unreachable!(),
    }
}