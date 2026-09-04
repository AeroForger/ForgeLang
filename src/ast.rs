// ForgeLang AST - Rust port of furnace/ast_nodes.py

#[derive(Debug, Clone, PartialEq)]
pub enum Modifier { Open, Closed, Showcase }

#[derive(Debug, Clone, PartialEq)]
pub enum Subtype { Int, Float, Generic, Weld }

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDecl {
    Number(Subtype),
    Weld,
    Bool,
    Ore(Option<i64>),
    OreTuple(Vec<(Subtype, String)>),
    Materials(Subtype, bool /* New */),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RetKind {
    Int, Float, Generic, Weld, Ore, Materials,
    Function, // "function" lowercase = dynamic
    Nunction, // void
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl(VarDecl),
    DataDecl(DataDecl),
    ObjectDecl(ObjectDecl),
    FunctionDecl(FunctionDecl),
    Print(PrintNode),
    Input(InputNode),
    If(IfNode),
    While(WhileNode),
    For(ForNode),
    Return(Option<Expr>),
    Stop,
    Assignment(AssignmentNode),
    Use(UseNode),
    ExprStmt(Expr),
}

/// Increment/decrement direction for For loop headers.
#[derive(Debug, Clone, PartialEq)]
pub enum IncrOp { Inc, Dec }

/// For loop: `For (init; condition; increment) { body }`
#[derive(Debug, Clone)]
pub struct ForNode {
    pub init: VarDecl,
    pub condition: Expr,
    pub increment_var: String,
    pub increment_op: IncrOp,
    pub body: Vec<Statement>,
}


#[derive(Debug, Clone)]
pub struct VarDecl {
    pub modifier: Option<Modifier>,
    pub type_decl: TypeDecl,
    pub name: String,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct DataDecl {
    pub modifier: Option<Modifier>,
    pub name: String,
    pub members: Vec<VarDecl>,
}

#[derive(Debug, Clone)]
pub struct ObjectDecl {
    pub type_name: String,
    pub name: String,
    pub inits: Vec<(Vec<String>, Expr)>, // member path + value
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub modifier: Option<Modifier>,
    pub ret_kind: RetKind,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Param { pub type_decl: TypeDecl, pub name: String }

#[derive(Debug, Clone)]
pub struct PrintNode { pub expr: Expr }

#[derive(Debug, Clone)]
pub enum StringPart {
    Literal(String),
    Interp(String),
}

#[derive(Debug, Clone)]
pub struct InputNode { pub subtype: Option<Subtype> }

#[derive(Debug, Clone)]
pub struct IfNode {
    pub branches: Vec<(Expr, Vec<Statement>)>,
    pub else_body: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct WhileNode {
    pub condition: Expr,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum AssignmentTarget {
    Var(String),
    Member { object: Box<AssignmentTarget>, member: String },
    Index { object: Box<AssignmentTarget>, index: Expr },
}

#[derive(Debug, Clone)]
pub struct AssignmentNode {
    pub target: AssignmentTarget,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct UseNode {
    pub path: Vec<String>,
    pub item: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(NumberLiteral),
    Str(Vec<StringPart>),
    Bool(bool),
    Identifier(String),
    MemberAccess { object: Box<Expr>, member: String },
    IndexAccess { object: Box<Expr>, index: Box<Expr> },
    BinaryOp { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    UnaryOp { op: UnOp, operand: Box<Expr> },
    Call { callee: String, args: Vec<Expr> },
    NamespaceCall { namespace: String, method: String, args: Vec<Expr> },
    MethodCall { object: Box<Expr>, method: String, args: Vec<Expr> },
    Input(InputNode),
    ArrayLiteral(Vec<Expr>),
    TupleLiteral(Vec<Expr>),
    ListLiteral(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteral {
    pub int_val: i64,
    pub float_val: f64,
    pub is_float: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Pow,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or, Xor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp { Plus, Neg }

#[derive(Debug, Clone, Default)]
pub struct Program { pub statements: Vec<Statement> }