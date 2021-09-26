use parity_wasm;
use parity_wasm::elements;

use wasmer;
use wasmer_compiler_cranelift;

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
    let mut ins = expr_to_ins(a);

    // Functions have to finish with an `End` instruction
    ins.push(elements::Instruction::End);

    parity_wasm::builder::module()
        .function()
            .signature()
                .with_result(elements::ValueType::I64)
                .build()

            .body()
                .with_instructions(elements::Instructions::new(ins))
                .build()

            .build()

        .export()
            .field("main")
            .internal()
            .func(0)
            .build()

		.build()
}

pub fn eval(parity_module: elements::Module, final_ty: &ast::JustType) {
    let compiler = wasmer_compiler_cranelift::Cranelift::new();
    let store = wasmer::Store::new(&wasmer::Universal::new(compiler).engine());
    let module = wasmer::Module::from_binary(&store, &parity_module.to_bytes().unwrap()).unwrap();

    let import_object = wasmer::imports! {};
    let instance = wasmer::Instance::new(&module, &import_object).unwrap();

    let main = instance.exports.get_function("main").unwrap();
    let output = main.call(&[]).unwrap();
    
    match (&output[0], final_ty) {
        (wasmer::Value::I64(n), ast::Type::Int(_)) => println!("{}", n),
        (wasmer::Value::I64(n), ast::Type::Bool(_)) => println!("{}", n == &1),
        _ => panic!("Internal compiler error"),
    }
}