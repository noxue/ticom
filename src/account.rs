use std::thread;
use std::time::Duration;

use log::{debug, error, info};
use reqwest::header::{HeaderMap};
use reqwest::header::{ACCEPT, REFERER, USER_AGENT};
use reqwest::{self, Url};
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use thirtyfour::common::cookie::Cookie as TCookie;
use thirtyfour::prelude::*;

#[derive(Debug,Clone)]
pub struct Account {
    client: Client,
}

/// 一些辅助函数
impl Account {

    fn gen_default_headers() -> HeaderMap {
        let mut default_headers = HeaderMap::new();

        default_headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36".parse().unwrap());
        default_headers.insert(REFERER, "https://www.ti.com/".parse().unwrap());
        default_headers.insert(
            "sec-ch-ua",
            r#"" Not A;Brand";v="99", "Chromium";v="96", "Google Chrome";v="96""#
                .parse()
                .unwrap(),
        );
        default_headers.insert("sec-ch-ua-mobile", r#"?0"#.parse().unwrap());
        default_headers.insert("sec-ch-ua-platform", r#""Windows""#.parse().unwrap());
        default_headers.insert("Upgrade-Insecure-Requests", r#"1"#.parse().unwrap());
        default_headers.insert(ACCEPT,r#"text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"#.parse().unwrap());

        default_headers.insert("Sec-Fetch-Site", r#"same-origin"#.parse().unwrap());
        default_headers.insert("Sec-Fetch-Mode", r#"navigate"#.parse().unwrap());
        default_headers.insert("Sec-Fetch-User", r#"?1"#.parse().unwrap());
        default_headers.insert("Sec-Fetch-Dest", r#"document"#.parse().unwrap());
        default_headers.insert("Accept-Language", r#"zh"#.parse().unwrap());
        default_headers.insert("Accept-Encoding", r#"gzip, deflate, br"#.parse().unwrap());

        default_headers
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Store {
    orderable_number: String,
    inventory: usize,
}

impl Account {
    pub async fn new() -> Self {
        // 调用selenium 模拟登录，获取登录后的cookie
        // let cookies = Self::login(username, password).await;

        // 根据获取到的cookie创建 reqwest client
        let client = {
            let cookie_store = cookie_store::CookieStore::default();

            let cookie_store =
                std::sync::Arc::new(reqwest_cookie_store::CookieStoreMutex::new(cookie_store));

            let default_headers = Self::gen_default_headers();

            reqwest::Client::builder()
                .default_headers(default_headers.clone())
                // .proxy(Proxy::all("http://123.128.12.87:30001").unwrap())
                .cookie_provider(cookie_store)
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap()
        };
        Account { client }
    }

    pub async fn get_store_by_product_name(&self, product_name: &str) -> Result<usize, String> {
        debug!("正在获取产品库存:{}", product_name);

        let res = match self
            .client
            .get(format!(
                "https://www.ti.com/storeservices/cart/opninventory?opn={}",
                product_name
            ))
            .send()
            .await
        {
            Ok(v) => v,
            Err(e) => {
                error!("获取库存出错:{}", e);
                return Err(format!("{}", e));
            }
        };

        let text = match res.text().await {
            Ok(v) => v,
            Err(e) => {
                error!("获取库存返回的html出错:{}", e);
                return Err(format!("{}", e));
            }
        };

        let store: Store = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                error!("json解析库存返回的内容出错:{}", e);
                return Err(format!("{}", e));
            }
        };

        debug!(
            "获取产品：{} 的库存数:{:#?}",
            store.orderable_number, store.inventory
        );

        Ok(store.inventory)
    }
}
