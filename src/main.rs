#![windows_subsystem = "windows"]

use eframe::{
    egui::{self, FontDefinitions, FontFamily, Visuals},
    epi::{self, IconData},
};
use log::{debug, info};
use thirtyfour::support::block_on;
use ti::account::Account;
use tokio::sync::Mutex;

use std::{
    borrow::Borrow,
    collections::HashMap,
    fs::{self, read_to_string},
    ops::AddAssign,
    path::Path,
    rc::Rc,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

#[derive(Default)]
struct TiApp {
    username: String,
    password: String,
    product_list: String,
    email_from: String,
    email_from_password: String,
    email_to: String,
    log_text: String,
    // 接受执行结果
    reciver_product_count_log: Option<Receiver<String>>,
    // 发送产品列表
    sender_product_name: Option<Sender<String>>,
    // 发送用户信息
    sender_user: Option<Sender<String>>,
    // 发送邮箱信息
    sender_email: Option<Sender<String>>,
}

impl epi::App for TiApp {
    fn name(&self) -> &str {
        "芯片库存监控软件"
    }

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // 如果配置信息存在，就读取
        if Path::new("./user.txt").exists() {
            let data = read_to_string("./user.txt").unwrap_or("".to_owned());
            let t = data.split("\n").map(|v| v).collect::<Vec<&str>>();
            if t.len() == 2 {
                self.username = t[0].to_string();
                self.password = t[1].to_string();
            }
        }

        // 邮箱配置信息
        if Path::new("./email.txt").exists() {
            let data = read_to_string("./email.txt").unwrap_or("".to_owned());
            let t = data.split("\n").map(|v| v).collect::<Vec<&str>>();
            if t.len() == 3 {
                self.email_from = t[0].to_string();
                self.email_from_password = t[1].to_string();
                self.email_to = t[2].to_string();
            }
        }

        // 产品列表
        if Path::new("./products.txt").exists() {
            let data = read_to_string("./products.txt").unwrap_or("".to_owned());
            self.product_list = data;
        }

        thread::spawn(move || {
            let mut cmd = Command::new(r#"./chromedriver.exe"#);
            let output = cmd
                .creation_flags(0x08000000)
                .stdout(Stdio::piped())
                .output()
                .expect("exec error!");

            println!("{}", String::from_utf8_lossy(&output.stdout));
        });

        let repaint = frame.repaint_signal();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(100));
            repaint.request_repaint();
        });
        // 默认黑色主题
        // ctx.set_visuals(egui::Visuals::dark());

        //Custom font install
        // # use epaint::text::*;
        // 1. Create a `FontDefinitions` object.
        let mut font = FontDefinitions::default();
        // Install my own font (maybe supporting non-latin characters):
        // 2. register the font content with a name.
        font.font_data.insert(
            "cn_font".to_owned(),
            std::borrow::Cow::Borrowed(include_bytes!("../fonts/方正标雅宋简体.TTF")),
        );
        //font.font_data.insert("mPlus".to_string(), Cow::from(&mPlus_font[..]));
        // 3. Set two font families to use the font, font's name must have been
        // Put new font first (highest priority)registered in `font_data`.
        font.fonts_for_family
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "cn_font".to_owned());
        font.fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "cn_font".to_owned());
        // 4. Configure context with modified `FontDefinitions`.
        ctx.set_fonts(font);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self {
            username,
            password,
            product_list,
            email_from,
            email_from_password,
            email_to,
            log_text,
            reciver_product_count_log,
            sender_product_name,
            sender_user,
            sender_email,
        } = self;

        if let Ok(data) = reciver_product_count_log.as_ref().unwrap().try_recv() {
            let t: Vec<String> = log_text
                .split("\n")
                .into_iter()
                .map(|v| {
                    if v.is_empty() {
                        "".to_owned()
                    } else {
                        v.to_string() + "\n"
                    }
                })
                .collect();
            // t.reverse();
            *log_text = "".to_owned();

            let left = if t.len() as i32 - 20 < 0 {
                0
            } else {
                t.len() - 20
            };
            let right = if t.len() as i32 - 1 < 0 {
                0
            } else {
                t.len() - 1
            };

            for i in left..right {
                *log_text += match t.get(i) {
                    Some(v) => v,
                    None => {
                        break;
                    }
                };
            }

            *log_text += format!("{}\n", data).as_str();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    ui.indent("left", |ui| {
                        ui.set_max_height(580.0);
                        ui.set_max_width(300.0);
                        ui.set_min_width(300.0);
                        ui.heading("监控的产品列表");
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if ui.text_edit_multiline(product_list).changed() {
                                println!("xx");
                            }
                        });
                    });
                });

                ui.vertical(|ui| {
                    ui.set_min_height(580.0);
                    ui.indent("right", |ui| {
                        ui.scope(|ui| {
                            ui.heading("ti.com账号密码");
                            ui.horizontal(|ui| {
                                ui.label("账号:");
                                ui.text_edit_singleline(username);
                            });
                            ui.horizontal(|ui| {
                                ui.label("密码:");
                                ui.text_edit_singleline(password);
                            });
                        });

                        ui.separator();
                        ui.scope(|ui| {
                            ui.heading("邮件通知配置");
                            ui.horizontal(|ui| {
                                ui.label("发件箱账号:");
                                ui.text_edit_singleline(email_from);
                            });
                            ui.horizontal(|ui| {
                                ui.label("发件箱密码:");
                                ui.text_edit_singleline(email_from_password);
                            });
                            ui.horizontal(|ui| {
                                ui.label("收件箱账号:");
                                ui.text_edit_singleline(email_to);
                            });
                        });

                        ui.add_space(10.0);
                        ui.scope(|ui| {
                            if ui.button("开始监控").clicked() {
                                // 发送产品列表
                                sender_product_name
                                    .as_ref()
                                    .unwrap()
                                    .send(product_list.to_string())
                                    .unwrap();

                                // 发送用户信息
                                sender_user
                                    .as_ref()
                                    .unwrap()
                                    .send(format!("{}\n{}", username, password))
                                    .unwrap();

                                // 发送邮箱配置
                                sender_email
                                    .as_ref()
                                    .unwrap()
                                    .send(format!(
                                        "{}\n{}\n{}",
                                        email_from, email_from_password, email_to
                                    ))
                                    .unwrap();
                            }
                        });

                        ui.separator();
                        ui.add_space(20.0);
                        ui.heading("运行记录");
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.set_min_width(300.0);
                            if ui
                                .colored_label(egui::Color32::from_rgb(0, 100, 0), log_text)
                                .changed()
                            {}
                        });
                    });
                });
            });
        });

        // Resize the native window to be just the size we need it to be:
        // frame.set_window_size(ctx.used_size());

        frame.set_window_size(egui::vec2(800.0, 600.0));
    }

    fn warm_up_enabled(&self) -> bool {
        false
    }

    fn save(&mut self, _storage: &mut dyn epi::Storage) {}

    fn on_exit(&mut self) {
        // 保存用户信息到配置文件
        fs::write(
            "./user.txt",
            format!("{}\n{}", self.username, self.password),
        )
        .unwrap();

        // 保存邮箱配置信息
        fs::write(
            "./email.txt",
            format!(
                "{}\n{}\n{}",
                self.email_from, self.email_from_password, self.email_to
            ),
        )
        .unwrap();

        // 保存产品列表
        fs::write("./products.txt", format!("{}", self.product_list)).unwrap();

        //taskkill /f /t /im chromedriver.exe
        let mut cmd = Command::new("taskkill");
        let output = cmd
            .creation_flags(0x08000000)
            .arg("/f")
            .arg("/t")
            .arg("/im")
            .arg("chromedriver.exe")
            .stdout(Stdio::piped())
            .output()
            .expect("exec error!");

        println!("{}", String::from_utf8_lossy(&output.stdout));
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn max_size_points(&self) -> egui::Vec2 {
        // Some browsers get slow with huge WebGL canvases, so we limit the size:
        egui::Vec2::new(1024.0, 2048.0)
    }

    fn clear_color(&self) -> egui::Rgba {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).into()
    }

    fn persist_native_window(&self) -> bool {
        true
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // let options = eframe::NativeOptions::default();

    let image_data = include_bytes!("../2.jpg");
    use image::GenericImageView;
    let image = image::load_from_memory(image_data).expect("Failed to load image");
    let image_buffer = image.to_rgba8();

    let options = eframe::NativeOptions {
        resizable: false,
        decorated: true, // 是否显示 标题栏 边框等

        // 软件左上角图标
        icon_data: Some(IconData {
            rgba: image_buffer.to_vec(),
            width: image.width(),
            height: image.height(),
        }),

        ..Default::default()
    };

    // 执行日志通道
    let (sender_product_count_log, receiver_product_count_log) =
        std::sync::mpsc::channel::<String>();

    // 产品名称列表通道
    let (sender_product_name, receiver_product_name) = std::sync::mpsc::channel::<String>();

    // 用户名列表通道
    let (sender_user, receiver_user) = std::sync::mpsc::channel::<String>();

    // 邮箱信息通道
    let (sender_email, receiver_email) = std::sync::mpsc::channel::<String>();

    let mut app = TiApp::default();
    app.reciver_product_count_log = Some(receiver_product_count_log);
    app.sender_product_name = Some(sender_product_name);
    app.sender_user = Some(sender_user);
    app.sender_email = Some(sender_email);

    thread::spawn(move || {
        use tokio::runtime::Runtime;

        let runtime = Runtime::new().unwrap();

        let sender_ui = sender_product_count_log.clone();

        sender_ui
            .send("在左侧输入产品名称，一行一个，然后点击开始监控按钮".to_string())
            .unwrap();

        runtime.block_on(async move {
            // 接收到列表才执行
            let products;

            // 一直等待，直到接收到产品列表
            loop {
                if let Ok(v) = receiver_product_name.try_recv() {
                    products = v
                        .split("\n")
                        .into_iter()
                        .map(|v| v.trim().to_string())
                        .collect::<Vec<String>>();
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }

            // 一直等待，直到接收到用户信息
            let username;
            let password;
            loop {
                if let Ok(v) = receiver_user.try_recv() {
                    let mut t = v.split("\n");
                    username = t.next().unwrap().to_string();
                    password = t.next().unwrap().to_string();
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }

            // 一直等待，直到接收到邮箱信息
            let email_from;
            let email_from_password;
            let email_to;
            loop {
                if let Ok(v) = receiver_email.try_recv() {
                    let mut t = v.split("\n");
                    email_from = t.next().unwrap().to_string();
                    email_from_password = t.next().unwrap().to_string();
                    email_to = t.next().unwrap().to_string();
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
            debug!("要监控的产品列表:{:#?}", products);

            let account = Account::new(username.as_str(), password.as_str()).await;

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

                    let sender_ui = sender_ui.clone();

                    let email_from = email_from.clone();
                    let email_from_password = email_from_password.clone();
                    let email_to = email_to.clone();

                    let account = account.clone();
                    let notices = notices.clone();

                    let v = tokio::spawn(async move {
                        sender_ui
                            .send(format!("正在获取 {} 的库存", product_name))
                            .unwrap();

                        let count = match account
                            .get_store_by_product_name(product_name.as_str())
                            .await
                        {
                            Ok(v) => v,
                            Err(e) => {
                                sender_ui
                                    .send(format!(
                                        "获取产品 {} 库存失败，请检查产品名字是否正确",
                                        product_name
                                    ))
                                    .unwrap();
                                info!("获取失败:{}", e);
                                0 as usize
                            }
                        };

                        sender_ui
                            .send(format!("产品: {}, 库存: {}", product_name, count))
                            .unwrap();

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

                    if tasks.len() == 6 {
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
        });
    });
    eframe::run_native(Box::new(app), options);
}

/// 发送邮件通知
/// 173126019@qq.com
/// jslmkwghsxdjbgji
fn send_email(from: &str, password: &str, to: &str, title: &str, body: &str) {
    use lettre::smtp::authentication::Credentials;
    use lettre::{SmtpClient, Transport};
    use lettre_email::EmailBuilder;

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
