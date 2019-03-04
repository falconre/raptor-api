#[macro_use] extern crate error_chain;
extern crate falcon;
extern crate jsonrpc_http_server;
extern crate log;
extern crate owning_ref;
extern crate raptor;
extern crate rayon;
extern crate serde_json;
extern crate simplelog;

use jsonrpc_http_server::*;
use std::sync::Arc;

mod register_api;

pub mod document;
pub mod store;
pub mod translate;


pub mod error {
    error_chain! {
        types {
            Error, ErrorKind, ResultExt, Result;
        }

        foreign_links {
            Falcon(::falcon::error::Error);
            Raptor(::raptor::error::Error);
        }
    }
}


fn main() {
    // simplelog::TermLogger::init(
    //     simplelog::LevelFilter::Debug,
    //     simplelog::Config::default()
    // ).expect("Failed to initialize logging");

    let global_store: Arc<store::Store> = Arc::new(store::Store::new());

    let io = register_api::register_endpoints(global_store);

    let server = ServerBuilder::new(io)
        // .cors(DomainsValidation::AllowOnly(vec![AccessControlAllowOrigin::Any]))
        .cors(DomainsValidation::Disabled)
        .threads(8)
        .start_http(&"0.0.0.0:3030".parse().unwrap())
        .expect("Unable to start RPC server");

    server.wait();
}