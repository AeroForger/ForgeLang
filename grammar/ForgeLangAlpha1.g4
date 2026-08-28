grammar ForgeLangAlpha1;

// ============================================================================
// Parser Rules
// ============================================================================

program
    : topLevelItem* EOF
    ;

topLevelItem
    : importStatement
    | dataDeclaration
    | functionDeclaration
    | statement
    ;

// ---------------------------------------------------------------------------
// Imports (Section 3)
// ---------------------------------------------------------------------------

importStatement
    : Use qualifiedName SEMI              # useImport
    | Using qualifiedName COLON ID SEMI   # usingImport
    ;

qualifiedName
    : ID (DOT ID)*
    ;

// ---------------------------------------------------------------------------
// Access Modifiers (Section 5) - public / private / read-only
// ---------------------------------------------------------------------------

modifier
    : Open
    | Closed
    | Showcase
    ;

// ---------------------------------------------------------------------------
// Types (Sections 6-10)
//   Number requires an explicit subtype. Deprecated forms
//   'Number X;' and 'Int.Number X;' are structurally impossible here.
// ---------------------------------------------------------------------------

typeDeclaration
    : numberType
    | Weld
    | oreType
    | materialsType
    | ID
    ;

numberType
    : Number numberKind
    ;

numberKind
    : Int
    | Float
    | Generic
    ;

oreType
    : Ore
    | Ore LBRACK INT RBRACK
    ;

materialsType
    : Materials ID
    ;

// ---------------------------------------------------------------------------
// Declarations
// ---------------------------------------------------------------------------

variableDeclaration
    : standardVarDecl
    | materialsConstructorDecl
    ;

standardVarDecl
    : modifier? typeDeclaration ID initialization? SEMI
    ;

materialsConstructorDecl
    : modifier? materialsType New ID SEMI
    ;

initialization
    : ASSIGN expression
    ;

dataDeclaration
    : modifier? Data ID LBRACE dataMember* RBRACE
    ;

dataMember
    : variableDeclaration
    | functionDeclaration
    ;

objectDeclaration
    : ID ID block
    ;

// ---------------------------------------------------------------------------
// Functions (Sections 17-19)
//   Function  = dynamically returning
//   Nunction  = returns nothing
//   Open/Closed/Showcase are access modifiers, never return kinds.
// ---------------------------------------------------------------------------

functionDeclaration
    : modifier? returnType ID LPAREN parameterList? RPAREN block
    ;

returnType
    : typeDeclaration # typeReturnType
    | Function        # dynamicReturnType
    | Nunction        # noReturnType
    ;

parameterList
    : parameter (COMMA parameter)*
    ;

parameter
    : typeDeclaration ID
    ;

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

statement
    : variableDeclaration
    | objectDeclaration
    | assignment
    | returnStatement
    | ifStatement
    | switchStatement
    | doFailFinal
    | printStatement
    | expressionStatement
    ;

assignment
    : qualifiedName ASSIGN expression SEMI
    ;

returnStatement
    : Return expression? SEMI
    ;

printStatement
    : Print LPAREN expression RPAREN SEMI
    ;

expressionStatement
    : expression SEMI
    ;

// ---------------------------------------------------------------------------
// Control Flow (Sections 23-27)
//   Grammar recognizes the structure. Codegen arrives in Alpha 0.2.
// ---------------------------------------------------------------------------

ifStatement
    : If LPAREN expression RPAREN block
      (Else If LPAREN expression RPAREN block)*
      (Else block)?
    ;

switchStatement
    : Switch LPAREN expression RPAREN LBRACE
      (Deal expression block)*
      (Base block)?
      RBRACE
    ;

doFailFinal
    : Do block (Fail block)? (Final block)?
    ;

block
    : LBRACE statement* RBRACE
    ;

// ---------------------------------------------------------------------------
// Expressions (Section 13)
//   One left-recursive rule; alternative order IS precedence, highest first.
//   Documented decision: unary minus binds looser than '**',
//   so -2 ** 2 == -(2 ** 2) == -4, matching math/Python convention.
// ---------------------------------------------------------------------------

expression
    : literal                                      # literalExpr
    | qualifiedName LPAREN argumentList? RPAREN    # functionCallExpr
    | Input LPAREN inputType? RPAREN               # inputCallExpr
    | qualifiedName                                # memberAccessExpr
    | LPAREN expression RPAREN                     # groupingExpr
    | <assoc=right> expression POW expression      # powExpr
    | (ADD | SUB) expression                       # unaryExpr
    | expression (MUL | DIV) expression            # mulExpr
    | expression (ADD | SUB) expression            # addExpr
    | expression (EQ | NEQ | LT | GT | LTE | GTE) expression  # cmpExpr
    | expression And expression                    # andExpr
    | expression Or expression                     # orExpr
    | expression Xor expression                    # xorExpr
    ;

argumentList
    : expression (COMMA expression)*
    ;
    
inputType
    : Int
    | Float
    | Generic
    ;

literal
    : INT            # intLiteral
    | FLOAT          # floatLiteral
    | STRING         # stringLiteral
    | INTERP_STRING  # interpStringLiteral
    ;

// ============================================================================
// Lexer Rules
// ============================================================================

// Modifiers
Open       : 'Open';
Closed     : 'Closed';
Showcase   : 'Showcase';

// Types & Declarations
Number     : 'Number';
Weld       : 'Weld';
Ore        : 'Ore';
Materials  : 'Materials';
Data       : 'Data';

// Number subtypes (explicit, prevents deprecated 'Number X')
Int        : 'Int';
Float      : 'Float';
Generic    : 'Generic';

// Functions ('function' lowercase by spec; token name capitalized)
Function   : 'function';
Nunction   : 'Nunction';
Return     : 'Return';

// Control flow
If         : 'If';
Else       : 'Else';
Switch     : 'Switch';
Deal       : 'Deal';
Base       : 'Base';
Do         : 'Do';
Fail       : 'Fail';
Final      : 'Final';

// Logic
And        : 'And';
Or         : 'Or';
Xor        : 'Xor';

// Imports
Use        : 'Use';
Using      : 'Using';

// Other
New        : 'New';
Print      : 'Print';
Input      : 'Input';

// Operators (longest match first)
POW    : '**';
LTE    : '<=';
GTE    : '>=';
EQ     : '==';
NEQ    : '!=';
MUL    : '*';
DIV    : '/';
ADD    : '+';
SUB    : '-';
LT     : '<';
GT     : '>';
ASSIGN : '=';

// Punctuation
SEMI   : ';';
COLON  : ':';
LPAREN : '(';
RPAREN : ')';
LBRACE : '{';
RBRACE : '}';
LBRACK : '[';
RBRACK : ']';
DOT    : '.';
COMMA  : ',';

// Literals (escaped quotes allowed; other escapes like \n pass through
// to the code generator, which processes them at compile time)
FLOAT          : [0-9]+ '.' [0-9]+;
INT            : [0-9]+;
STRING         : '"' ( '\\"' | ~[\r\n"] )* '"';
INTERP_STRING  : '\\V"' ( '\\"' | ~[\r\n"] )*? '"';

// Comments & Whitespace
COMMENT  : '-[' .*? ']-' -> skip;
WS       : [ \t\r\n]+ -> skip;

// Identifiers
ID       : [a-zA-Z_][a-zA-Z0-9_]*;