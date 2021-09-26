use parity_wasm;
use parity_wasm::elements;

use wasmer;

use crate::ast;

pub fn expr_to_ins(a: &ast::JustExpr) -> Vec<elements::Instruction> {
    match a {
        ast::Expr::IntLit(_, n) => vec![elements::Instruction::I64Const(*n)],
        ast::Expr::BoolLit(_, b) => vec![elements::Instruction::I64Const(*b as i64)],
        ast::Expr::TypeAnno { term, .. } => expr_to_ins(term),
        ast::Expr::IfFlow { cond, on_true, on_false, .. } => {
            let mut if_ins = vec![];

            if_ins.append(&mut expr_to_ins(cond));
            if_ins.push(elements::Instruction::I32WrapI64);

            if_ins.push(elements::Instruction::If(elements::BlockType::Value(elements::ValueType::I64)));
            if_ins.append(&mut expr_to_ins(on_true));
            if_ins.push(elements::Instruction::Else);
            if_ins.append(&mut expr_to_ins(on_false));
            if_ins.push(elements::Instruction::End);

            if_ins
        },
        ast::Expr::Error => panic!("Internal compiler error"),
    }
}

pub fn ast_to_wasm(a: &ast::JustExpr) -> elements::Module {
    let ins = elements::Instructions::new(expr_to_ins(a));

    parity_wasm::builder::module()
        .function()
            .signature()
                .with_result(elements::ValueType::I64)
                .build()

            .body()
                .with_instructions(ins)
                .build()

            .build()

        .export()
            .field("main")
            .internal()
            .func(0)
            .build()

		.build()
}

/*Copub fn eval(m: &elements::Module) {
    let store = wasmer::Store::default();
    let module = wasmer::Module::new(&store, m.to_bytes().unwrap()).unwrap();

    // The module doesn't import anything, so we create an empty import object.
    let import_object = imports! {};
    let instance = wasmer::Instance::new(module_bytes, &import_object)?;

    let main = instance.exports.get_function("main").unwrap();
    let output = main.call(&[]).unwrap();
    
    match output {
        wasmer::Value::I64(n) => println!("{}", n),
        _ => panic("Internal compiler error"),
    }
}*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch() {

    }
}