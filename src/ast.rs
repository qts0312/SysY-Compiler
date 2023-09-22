#[derive(Debug)]
pub struct CompUnit {
    pub items: Vec<CompItem>,
}

#[derive(Debug)]
pub enum CompItem {
    Func(FuncDef),
    Decl(Decl),
}

#[derive(Debug)]
pub enum Decl {
    Const(ConstDecl),
    Var(VarDecl),
}

#[derive(Debug)]
pub struct ConstDecl {
    pub defs: Vec<ConstDef>,
}

#[derive(Debug)]
pub struct ConstDef {
    pub id: String,
    pub dims: Vec<ConstExp>,
    pub init: ConstInitVal,
}

#[derive(Debug)]
pub enum ConstInitVal {
    Exp(ConstExp),
    List(Box<Vec<ConstInitVal>>),
}

#[derive(Debug)]
pub struct VarDecl {
    pub defs: Vec<VarDef>,
}

#[derive(Debug)]
pub struct VarDef {
    pub id: String,
    pub dims: Vec<ConstExp>,
    pub init: Option<InitVal>,
}

#[derive(Debug)]
pub enum InitVal {
    Exp(Exp),
    List(Box<Vec<InitVal>>),
}

#[derive(Debug)]
pub struct FuncDef {
    pub ty: FuncType,
    pub id: String,
    pub params: Vec<FuncParam>,
    pub body: Block,
}

#[derive(Debug)]
pub struct FuncParam {
    pub id: String,
    pub dims: Option<Vec<ConstExp>>,
}

#[derive(Debug)]
pub enum FuncType {
    Void,
    Int,
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

#[derive(Debug)]
pub enum BlockItem {
    Stmt(Stmt),
    Decl(Decl),
}

#[derive(Debug)]
pub enum Stmt {
    Return(Return),
    Assign(Assign),
    Exp(Option<Exp>),
    Block(Box<Block>),
    If(Box<If>),
    While(Box<While>),
    Break(Break),
    Continue(Continue),
}

#[derive(Debug)]
pub struct Return {
    pub exp: Option<Exp>,
}

#[derive(Debug)]
pub struct Assign {
    pub lval: LVal,
    pub exp: Exp,
}

#[derive(Debug)]
pub struct If {
    pub cond: Exp,
    pub then: Stmt,
    pub els: Option<Stmt>,
}

#[derive(Debug)]
pub struct While {
    pub cond: Exp,
    pub body: Stmt,
}

#[derive(Debug)]
pub struct Break;

#[derive(Debug)]
pub struct Continue;

#[derive(Debug)]
pub struct Exp {
    pub lor: LOrExp,
}

#[derive(Debug)]
pub struct LVal {
    pub id: String,
    pub dims: Vec<Exp>,
}

#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    LVal(LVal),
    Num(i32),
}

#[derive(Debug)]
pub enum UnaryExp {
    Primary(PrimaryExp),
    Call(Box<Call>),
    Unary(UnaryOp, Box<UnaryExp>),
}

#[derive(Debug)]
pub struct Call {
    pub id: String,
    pub args: Vec<Exp>,
}

#[derive(Debug)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

#[derive(Debug)]
pub enum MulExp {
    Unary(UnaryExp),
    Mul(Box<MulExp>, MulOp, UnaryExp),
}

#[derive(Debug)]
pub enum MulOp {
    Mul,
    Div,
    Mod,
}

#[derive(Debug)]
pub enum AddExp {
    Mul(MulExp),
    Add(Box<AddExp>, AddOp, MulExp),
}

#[derive(Debug)]
pub enum AddOp {
    Add,
    Sub,
}

#[derive(Debug)]
pub enum RelExp {
    Add(AddExp),
    Rel(Box<RelExp>, RelOp, AddExp),
}

#[derive(Debug)]
pub enum RelOp {
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug)]
pub enum EqExp {
    Rel(RelExp),
    Eq(Box<EqExp>, EqOp, RelExp),
}

#[derive(Debug)]
pub enum EqOp {
    Eq,
    Ne,
}

#[derive(Debug)]
pub enum LAndExp {
    Eq(EqExp),
    LAnd(Box<LAndExp>, EqExp),
}

#[derive(Debug)]
pub enum LOrExp {
    LAnd(LAndExp),
    LOr(Box<LOrExp>, LAndExp),
}

#[derive(Debug)]
pub struct ConstExp {
    pub exp: Exp,
}
