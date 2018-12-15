#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_cpupool;
extern crate grpc;
extern crate httpbis;
extern crate protobuf;
extern crate serde;
extern crate serde_json;


use failure::Error;

use crate::protos::{api, api_grpc::{self, Dgraph}};

pub mod protos;

pub struct Transaction<'a> {
    context: api::TxnContext,
    finished: bool,
    read_only: bool,
    mutated: bool,
    client: &'a api_grpc::DgraphClient,
}

impl<'a> Transaction<'a> {

    pub fn query(&mut self, query: impl Into<String>) -> Result<api::Response, Error> {

        if self.finished {
            bail!("Transaction is completed");
        }

        let res = self.client.query(Default::default(),
                               api::Request {
                                   query: query.into(),
                                   ..Default::default()
                               }
        ).wait()?;

        let txn = match res.1.txn.as_ref() {
            Some(txn) => txn,
            None => bail!("Got empty transaction response back from query")
        };

        self.merge_context(txn)?;
        Ok(res.1)
    }

    pub fn mutate(&mut self, mut mu: api::Mutation) -> Result<api::Assigned, Error> {

        match (self.finished, self.read_only) {
            (true, _) => bail!("Transaction is finished"),
            (_, true) => bail!("Transaction is read only"),
            _ => ()
        }

        self.mutated = true;
        mu.start_ts = self.context.start_ts;
        let commit_now = mu.commit_now;
        let mu_res = self.client.mutate(
            Default::default(),
            mu
        ).wait();

        let mu_res = match mu_res {
            Ok(mu_res) => mu_res,
            Err(e) => {
                let _ = self.discard();
                bail!(e);
            }
        };

        if commit_now {
            self.finished = true;
        }

        let context = match mu_res.1.context.as_ref() {
            Some(context) => context,
            None => bail!("Missing transaction context on mutation response")
        };

        self.merge_context(context)?;
        Ok(mu_res.1)
    }

    pub fn commit(mut self) -> Result<(), Error> {
        match (self.finished, self.read_only) {
            (true, _) => bail!("Transaction is finished"),
            (_, true) => bail!("Transaction is read only"),
            _ => ()
        }

        self.finished = true;

        if !self.mutated {
            return Ok(())
        }


        self.client.commit_or_abort(Default::default(), self.context.clone())
            .wait()?;

        Ok(())
    }

    fn discard(&mut self) -> Result<(), Error> {
        if self.finished {
            return Ok(())
        }

        self.finished = true;

        if !self.mutated {
            return Ok(())
        }

        self.context.aborted = true;

        self.client.commit_or_abort(Default::default(), self.context.clone())
            .wait()?;

        Ok(())
    }

    fn merge_context(&mut self, src: &api::TxnContext) -> Result<(), Error> {
        if self.context.start_ts == 0 {
            self.context.start_ts = src.start_ts;
        }

        if self.context.start_ts != src.start_ts {
            bail!("self.context.start_ts != src.start_ts")
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
    use std::sync::Arc;

    use grpc::{Client, ClientStub};
    use grpc::ClientConf;
    use std::collections::HashMap;

    #[test]
    fn insert_query() -> Result<(), Error> {
        let addr = "localhost";
        let port = 9080;

        let client = api_grpc::DgraphClient::with_client(
            Arc::new(
                Client::new_plain(addr.as_ref(), port, ClientConf {
                    ..Default::default()
                })?
            )
        );

        client.alter(
            Default::default(),
            api::Operation {
                drop_all: true,
                ..Default::default()
            }
        ).wait()?;

        client.alter(
            Default::default(),
            api::Operation {
                schema: "key: string @upsert @index(hash) .".into(),
                ..Default::default()
            }
        ).wait()?;


        client.mutate(
            Default::default(),
            api::Mutation {
                set_json: r#"[{"key": "1234"}, {"key": "4567"}]"#.into(),
                commit_now: true,
                ..Default::default()
            }
        ).wait()?;


        let query = r#"
            {
                q1(func: eq(key, "1234")) {
                    uid,
                },
                q2(func: eq(key, "4567")) {
                    uid,
                }
            }
            "#;

        let res = client.query(Default::default(),
            api::Request {
                query: query.into(),
                ..Default::default()
            }
        ).wait()?;

        let json_res: HashMap<String, serde_json::Value> = serde_json::from_slice(&res.1.json)?;

        // Assert that we get uids back for both
        &json_res["q1"][0]["uid"];
        &json_res["q2"][0]["uid"];

        // Assert that we get uids back for only those 2
        assert!(&json_res.keys().len() == &2);

        assert!(&json_res["q1"].as_array().unwrap().len() == &1);
        assert!(&json_res["q2"].as_array().unwrap().len() == &1);

        Ok(())
    }


    #[test]
    fn insert_query_txn() -> Result<(), Error> {
        let addr = "localhost";
        let port = 9080;

        let client = api_grpc::DgraphClient::with_client(
            Arc::new(
                Client::new_plain(addr.as_ref(), port, ClientConf {
                    ..Default::default()
                })?
            )
        );

        client.alter(
            Default::default(),
            api::Operation {
                drop_all: true,
                ..Default::default()
            }
        ).wait()?;

        client.alter(
            Default::default(),
            api::Operation {
                schema: "key: string @upsert @index(hash) .".into(),
                ..Default::default()
            }
        ).wait()?;



        let mut handles = vec![];
        for _ in 0..50 {
            let handle = std::thread::spawn(move || {

                let client = &api_grpc::DgraphClient::with_client(
                    Arc::new(
                        Client::new_plain(addr.as_ref(), port, ClientConf {
                            ..Default::default()
                        })?
                    )
                );

                let mut tx = Transaction {
                    context: api::TxnContext::default(),
                    finished: false,
                    read_only: false,
                    mutated: false,
                    client,
                };

                let query = r#"
                    {
                        q1(func: eq(key, "1234")) {
                            uid,
                        },
                        q2(func: eq(key, "4567")) {
                            uid,
                        }
                    }
                    "#;

                // hack for type inference
                if false {
                    bail!("impossible")
                }

                let res = tx.query(query)?;
                let json_res: HashMap<String, serde_json::Value> = serde_json::from_slice(&res.json)?;

                let uid1 = json_res.get("q1").and_then(|a| a.get(0)).and_then(|m| m.get("uid"));

                let uid2 = json_res.get("q2").and_then(|a| a.get(0)).and_then(|m| m.get("uid"));


                if let (Some(uid1), Some(uid2)) = (uid1, uid2) {
                    let update = format!(
                        r#"[{{"key": "1234", "uid": {}, "update": "true"}}, {{"key": "4567", "uid": {}, "update": "true"}}]"#,
                        uid1, uid2
                    );

                    tx.mutate(
                        api::Mutation {
                            set_json: update.into_bytes(),
                            commit_now: false,
                            ..Default::default()
                        }
                    )?;
                } else {
                    tx.mutate(
                        api::Mutation {
                            set_json: r#"[{"key": "1234"}, {"key": "4567"}]"#.into(),
                            commit_now: false,
                            ..Default::default()
                        }
                    )?;
                }

                tx.commit()?;

                Ok(())
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();  // TODO: Ensure only transaction failures occurred
        }

        // Assert that they are created
        let query = r#"
            {
                q1(func: eq(key, "1234")) {
                    uid,
                    update
                },
                q2(func: eq(key, "4567")) {
                    uid,
                    update
                }
            }
            "#;

        let res = client.query(Default::default(),
                               api::Request {
                                   query: query.into(),
                                   ..Default::default()
                               }
        ).wait()?;

        let json_res: HashMap<String, serde_json::Value> = serde_json::from_slice(&res.1.json)?;
        println!("{:#?}", json_res);

        // Assert that we get uids back for both
        &json_res["q1"][0]["uid"];
        &json_res["q2"][0]["uid"];
        &json_res["q1"][0]["update"];
        &json_res["q2"][0]["update"];

        // Assert that we get uids back for only those 2
        assert!(&json_res.keys().len() == &2);

        assert!(&json_res["q1"].as_array().unwrap().len() == &1);
        assert!(&json_res["q2"].as_array().unwrap().len() == &1);

        Ok(())
    }
}
