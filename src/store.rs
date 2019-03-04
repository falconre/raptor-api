use document::Document;
use error::*;
use owning_ref::RwLockReadGuardRef;
use std::collections::HashMap;
use std::sync::RwLock;


pub struct Store {
    documents: RwLock<HashMap<String, Document>>
}

impl Store {
    pub fn new() -> Store {
        Store {
            documents: RwLock::new(HashMap::new())
        }
    }

    pub fn documents(&self) -> Result<RwLockReadGuardRef<HashMap<String, Document>, HashMap<String, Document>>> {
        let documents = match self.documents.read() {
            Ok(documents) => RwLockReadGuardRef::new(documents),
            Err(_) => bail!("Lock poisoned for store document")
        };
        
        Ok(documents)
    }

    pub fn add_document<S: Into<String>>(
        &self,
        name: S,
        document: Document
    ) -> Result<()> {
        println!("Adding document");
        match self.documents.write() {
            Ok(mut documents) => documents.insert(name.into(), document),
            Err(_) => bail!("Lock poisoned for store document")
        };
        println!("Document added");
        Ok(())
    }
}