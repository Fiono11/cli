use anyhow::{bail, Error};
use reqwest::Url;
use serde_json::{json, Value};
use std::time::Duration;

pub struct RpcClient {
    url: Url,
    client: reqwest::Client,
}

impl RpcClient {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            client: reqwest::ClientBuilder::new()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }

    async fn rpc_request(&self, request: &Value) -> Result<Value, Error> {
        let result = self
            .client
            .post(self.url.clone())
            .json(request)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        if let Some(error) = result.get("error") {
            bail!("node returned error: {}", error);
        }

        Ok(result)
    }

    /* {
      "action": "process",
      "json_block": "true",
      "subtype": "send",
      "block": {
        "type": "state",
        "account": "nano_1qato4k7z3spc8gq1zyd8xeqfbzsoxwo36a45ozbrxcatut7up8ohyardu1z",
        "previous": "6CDDA48608C7843A0AC1122BDD46D9E20E21190986B19EAC23E7F33F2E6A6766",
        "representative": "nano_3pczxuorp48td8645bs3m6c3xotxd3idskrenmi65rbrga5zmkemzhwkaznh",
        "balance": "40200000001000000000000000000000000",
        "link": "87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9",
        "link_as_account": "nano_33t5by1653nt196hfwm5q3wq7oxtaix97r7bhox5zn8eratrzoqsny49ftsd",
        "signature": "A5DB164F6B81648F914E49CAB533900C389FAAD64FBB24F6902F9261312B29F730D07E9BCCD21D918301419B4E05B181637CF8419ED4DCBF8EF2539EB2467F07",
        "work": "000bc55b014e807d"
      }
    } */

    pub async fn receivable(&self, account: &str, count: u32) -> Result<Value, Error> {
        let request = json!({
            "action": "receivable",
            "account": account,
            "count": count,
            "threshold": 1,
        });
        println!("request: {:?}", request);
        self.rpc_request(&request).await
    }

    pub async fn account_balance(&self, account: &str) -> Result<Value, Error> {
        let request = json!({
            "action": "account_balance",
            "account": account,
        });
        println!("request: {:?}", request);
        self.rpc_request(&request).await
    }

    pub async fn account_history(&self, account: &str, count: u32) -> Result<Value, Error> {
        let request = json!({
            "action": "account_history",
            "account": account,
            "count": count,
        });
        println!("request: {:?}", request);
        self.rpc_request(&request).await
    }

    pub async fn work_generate(&self, hash: &str) -> Result<Value, Error> {
        let request = json!({
            "action": "work_generate",
            "hash": hash,
        });
        println!("request: {:?}", request);
        self.rpc_request(&request).await
    }

    pub async fn process(&self, subtype: &str, block: &str) -> Result<Value, Error> {
        //let request = Request::new(subtype, block);
        //let request_json = serde_json::to_value(&request)?;

        let request_json = json!({
            "action": "process",
            "json_block": "false",
            "subtype": subtype,
            "block": block
        });

        println!("request: {:?}", request_json);
        self.rpc_request(&request_json).await
    }

    pub async fn receive_block(
        &self,
        wallet: &str,
        destination: &str,
        block: &str,
    ) -> Result<Value, Error> {
        let request = json!({
            "action": "receive",
            "wallet": wallet,
            "account": destination,
            "block": block
        });
        self.rpc_request(&request).await
    }

    pub async fn account_info_rpc(&self, account: &str) -> Result<AccountInfo, Error> {
        let request = json!({
            "action": "account_info",
            "account": account
        });

        let json = self.rpc_request(&request).await?;

        Ok(AccountInfo {
            frontier: json["frontier"].as_str().unwrap().to_owned(),
            block_count: json["block_count"].as_str().unwrap().to_owned(),
            balance: json["balance"].as_str().unwrap().to_owned(),
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct AccountInfo {
    pub frontier: String,
    pub block_count: String,
    pub balance: String,
}
