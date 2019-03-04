use falcon::il;
use jsonrpc_http_server::jsonrpc_core::Value;
use raptor::features::XRefs;
use raptor::ir;
use serde_json::Map;



pub fn variable_to_json(variable: &ir::Variable) -> Value {
    match variable {
        ir::Variable::Scalar(scalar) => {
            let mut m = Map::new();
            m.insert("type".to_string(), "scalar".into());
            m.insert("name".to_string(), scalar.name().into());
            m.insert("bits".to_string(), scalar.bits().into());
            m.into()
        },
        ir::Variable::StackVariable(stack_variable) => {
            let mut m = Map::new();
            m.insert("type".to_string(), "stack_variable".into());
            m.insert("offset".to_string(), stack_variable.offset().into());
            m.insert("bits".to_string(), stack_variable.bits().into());
            m.into()
        }
    }
}


pub fn lvalue_to_json(lvalue: &ir::LValue<ir::Constant>) -> Value {
    match lvalue {
        ir::LValue::Variable(variable) => variable_to_json(variable),
        ir::LValue::Dereference(dereference) => {
            let mut m = Map::new();
            m.insert("type".to_string(), "dereference".into());
            m.insert("expression".to_string(),
                     expression_to_json(dereference.expression()));
            m.into()
        }
    }
}


pub fn constant_to_json(constant: &ir::Constant) -> Value {
    let mut m = Map::new();
    m.insert("type".to_string(), "constant".into());
    m.insert("value".to_string(),
             format!("0x{}", constant.value().to_str_radix(16)).into());
    m.insert("bits".to_string(), constant.bits().into());
    m.into()
}


pub fn rvalue_to_json(rvalue: &ir::RValue<ir::Constant>) -> Value {
    match rvalue {
        ir::RValue::Value(constant) => constant_to_json(constant),
        ir::RValue::Reference(reference) => {
            let mut m = Map::new();
            m.insert("type".to_string(), "reference".into());
            m.insert("expression".to_string(),
                     expression_to_json(reference.expression()));
            m.into()
        }
    }
}



pub fn expression_to_json(expression: &ir::Expression<ir::Constant>) -> Value {
    fn binop(
        op: &str,
        lhs: &ir::Expression<ir::Constant>,
        rhs: &ir::Expression<ir::Constant>
    ) -> Value {
        let mut m = Map::new();
        m.insert("op".to_string(), op.into());
        m.insert("lhs".to_string(), expression_to_json(lhs));
        m.insert("rhs".to_string(), expression_to_json(rhs));
        m.into()

    }

    match expression {
        ir::Expression::LValue(lvalue) => lvalue_to_json(lvalue),
        ir::Expression::RValue(rvalue) => rvalue_to_json(rvalue),
        ir::Expression::Add(lhs, rhs) => binop("add", lhs, rhs),
        ir::Expression::Sub(lhs, rhs) => binop("sub", lhs, rhs),
        ir::Expression::Mul(lhs, rhs) => binop("mul", lhs, rhs),
        ir::Expression::Divu(lhs, rhs) => binop("divu", lhs, rhs),
        ir::Expression::Modu(lhs, rhs) => binop("modu", lhs, rhs),
        ir::Expression::Divs(lhs, rhs) => binop("divs", lhs, rhs),
        ir::Expression::Mods(lhs, rhs) => binop("mods", lhs, rhs),
        ir::Expression::And(lhs, rhs) => binop("and", lhs, rhs),
        ir::Expression::Or(lhs, rhs) => binop("or", lhs, rhs),
        ir::Expression::Xor(lhs, rhs) => binop("xor", lhs, rhs),
        ir::Expression::Shl(lhs, rhs) => binop("shl", lhs, rhs),
        ir::Expression::Shr(lhs, rhs) => binop("shr", lhs, rhs),
        ir::Expression::Cmpeq(lhs, rhs) => binop("cmpeq", lhs, rhs),
        ir::Expression::Cmpneq(lhs, rhs) => binop("cmpneq", lhs, rhs),
        ir::Expression::Cmplts(lhs, rhs) => binop("cmplts", lhs, rhs),
        ir::Expression::Cmpltu(lhs, rhs) => binop("cmpltu", lhs, rhs),
        ir::Expression::Trun(bits, rhs) => {
            let mut m = Map::new();
            m.insert("op".to_string(), "trun".into());
            m.insert("bits".to_string(), (*bits).into());
            m.insert("rhs".to_string(), expression_to_json(rhs));
            m.into()
        },
        ir::Expression::Sext(bits, rhs) => {
            let mut m = Map::new();
            m.insert("op".to_string(), "sext".into());
            m.insert("bits".to_string(), (*bits).into());
            m.insert("rhs".to_string(), expression_to_json(rhs));
            m.into()
        },
        ir::Expression::Zext(bits, rhs) => {
            let mut m = Map::new();
            m.insert("op".to_string(), "zext".into());
            m.insert("bits".to_string(), (*bits).into());
            m.insert("rhs".to_string(), expression_to_json(rhs));
            m.into()
        },
        ir::Expression::Ite(cond, then, else_) => {
            let mut m = Map::new();
            m.insert("op".to_string(), "ite".into());
            m.insert("cond".to_string(), expression_to_json(cond));
            m.insert("then".to_string(), expression_to_json(then));
            m.insert("else".to_string(), expression_to_json(else_));
            m.into()
        }
    }
}


pub fn call_to_json(call: &ir::Call<ir::Constant>) -> Value {
    let mut m = Map::new();

    let mut target = Map::new();
    match call.target() {
        ir::CallTarget::Expression(expression) => {
            target.insert("type".to_string(), "expression".into());
            target.insert("expression".to_string(),
                          expression_to_json(expression));
        },
        ir::CallTarget::Symbol(symbol) => {
            target.insert("type".to_string(), "symbol".into());
            target.insert("symbol".to_string(), symbol.to_string().into());
        },
        ir::CallTarget::FunctionId(function_id) => {
            target.insert("type".to_string(), "function_id".into());
            target.insert("function_id".to_string(), (*function_id).into());
        }
    }

    let arguments: Value =
        call.arguments()
            .map(|arguments| arguments
                .into_iter()
                .map(|argument| expression_to_json(argument))
                .collect::<Vec<Value>>()
                .into())
            .unwrap_or(Value::Null);

    let variables_written: Value =
        call.variables_written()
            .map(|variables_written| variables_written
                .into_iter()
                .map(|variable| variable_to_json(variable))
                .collect::<Vec<Value>>()
                .into())
            .unwrap_or(Value::Null);

    m.insert("target".to_string(), target.into());
    m.insert("arguments".to_string(), arguments.into());
    m.insert("variables_written".to_string(), variables_written.into());

    m.into()
}


pub fn intrinsic_to_json(intrinsic: &il::Intrinsic) -> Value {
    let mut m = Map::new();

    m.insert("mnemonic".to_string(), intrinsic.mnemonic().into());
    m.insert("instruction_str".to_string(), intrinsic.instruction_str().into());
    m.insert("arguments".to_string(),
        intrinsic.arguments()
            .into_iter()
            .map(|expression|
                expression_to_json(&ir::Expression::from_il(expression)))
            .collect::<Vec<Value>>()
            .into());
    m.insert("written_expressions".to_string(),
        intrinsic.written_expressions()
            .map(|written_expressions| written_expressions
                .into_iter()
                .map(|expression|
                    expression_to_json(&ir::Expression::from_il(expression)))
                .collect::<Vec<Value>>()
                .into())
            .unwrap_or(Value::Null));
    m.insert("read_expressions".to_string(),
        intrinsic.read_expressions()
            .map(|read_expressions| read_expressions
                .into_iter()
                .map(|expression|
                    expression_to_json(&ir::Expression::from_il(expression)))
                .collect::<Vec<Value>>()
                .into())
            .unwrap_or(Value::Null));
    m.insert("bytes".to_string(), intrinsic.bytes().clone().into());

    m.into()
}


pub fn operation_to_json(operation: &ir::Operation<ir::Constant>) -> Value {
    let mut m = Map::new();
    match operation {
        ir::Operation::Assign { dst, src } => {
            m.insert("operation".to_string(), "assign".into());
            m.insert("dst".to_string(), variable_to_json(dst));
            m.insert("src".to_string(), expression_to_json(src));
        },
        ir::Operation::Store { index, src } => {
            m.insert("operation".to_string(), "store".into());
            m.insert("index".to_string(), expression_to_json(index));
            m.insert("src".to_string(), expression_to_json(src));
        },
        ir::Operation::Load { dst, index } => {
            m.insert("operation".to_string(), "load".into());
            m.insert("dst".to_string(), variable_to_json(dst));
            m.insert("index".to_string(), expression_to_json(index));
        },
        ir::Operation::Branch { target } => {
            m.insert("operation".to_string(), "branch".into());
            m.insert("target".to_string(), expression_to_json(target));
        },
        ir::Operation::Call(call) => {
            m.insert("operation".to_string(), "call".into());
            m.insert("call".to_string(), call_to_json(call));
        },
        ir::Operation::Intrinsic(intrinsic) => {
            m.insert("operation".to_string(), "intrinsic".into());
            m.insert("intrinsic".to_string(), intrinsic_to_json(intrinsic));
        },
        ir::Operation::Return(result) => {
            m.insert("operation".to_string(), "return".into());
            m.insert("result".to_string(),
                     result.as_ref().map(|e| expression_to_json(e))
                        .unwrap_or(Value::Null));
        }
        ir::Operation::Nop => {
            m.insert("operation".to_string(), "nop".into());
        }
    }
    m.into()
}


pub fn instruction_to_json(instruction: &ir::Instruction<ir::Constant>) -> Value {
    let mut m = Map::new();

    m.insert("operation".to_string(),
             operation_to_json(instruction.operation()));
    m.insert("index".to_string(), instruction.index().into());
    m.insert("comment".to_string(),
        instruction.comment()
            .map(|comment| comment.into())
            .unwrap_or(Value::Null));
    m.insert("address".to_string(),
        instruction.address()
            .map(|address| address.into())
            .unwrap_or(Value::Null));

    m.into()
}


pub fn block_to_json(block: &ir::Block<ir::Constant>) -> Value {
    let mut m = Map::new();

    m.insert("index".to_string(), block.index().into());
    m.insert("instructions".to_string(),
        block.instructions()
            .into_iter()
            .map(|instruction| instruction_to_json(instruction))
            .collect::<Vec<Value>>()
            .into());

    m.into()
}


pub fn edge_to_json(edge: &ir::Edge<ir::Constant>) -> Value {
    let mut m = Map::new();

    m.insert("head".to_string(), edge.head().into());
    m.insert("tail".to_string(), edge.tail().into());
    m.insert("condition".to_string(),
        edge.condition()
            .map(|condition| expression_to_json(condition))
            .unwrap_or(Value::Null));
    m.insert("comment".to_string(),
        edge.comment()
            .map(|comment| comment.into())
            .unwrap_or(Value::Null));

    m.into()
}


pub fn function_to_json(function: &ir::Function<ir::Constant>) -> Value {
    let mut m = Map::new();

    m.insert("address".to_string(), function.address().into());
    m.insert("index".to_string(),
        function.index()
            .map(|index| index.into())
            .unwrap_or(Value::Null));
    m.insert("name".to_string(), function.name().into());
    m.insert("blocks".to_string(),
        function.blocks()
            .into_iter()
            .map(|block| block_to_json(block))
            .collect::<Vec<Value>>()
            .into());
    m.insert("edges".to_string(),
        function.edges()
            .into_iter()
            .map(|edge| edge_to_json(edge))
            .collect::<Vec<Value>>()
            .into());

    m.into()
}


pub fn function_location_to_json(fl: &ir::FunctionLocation) -> Value {
    let mut m = Map::new();

    match *fl {
        ir::FunctionLocation::Instruction(block_index, instruction_index) => {
            m.insert("block-index".to_string(), block_index.into());
            m.insert("instruction-index".to_string(),
                     instruction_index.into());
        },
        ir::FunctionLocation::EmptyBlock(block_index) => {
            m.insert("block-index".to_string(), block_index.into());
        },
        ir::FunctionLocation::Edge(head_index, tail_index) => {
            m.insert("edge-head".to_string(), head_index.into());
            m.insert("edge-tail".to_string(), tail_index.into());
        }
    }

    m.into()
}


pub fn program_location_to_json(pl: &ir::ProgramLocation) -> Value {

    let mut m = Map::new();

    m.insert("function-index".to_string(), pl.function_index().into());
    m.insert("function-location".to_string(),
             function_location_to_json(pl.function_location()));

    m.into()
}


pub fn xrefs_to_json(xrefs: &XRefs) -> Value {
    let mut from_to = Map::new();

    for (from, to) in xrefs.from_to() {
        let to: Vec<Value> = to.into_iter().map(|to| (*to).into()).collect();
        from_to.insert(format!("{}", from), to.into());
    }

    let mut to_from = Map::new();

    for (to, from) in xrefs.to_from() {
        let from: Vec<Value> = from.into_iter().map(|from| (*from).into()).collect();
        to_from.insert(format!("{}", to), from.into());
    }

    let mut m = Map::new();

    m.insert("from_to".to_string(), from_to.into());
    m.insert("to_from".to_string(), to_from.into());

    m.into()
}