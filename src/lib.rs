extern crate futures;

extern crate grpc;
extern crate httpbis;
extern crate protobuf;
extern crate serde;
extern crate serde_json;

use crate::protos::{api, api_grpc::{self, Dgraph}};
//use std::sync::{Arc, Mutex};

use futures::compat::Future01CompatExt;

use errors::DgraphError;
use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand::seq::SliceRandom;

pub mod errors;
pub mod protos;


pub struct DgraphClient
{
    //    _jwt_mutex: Option<Arc<Mutex<api::Jwt>>>,
    dc: Vec<api_grpc::DgraphClient>,
    rng_seed: [u8; 16],
}

impl DgraphClient
{
    pub fn new(dc: Vec<api_grpc::DgraphClient>) -> Self {
        assert!(!dc.is_empty());
        Self {
//            jwt_mutex: None,
            rng_seed: rand::thread_rng().gen(),
            dc,
        }
    }

    pub fn new_txn(&self) -> Txn {
        Txn {
            context: Default::default(),
            finished: false,
            read_only: false,
            best_effort: false,
            mutated: false,
            dc: self.any_client(),
        }
    }


    pub fn new_read_only(&self) -> Txn {
        Txn {
            context: Default::default(),
            finished: false,
            read_only: true,
            best_effort: false,
            mutated: false,
            dc: self.any_client(),
        }
    }


    pub fn new_best_effort(&self) -> Txn {
        Txn {
            context: Default::default(),
            finished: false,
            read_only: true,
            best_effort: true,
            mutated: false,
            dc: self.any_client(),
        }
    }

    fn any_client(&self) -> &api_grpc::DgraphClient {
        let mut rng = rand_xoshiro::Xoroshiro128Plus::from_seed(self.rng_seed);

        self.dc.choose(&mut rng).unwrap()
    }
}

pub struct Txn<'a> {
    context: api::TxnContext,
    finished: bool,
    read_only: bool,
    best_effort: bool,
    mutated: bool,
    dc: &'a api_grpc::DgraphClient,
}

impl<'a> Txn<'a> {
    pub async fn query(&mut self, q: impl Into<String>) -> Result<api::Response, DgraphError> {
        self.query_with_vars(q, HashMap::new()).await
    }

    pub async fn query_with_vars(
        &mut self,
        q: impl Into<String>,
        vars: HashMap<String, String>,
    ) -> Result<api::Response, DgraphError> {
        self._do(
            api::Request {
                query: q.into(),
                start_ts: self.context.start_ts,
                read_only: self.read_only,
                best_effort: self.best_effort,
                vars,
                ..Default::default()
            }
        ).await
    }

    pub async fn mutate(&mut self, mu: api::Mutation) -> Result<api::Response, DgraphError> {
        self._do(
            api::Request {
                start_ts: self.context.start_ts,
                commit_now: mu.commit_now,
                mutations: vec![mu].into(),
                ..Default::default()
            }
        ).await
    }

    pub async fn upsert(&mut self, q: impl Into<String>, mut mu: api::Mutation) -> Result<api::Response, DgraphError> {
        mu.commit_now = true;
        self._do(
            api::Request {
                query: q.into(),
                mutations: vec![mu].into(),
                commit_now: true,
                ..Default::default()
            }
        ).await
    }


    pub async fn commit(&mut self) -> Result<(), DgraphError> {
        match (self.read_only, self.finished) {
            (true, _) => return Err(DgraphError::ReadOnly),
            (_, true) => return Err(DgraphError::Finished),
            _ => self.commit_or_abort().await,
        }
    }

    pub async fn commit_or_abort(&mut self) -> Result<(), DgraphError> {
        if self.finished {
            return Ok(());
        }
        self.finished = true;

        if !self.mutated {
            return Ok(());
        }

        self.dc.commit_or_abort(
            Default::default(),
            self.context.clone(),
        ).join_metadata_result().compat().await?;

        Ok(())
    }

    pub async fn discard(&mut self) -> Result<(), DgraphError> {
        self.context.aborted = true;
        self.commit_or_abort().await
    }

    async fn _do(&mut self, mut req: api::Request) -> Result<api::Response, DgraphError> {
        if self.finished {
            return Err(DgraphError::Finished);
        }

        if !req.mutations.is_empty() {
            if self.read_only {
                return Err(DgraphError::ReadOnly);
            }
            self.mutated = true;
        }

        req.start_ts = self.context.start_ts;

        let commit_now = req.commit_now;

        let query_res = self.dc.query(
            Default::default(),
            req,
        ).join_metadata_result().compat().await;

        // TODO: Handle JWT failure by logging in again
        if let Err(_) = query_res {
            let _ = self.discard().await;
        }
        let query_res = query_res?;

        if commit_now {
            self.finished = true;
        }

        let txn = match query_res.1.txn.as_ref() {
            Some(txn) => txn,
            None => return Err(DgraphError::EmptyTransaction)
        };

        self.merge_context(txn)?;
        Ok(query_res.1)
    }

    fn merge_context(&mut self, src: &api::TxnContext) -> Result<(), DgraphError> {
        if self.context.start_ts == 0 {
            self.context.start_ts = src.start_ts;
        }

        if self.context.start_ts != src.start_ts {
            return Err(DgraphError::StartTsMismatch);
        }

        for key in src.keys.iter() {
            self.context.keys.push(key.clone());
        }

        for pred in src.preds.iter() {
            self.context.preds.push(pred.clone());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::Value;
    use grpc::{ClientStub, Client, ClientConf};
    use std::sync::Arc;

    fn local_dgraph_client() -> DgraphClient {
        let addr = "localhost";
        let port = 9080;

        let client = api_grpc::DgraphClient::with_client(
            Arc::new(
                Client::new_plain(addr.as_ref(), port, ClientConf {
                    ..Default::default()
                }).expect("Failed to initialize client stub")
            )
        );

        DgraphClient::new(vec![client])
    }

    // This is a basic smoke test - query for node_key, assert we get a response
    #[test]
    fn test_query() {
        async_std::task::block_on(async {
            println!("Connecting to local dg");
            let dg = local_dgraph_client();
            let mut txn = dg.new_txn();
            println!("Querying local dg");
            let query_res: Value = txn.query(r#"
                {
                  q0(func: has(node_key)) {
                    uid
                  }
                }
            "#)
                .await
                .map(|res| serde_json::from_slice(&res.json))
                .expect("Dgraph query failed")
                .expect("Json deserialize failed");

            // Assert that we get the response back
            assert!(query_res.as_object().unwrap().contains_key("q0"));
        });
    }

    #[test]
    fn test_upsert() {
        async_std::task::block_on(async {
            let dg = local_dgraph_client();

            let query = r#"
                {
                  p as var(func: eq(node_key, "{453120d4-5c9f-43f6-b7af-28b376b3a993}"))
                }
                "#;

            let j_mut = serde_json::json!{
                {
                    "uid": "uid(p)",
                    "node_key": "{453120d4-5c9f-43f6-b7af-28b376b3a993}",
                    "process_name": "bar.exe",
                }
            };

            let mu = api::Mutation {
                set_json: j_mut.to_string().into_bytes(),
                ..Default::default()
            };
            let mut txn = dg.new_txn();
            let txn_res = txn.upsert(
                query, mu,
            )
                .await
                .expect("Request to dgraph failed");
            dbg!(txn_res);

            txn.commit_or_abort().await.unwrap();

//            let mut txn = dg.new_txn();
//            let query_res: Value = txn.query(r#"
//                {
//                  q0(func: has(node_key)) {
//                    uid
//                  }
//                }
//            "#)
//                .await
//                .map(|res| serde_json::from_slice(&res.json))
//                .expect("Dgraph query failed")
//                .expect("Json deserialize failed");
////
//            dbg!(query_res)
        });
    }


    #[test]
    fn test_txn_query_mutate() {
        async_std::task::block_on(async {
            let dg = local_dgraph_client();

            let query = r#"
                {
                  q0(func: eq(node_key, "{453120d4-5c9f-43f6-b7af-28b376b3a993}")) {
                    uid
                  }
                }
                "#;

            let mut txn = dg.new_read_only();

            let query_res: Value = txn.query(query).await
                .map(|res| serde_json::from_slice(&res.json))
                .expect("query")
                .expect("json");

            let uid = query_res.get("q0")
                .and_then(|res| res.get(0))
                .and_then(|uid| uid.get("uid"))
                .and_then(|uid| uid.as_str());
//                .to_string();
            dbg!(&uid);

            let mut set = serde_json::json!({
                "node_key": "{453120d4-5c9f-43f6-b7af-28b376b3a993}",
                "process_name": "bar.exe",
            });

            if let Some(uid) = uid {
                set["uid"] = uid.into();
            }
            let j_mut = set;

            let mu = api::Mutation {
                set_json: j_mut.to_string().into_bytes(),
                ..Default::default()
            };
            let mut txn = dg.new_txn();
            let txn_res = txn.mutate(mu)
                .await
                .expect("Request to dgraph failed");
            dbg!(txn_res);

            txn.commit_or_abort().await.unwrap();

            let mut txn = dg.new_read_only();

            let query_res: Value = txn.query(query).await
                .map(|res| serde_json::from_slice(&res.json))
                .expect("query")
                .expect("json");
            dbg!(query_res)
        });
    }
}
