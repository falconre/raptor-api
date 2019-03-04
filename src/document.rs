use error::*;
use falcon::loader::Loader;
use log::info;
use raptor::analysis;
use raptor::features::XRefs;
use raptor::ir;
use raptor::translator::ProgramTranslator;
use rayon::prelude::*;
use std::ops::Deref;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};


pub struct Document {
    loader: Box<Loader>,
    program: RwLock<ir::Program<ir::Constant>>,
    xrefs: XRefs
}


impl Document {
    pub fn new(loader: Box<Loader>) -> Result<Document> {
        let program = loader.program_recursive()?;
        Ok(Document {
            loader: loader,
            program: RwLock::new(ir::Program::<ir::Constant>::from_il(&program)?),
            xrefs: XRefs::new()
        })
    }

    pub fn loader(&self) -> &Loader { self.loader.as_ref() }
    pub fn program(&self) -> Result<RwLockReadGuard<ir::Program<ir::Constant>>> {
        self.program
            .read()
            .map_err(|_| "Lock poisoned for document pogram".into())
    }
    pub fn program_mut(&mut self) -> Result<RwLockWriteGuard<ir::Program<ir::Constant>>> {
        self.program
            .write()
            .map_err(|_| "Lock poisoned for document program".into())
    }

    pub fn xrefs(&self) -> &XRefs { &self.xrefs }

    pub fn translate(&mut self) -> Result<()> {
        info!("Translating functions");
        let functions = {
            let program_translator = ProgramTranslator::new(
                self.loader().architecture().box_clone(),
                self.loader().architecture().calling_convention(),
                self.loader()
            )?;

            let function_translator = program_translator.function_translator();

            let functions:
                ::std::result::Result<
                    Vec<ir::Function<ir::Constant>>,
                    Error
                > =
                self.program()?
                    .functions()
                    // .into_par_iter()
                    // .try_fold(|| Vec::new(), |mut v, function| {
                    .into_iter()
                    .try_fold(Vec::new(), |mut v, function| {
                        println!("{}", function.name());
                        let mut function = match function_translator.optimize_function(function.clone()) {
                            Ok(f) => f,
                            Err(_) => return Ok(v)
                        };
                        // let mut function =
                        //     function_translator.optimize_function(function.clone())?;
                        loop {
                            let new_function =
                                analysis::dead_code_elimination(&function)?;
                            if new_function == function {
                                function = new_function;
                                break;
                            }
                            else {
                                function = new_function;
                            }
                        }
                        info!("Done with {}", function.name());
                        v.push(function);
                        Ok(v)
                    // })  
                    // .try_reduce(|| Vec::new(), |mut v, mut functions| {
                    //     v.append(&mut functions);
                    //     Ok(v)
                    });

            functions?
        };

        for function in functions {
            self.program_mut()?
                .replace_function(function.index().unwrap(), function);
        }

        info!("Computing xrefs");

        let xrefs = {
            XRefs::from_program(self.program()?.deref())
        };

        self.xrefs = xrefs;
        info!("Done");

        Ok(())
    }
}