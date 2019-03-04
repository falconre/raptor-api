use document;
use falcon;
use jsonrpc_http_server::*;
use jsonrpc_http_server::jsonrpc_core::*;
use raptor::ir;
use std::sync::Arc;
use store;
use translate;



fn internal_server_error<S: Into<String>>(description: S) -> jsonrpc_core::Error {
    jsonrpc_core::Error {
        code: jsonrpc_core::ErrorCode::InternalError,
        message: description.into(),
        data: None
    }
}



fn register_api_documents(io: &mut IoHandler, store: Arc<store::Store>) {
    io.add_method("documents", move |_| {
        let mut result = Vec::new();
        for (name, _) in
            store.documents()
                .map_err(|e| internal_server_error(e.description()))?
                .iter() {
            result.push(name.to_string());
        }
        Ok(result.into())
    });
}


fn register_api_document_new(io: &mut IoHandler, store: Arc<store::Store>) {
    io.add_method("document-new", move |params| {
        let params =
            match params {
                Params::Map(values) => values,
                _ => Err(internal_server_error("params must be a map"))?
            };

        let name: String =
            params.get("name")
                .ok_or(internal_server_error("missing name field"))?
                .as_str()
                .ok_or(internal_server_error("name was not a string"))?
                .to_string();
        let bytes: Vec<u8> =
            params.get("bytes")
                .ok_or(internal_server_error("missing bytes"))?
                .as_array()
                .ok_or(internal_server_error("bytes was not an array"))?
                .into_iter()
                .try_fold(Vec::new(), |mut bytes, byte_number| {
                    bytes.push(
                        byte_number.as_u64()
                            .ok_or(internal_server_error("byte was not valid nuber"))?
                        as u8);
                    Ok(bytes)
                })?;

        let loader: Box<falcon::loader::Loader> =
            falcon::loader::Elf::new(bytes.clone(), 0)
                .map(|elf| {
                    let loader: Box<falcon::loader::Loader> = Box::new(elf);
                    loader
                })
                .map_err(|e| internal_server_error(format!("Error parsing binary: {}", e)))?;;

        let mut document = document::Document::new(loader)
            .map_err(|e| internal_server_error(format!(
                "Error loading/lifting binary: {}", e)))?;

        document.translate()
            .map_err(|e| internal_server_error(
                format!("Error translating document: {}", e)))?;

        store.add_document(name, document)
            .map_err(|e| internal_server_error(format!("{}", e)))?;

        Ok(String::from("document added").into())
    });
}


fn register_api_document_functions(io: &mut IoHandler, store: Arc<store::Store>) {
    io.add_method("document-functions", move |params| {
        let params =
            match params {
                Params::Map(values) => values,
                _ => Err(internal_server_error("params must be a map"))?
            };

        let name: String =
            params.get("document-name")
                .ok_or(internal_server_error("missing document-name field"))?
                .as_str()
                .ok_or(internal_server_error("name was not a string"))?
                .to_string();

        let store =
            store.documents()
                .map_err(|e| internal_server_error(e.description()))?;

        let document =
            store
                .get(&name)
                .ok_or(internal_server_error(format!("Could not find document {}", name)))?;

        let functions =
            document.program()
                .map_err(|e| internal_server_error(e.description()))?
                .functions()
                .into_iter()
                .map(|function| {
                    let mut map = serde_json::Map::new();
                    map.insert("index".to_string(), function.index().unwrap().into());
                    map.insert("name".to_string(), function.name().into());
                    map.into()
                })
                .collect::<Vec<Value>>();

        Ok(functions.into())
    });
}


fn register_api_document_xrefs(io: &mut IoHandler, store: Arc<store::Store>) {
    io.add_method("document-xrefs", move |params| {
        let params =
            match params {
                Params::Map(values) => values,
                _ => Err(internal_server_error("params must be a map"))?
            };

        let name: String =
            params.get("document-name")
                .ok_or(internal_server_error("missing document-name field"))?
                .as_str()
                .ok_or(internal_server_error("name was not a string"))?
                .to_string();

        let store =
            store.documents()
                .map_err(|e| internal_server_error(e.description()))?;

        let document =
            store
                .get(&name)
                .ok_or(internal_server_error(format!("Could not find document {}", name)))?;

        Ok(translate::xrefs_to_json(document.xrefs()))
    });
}


fn register_api_function_name(io: &mut IoHandler, store: Arc<store::Store>) {
    io.add_method("function-name", move |params| {
        let params =
            match params {
                Params::Map(values) => values,
                _ => Err(internal_server_error("params must be a map"))?
            };

        let name: String =
            params.get("document-name")
                .ok_or(internal_server_error("missing document-name field"))?
                .as_str()
                .ok_or(internal_server_error("name was not a string"))?
                .to_string();

        let index: usize =
            params.get("function-index")
                .ok_or(internal_server_error("missing function-index field"))?
                .as_u64()
                .ok_or(internal_server_error("index was not a valid number"))?
                as usize;

        let store =
            store.documents()
                .map_err(|e| internal_server_error(e.description()))?;

        let program =
            store
                .get(&name)
                .ok_or(internal_server_error(format!("Could not find document: {}", name)))?
                .program()
                .map_err(|e| internal_server_error(e.description()))?;

        let function =
            program
                .function(index)
                .ok_or(internal_server_error(format!(
                    "Could not find function-index: {}", index)))?;

        Ok(function.name().into())
    });
}


fn register_api_function_ir(io: &mut IoHandler, store: Arc<store::Store>) {
    io.add_method("function-ir", move |params| {
        let params =
            match params {
                Params::Map(values) => values,
                _ => Err(internal_server_error("params must be a map"))?
            };

        let name: String =
            params.get("document-name")
                .ok_or(internal_server_error("missing document-name field"))?
                .as_str()
                .ok_or(internal_server_error("name was not a string"))?
                .to_string();

        let index: usize =
            params.get("function-index")
                .ok_or(internal_server_error("missing function-index field"))?
                .as_u64()
                .ok_or(internal_server_error("index was not a valid number"))?
                as usize;

        let store =
            store.documents()
                .map_err(|e| internal_server_error(e.description()))?;

        let document =
            store
                .get(&name)
                .ok_or(internal_server_error(format!("Could not find document {}", name)))?;

        let program =
            document.program()
                .map_err(|e| internal_server_error(e.description()))?;

        let function =
            program
                .function(index)
                .ok_or(internal_server_error(format!(
                    "Could not find function-index: {}", index)))?;


        Ok(translate::function_to_json(function))
    });
}


fn register_api_instruction_at(io: &mut IoHandler, store: Arc<store::Store>) {
    io.add_method("instruction-at", move |params| {
        let params =
            match params {
                Params::Map(values) => values,
                _ => Err(internal_server_error("params must be a map"))?
            };

        let name: String =
            params.get("document-name")
                .ok_or(internal_server_error("missing document-name field"))?
                .as_str()
                .ok_or(internal_server_error("name was not a string"))?
                .to_string();

        let address: u64 =
            params.get("address")
                .ok_or(internal_server_error("missing function-index field"))?
                .as_u64()
                .ok_or(internal_server_error("index was not a valid number"))?;

        let store =
            store.documents()
                .map_err(|e| internal_server_error(e.description()))?;

        let document =
            store
                .get(&name)
                .ok_or(internal_server_error(format!("Could not find document {}", name)))?;

        let program =
            document.program()
                .map_err(|e| internal_server_error(e.description()))?;

        let rpl =
            ir::RefProgramLocation::from_address(&program, address);

        match rpl {
            Some(rpl) => {
                let mut m = serde_json::Map::new();
                m.insert("function-index".to_string(),
                         rpl.function().index().unwrap().into());
                m.insert("function-name".to_string(),
                         rpl.function().name().into());
                m.insert("block-index".to_string(),
                         rpl.block().unwrap().index().into());
                m.insert("instruction".to_string(),
                         translate::instruction_to_json(rpl.instruction().unwrap()));
                Ok(m.into())
            },
            None => {
                Ok(Value::Null)
            }
        }
    });
}


fn register_api_calls_to_symbol(io: &mut IoHandler, store: Arc<store::Store>) {
    io.add_method("calls-to-symbol", move |params| {
        let params =
            match params {
                Params::Map(values) => values,
                _ => Err(internal_server_error("params must be a map"))?
            };

        let name: String =
            params.get("document-name")
                .ok_or(internal_server_error("missing document-name field"))?
                .as_str()
                .ok_or(internal_server_error("name was not a string"))?
                .to_string();

        let symbol: &str =
            params.get("symbol")
                .ok_or(internal_server_error("missing symbol field"))?
                .as_str()
                .ok_or(internal_server_error("symbol was not a valid string"))?;

        let store =
            store.documents()
                .map_err(|e| internal_server_error(e.description()))?;

        let document =
            store
                .get(&name)
                .ok_or(internal_server_error(format!("Could not find document {}", name)))?;

        let program =
            document.program()
                .map_err(|e| internal_server_error(e.description()))?;

        Ok(program
            .functions()
            .into_iter()
            .flat_map(|function| function.program_locations())
            .filter(|pl| pl.instruction().is_some())
            .filter(|pl| pl.instruction().unwrap().operation().is_call())
            .filter(|pl|
                pl.instruction()
                    .unwrap()
                    .operation()
                    .call()
                    .unwrap()
                    .target()
                    .symbol()
                    .map(|call_symbol| call_symbol == symbol)
                    .unwrap_or(false))
            .map(|pl| translate::program_location_to_json(&pl.into()))
            .collect::<Vec<Value>>().into())
    });
}


pub fn register_endpoints(global_store: Arc<store::Store>) -> IoHandler {

    let mut io = IoHandler::default();
    io.add_method("say_hello", |_| {
        Ok(Value::String("hello".into()))
    });

    register_api_documents(&mut io, global_store.clone());
    register_api_document_new(&mut io, global_store.clone());
    register_api_document_functions(&mut io, global_store.clone());
    register_api_document_xrefs(&mut io, global_store.clone());
    register_api_function_name(&mut io, global_store.clone());
    register_api_function_ir(&mut io, global_store.clone());
    register_api_instruction_at(&mut io, global_store.clone());
    register_api_calls_to_symbol(&mut io, global_store.clone());

    io
}