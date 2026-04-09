//! Control flow compilation: if/else, while, for-in, c-style for, break, continue.

use crate::ast::ast::{Expr, Ident, Program, Stmt};
use crate::vm::compiler::{Compiler, JumpPatch, LoopContext};
use crate::vm::instruction::Instruction;

/// Compiles an if/else expression.
pub fn compile_if_expr(
    compiler: &mut Compiler,
    cond: &Expr,
    consequence: &Program,
    alternative: &Option<Program>,
    line: u16,
) {
    compiler.compile_expression(cond, line);
    let else_jump = compiler.emit_jump_if_false(line);

    compile_program_body(compiler, consequence);

    if let Some(alt) = alternative {
        let end_jump = compiler.emit_jump(line);
        compiler.patch_jump(else_jump);
        compile_program_body(compiler, alt);
        compiler.patch_jump(end_jump);
    } else {
        let end_jump = compiler.emit_jump(line);
        compiler.patch_jump(else_jump);
        compiler.emit_constant(crate::runtime::obj::Object::Null, line);
        compiler.patch_jump(end_jump);
    }
}

/// Compiles a while loop expression.
pub fn compile_while_expr(compiler: &mut Compiler, cond: &Expr, body: &Program, line: u16) {
    let loop_start = compiler.chunk.current_offset();

    compiler.loop_contexts.push(LoopContext {
        break_patches: Vec::new(),
        continue_patches: vec![JumpPatch {
            addr: loop_start as usize,
        }],
    });

    compiler.compile_expression(cond, line);
    let end_jump = compiler.emit_jump_if_false(line);

    compile_program_body(compiler, body);
    if !body.is_empty() {
        compiler.emit(Instruction::Pop, line);
    }

    compiler.emit(Instruction::JumpBackward(loop_start), line);

    let loop_ctx = compiler.loop_contexts.pop().unwrap();
    for patch in loop_ctx.break_patches {
        compiler
            .chunk
            .patch_u16(patch.addr, compiler.chunk.current_offset());
    }

    compiler
        .chunk
        .patch_u16(end_jump.addr, compiler.chunk.current_offset());

    compiler.emit_constant(crate::runtime::obj::Object::Null, line);
}

/// Compiles a for-in loop expression.
pub fn compile_for_expr(
    compiler: &mut Compiler,
    idents: &[Ident],
    iterable: &Expr,
    body: &Program,
    line: u16,
) {
    use crate::runtime::obj::Object;

    // Allocate slots for the iterable and counter AFTER the ident slots.
    // The ident slots are assigned by compute_slots.
    let base_slot = if !idents.is_empty() && idents[0].slot != crate::ast::ast::SlotIndex::UNSET {
        idents[0].slot.0 as u8
    } else {
        0
    };
    let iter_slot = base_slot + idents.len() as u8;
    let counter_slot = iter_slot + 1;

    compiler.compile_expression(iterable, line);
    compiler.emit(Instruction::SetLocal(iter_slot), line);

    compiler.emit_constant(Object::Integer(0), line);
    compiler.emit(Instruction::SetLocal(counter_slot), line);

    let loop_start = compiler.chunk.current_offset();

    compiler.loop_contexts.push(LoopContext {
        break_patches: Vec::new(),
        continue_patches: vec![JumpPatch {
            addr: loop_start as usize,
        }],
    });

    compiler.emit(Instruction::GetLocal(counter_slot), line);
    compiler.emit(Instruction::GetLocal(iter_slot), line);
    compiler.emit(Instruction::GetLen, line);
    compiler.emit(Instruction::LessThan, line);
    let end_jump = compiler.emit_jump_if_false(line);

    compiler.emit(Instruction::GetLocal(iter_slot), line);
    compiler.emit(Instruction::GetLocal(counter_slot), line);
    compiler.emit(Instruction::Index, line);

    if idents.len() == 1 {
        let ident = &idents[0];
        if ident.slot != crate::ast::ast::SlotIndex::UNSET {
            compiler.emit(Instruction::SetLocal(ident.slot.0 as u8), line);
        }
    } else {
        for (i, ident) in idents.iter().enumerate() {
            compiler.emit(Instruction::Dup, line);
            compiler.emit_constant(Object::Integer(i as i64), line);
            compiler.emit(Instruction::Index, line);
            if ident.slot != crate::ast::ast::SlotIndex::UNSET {
                compiler.emit(Instruction::SetLocal(ident.slot.0 as u8), line);
            }
        }
        compiler.emit(Instruction::Pop, line);
    }

    compile_program_body(compiler, body);
    if !body.is_empty() {
        compiler.emit(Instruction::Pop, line);
    }

    compiler.emit(Instruction::GetLocal(counter_slot), line);
    compiler.emit_constant(Object::Integer(1), line);
    compiler.emit(Instruction::Add, line);
    compiler.emit(Instruction::SetLocal(counter_slot), line);

    compiler.emit(Instruction::JumpBackward(loop_start), line);

    let loop_ctx = compiler.loop_contexts.pop().unwrap();
    for patch in loop_ctx.break_patches {
        compiler
            .chunk
            .patch_u16(patch.addr, compiler.chunk.current_offset());
    }

    compiler
        .chunk
        .patch_u16(end_jump.addr, compiler.chunk.current_offset());

    compiler.emit_constant(Object::Null, line);
}

/// Compiles a C-style for loop.
pub fn compile_cstyle_for(
    compiler: &mut Compiler,
    init: &Option<Box<Stmt>>,
    cond: &Option<Box<Expr>>,
    update: &Option<Box<Stmt>>,
    body: &Program,
    line: u16,
) {
    use crate::runtime::obj::Object;

    if let Some(init_stmt) = init {
        compiler.compile_statement(&init_stmt, line);
    }

    let cond_start = compiler.chunk.current_offset();

    compiler.loop_contexts.push(LoopContext {
        break_patches: Vec::new(),
        continue_patches: Vec::new(),
    });

    if let Some(cond_expr) = cond {
        compiler.compile_expression(cond_expr, line);
    } else {
        compiler.emit_constant(Object::Boolean(true), line);
    }
    let end_jump = compiler.emit_jump_if_false(line);

    compile_program_body(compiler, body);
    if !body.is_empty() {
        compiler.emit(Instruction::Pop, line);
    }

    let continue_addr = compiler.chunk.current_offset();

    if let Some(update_stmt) = update {
        compiler.compile_statement(&update_stmt, line);
    }

    compiler.emit(Instruction::JumpBackward(cond_start), line);

    compiler
        .chunk
        .patch_u16(end_jump.addr, compiler.chunk.current_offset());

    let loop_ctx = compiler.loop_contexts.pop().unwrap();
    for patch in loop_ctx.continue_patches {
        compiler.chunk.patch_u16(patch.addr, continue_addr);
    }

    for patch in loop_ctx.break_patches {
        compiler
            .chunk
            .patch_u16(patch.addr, compiler.chunk.current_offset());
    }

    compiler.emit_constant(Object::Null, line);
}

/// Emits a `break` instruction and records it for backpatching.
pub fn compile_break(compiler: &mut Compiler, line: u16) {
    if compiler.loop_contexts.is_empty() {
        return;
    }
    let offset = compiler.chunk.code.len();
    compiler.emit(Instruction::Break(0), line);
    compiler
        .loop_contexts
        .last_mut()
        .unwrap()
        .break_patches
        .push(JumpPatch { addr: offset + 1 });
}

/// Emits a `continue` instruction and records it for backpatching.
pub fn compile_continue(compiler: &mut Compiler, line: u16) {
    if compiler.loop_contexts.is_empty() {
        return;
    }
    let offset = compiler.chunk.code.len();
    compiler.emit(Instruction::Continue(0), line);
    compiler
        .loop_contexts
        .last_mut()
        .unwrap()
        .continue_patches
        .push(JumpPatch { addr: offset + 1 });
}

/// Compiles a sequence of statements for block bodies.
/// Pops intermediate expression results but keeps the last one.
fn compile_program_body(compiler: &mut Compiler, program: &Program) {
    for (i, stmt) in program.iter().enumerate() {
        let line = 0u16;
        compiler.compile_statement(stmt, line);

        if i < program.len() - 1 {
            match stmt {
                Stmt::ExprStmt(_) => {
                    compiler.emit(Instruction::Pop, line);
                }
                _ => {}
            }
        }
    }
}
