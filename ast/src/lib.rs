#![feature(box_patterns)]

use derive_more::{Deref, DerefMut, From};
use enum_as_inner::EnumAsInner;
use enum_dispatch::enum_dispatch;
use itertools::Itertools;
use std::fmt;

mod assign;
mod binary;
mod r#break;
mod call;
mod r#continue;
mod global;
mod goto;
mod r#if;
mod index;
mod literal;
mod local;
pub mod local_allocator;
mod name_gen;
mod r#return;
mod table;
mod unary;
mod r#while;
mod side_effects;

pub use assign::*;
pub use binary::*;
pub use call::*;
pub use global::*;
pub use goto::*;
pub use index::*;
pub use literal::*;
pub use local::*;
pub use r#break::*;
pub use r#continue::*;
pub use r#if::*;
pub use r#return::*;
pub use r#while::*;
pub use table::*;
pub use unary::*;
pub use side_effects::*;

pub trait Reduce {
    fn reduce(self) -> RValue;
}

#[enum_dispatch(LocalRw, SideEffects)]
#[derive(Debug, Clone, PartialEq, EnumAsInner)]
pub enum RValue {
    Local(RcLocal),
    Global(Global),
    Call(Call),
    Table(Table),
    Literal(Literal),
    Index(Index),
    Unary(Unary),
    Binary(Binary),
}

impl<'a: 'b, 'b> Reduce for RValue {
    fn reduce(self) -> RValue {
        match self {
            Self::Unary(unary) => unary.reduce(),
            Self::Binary(binary) => binary.reduce(),
            other => other,
        }
    }
}

impl RValue {
    pub fn precedence(&self) -> usize {
        match self {
            Self::Binary(binary) => binary.precedence(),
            _ => 0,
        }
    }
}

impl fmt::Display for RValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RValue::Local(local) => write!(f, "{}", local),
            RValue::Global(global) => write!(f, "{}", global),
            RValue::Literal(literal) => write!(f, "{}", literal),
            RValue::Call(call) => write!(f, "{}", call),
            RValue::Table(table) => write!(f, "{}", table),
            RValue::Index(index) => write!(f, "{}", index),
            RValue::Unary(unary) => write!(f, "{}", unary),
            RValue::Binary(binary) => write!(f, "{}", binary),
        }
    }
}

#[enum_dispatch(SideEffects)]
#[derive(Debug, Clone, PartialEq, EnumAsInner)]
pub enum LValue {
    Local(RcLocal),
    Global(Global),
    Index(Index),
}

impl LocalRw for LValue {
    fn values_read(&self) -> Vec<&RcLocal> {
        match self {
            LValue::Local(_) => Vec::new(),
            LValue::Global(global) => global.values_read(),
            LValue::Index(index) => index.values_read(),
        }
    }

    fn values_read_mut(&mut self) -> Vec<&mut RcLocal> {
        match self {
            LValue::Local(_) => Vec::new(),
            LValue::Global(global) => global.values_read_mut(),
            LValue::Index(index) => index.values_read_mut(),
        }
    }

    fn values_written(&self) -> Vec<&RcLocal> {
        match self {
            LValue::Local(local) => vec![local],
            LValue::Global(global) => global.values_written(),
            LValue::Index(index) => index.values_written(),
        }
    }

    fn values_written_mut(&mut self) -> Vec<&mut RcLocal> {
        match self {
            LValue::Local(local) => vec![local],
            LValue::Global(global) => global.values_written_mut(),
            LValue::Index(index) => index.values_written_mut(),
        }
    }
}

impl fmt::Display for LValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LValue::Local(local) => write!(f, "{}", local),
            LValue::Global(global) => write!(f, "{}", global),
            LValue::Index(index) => write!(f, "{}", index),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub text: String,
}

impl Comment {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl SideEffects for Comment {}

impl LocalRw for Comment {}

#[enum_dispatch(LocalRw, SideEffects)]
#[derive(Debug, Clone, EnumAsInner)]
pub enum Statement {
    Call(Call),
    Assign(Assign),
    If(If),
    Goto(Goto),
    Label(Label),
    While(While),
    Return(Return),
    Continue(Continue),
    Break(Break),
    Comment(Comment),
}

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-- {}", self.text)
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Statement::Call(call) => write!(f, "{}", call),
            Statement::Assign(assign) => write!(f, "{}", assign),
            Statement::If(if_) => write!(f, "{}", if_),
            Statement::Goto(goto) => write!(f, "{}", goto),
            Statement::Label(label) => write!(f, "{}", label),
            Statement::While(while_) => write!(f, "{}", while_),
            Statement::Return(return_) => write!(f, "{}", return_),
            Statement::Continue(continue_) => write!(f, "{}", continue_),
            Statement::Break(break_) => write!(f, "{}", break_),
            Statement::Comment(comment) => write!(f, "{}", comment),
        }
    }
}

#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct Block(pub Vec<Statement>);

impl Block {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(statements: Vec<Statement>) -> Self {
        Self(statements)
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0.iter().map(|node| node.to_string()).join("\n")
        )
    }
}