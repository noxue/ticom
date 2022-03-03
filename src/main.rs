// #![windows_subsystem = "windows"]

use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
use log::{debug, info};
use ti::account::Account;
use tokio::sync::Mutex;

use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    path::Path,
    sync::Arc,
};

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // 接收到列表才执行
    let products;

    // 产品列表
    if Path::new("./products.txt").exists() {
        let data = read_to_string("./products.txt").unwrap_or("".to_owned());
        products = data
            .split("\n")
            .into_iter()
            .map(|v| v.trim().to_string())
            .collect::<Vec<String>>();
    } else {
        File::create("./products.txt").unwrap();
        println!("请配置产品");
        return;
    }

    // 一直等待，直到接收到邮箱信息
    let mut email_from = "".to_owned();
    let mut email_from_password = "".to_owned();
    let mut email_to = "".to_owned();

    // 邮箱配置信息
    if Path::new("./email.txt").exists() {
        let data = read_to_string("./email.txt").unwrap_or("".to_owned());
        let t = data.split("\n").map(|v| v).collect::<Vec<&str>>();
        if t.len() == 3 {
            email_from = t[0].trim().to_string();
            email_from_password = t[1].trim().to_string();
            email_to = t[2].trim().to_string();
        }
    } else {
        File::create("./email.txt").unwrap();
        println!("请配置邮件信息");
        return;
    }
    debug!("要监控的产品列表:{:#?}", products);

    let account = Account::new().await;

    // let product_name = "OPA1622IDRCR";

    // hashmap记录有库存是否通知
    let notices = Arc::new(Mutex::new(HashMap::new()));

    // 记录当前几个任务在执行
    let mut tasks = vec![];
    // let tasks = Mutex::new(tasks);

    loop {
        for product_name in &products {
            // 忽略空行
            if product_name.is_empty() {
                continue;
            }

            let product_name = product_name.clone();

            let email_from = email_from.clone();
            let email_from_password = email_from_password.clone();
            let email_to = email_to.clone();

            let account = account.clone();
            let notices = notices.clone();

            let v = tokio::spawn(async move {
                println!("正在获取 {} 的库存", product_name);

                let count = match account
                    .get_store_by_product_name(product_name.as_str())
                    .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        println!("获取产品 {} 库存失败，请检查产品名字是否正确", product_name);
                        info!("获取失败:{}", e);
                        0 as usize
                    }
                };

                println!("产品: {}, 库存: {}", product_name, count);

                let mut t = notices.lock().await;

                // 如果对应产品有库存，但是没有记录，就发邮件通知并记录一下
                if count > 0 && t.get(&product_name).is_none() {
                    send_email(
                        email_from.as_str(),
                        email_from_password.as_str(),
                        email_to.as_str(),
                        format!("{} 产品有 {} 个新库存", product_name, count).as_str(),
                        format!("{} 产品有 {} 个新库存", product_name, count).as_str(),
                    );

                    t.insert(product_name.clone(), true);
                }

                // 如果库存为0  就把之前的记录给删除
                if count == 0 && t.get(&product_name).is_some() {
                    t.remove(&product_name);
                }

                info!("库存:{}", count);
            });

            tasks.push(v);

            if tasks.len() == 4 {
                while let Some(task) = tasks.pop() {
                    task.await;
                }
            }

            // loop {
            //     // let tasks = tasks.lock().await;
            //     if tasks > 0 {
            //         tokio::time::sleep(Duration::from_millis(300)).await;
            //     }
            //     break;
            // }
        }
    }
}

/// 发送邮件通知
/// 173126019@qq.com
/// jslmkwghsxdjbgji
fn send_email(from: &str, password: &str, to: &str, title: &str, body: &str) {
    log::debug!("发件箱:{}, 收件箱:{}", from, to);
    let email = EmailBuilder::new()
        .from(from)
        .to(to)
        .subject(title)
        .html(body)
        .build()
        .unwrap();

    let mut mailer = SmtpClient::new_simple("smtp.qq.com")
        .unwrap()
        .credentials(Credentials::new(from.into(), password.into()))
        .transport();

    mailer.send(email.into()).unwrap();
}

#[test]
fn test_mail() {
    let email = EmailBuilder::new()
        .from("513004165@qq.com")
        .to("513004165@qq.com")
        .subject("asdfaf")
        .html("sssssssssss")
        .build()
        .unwrap();
}
