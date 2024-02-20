use crate::config::{Account, Selection};
use cslt::{
    blocking::{client, login},
    client::SelectMode,
};
use std::{
    sync::{
        atomic::{self, AtomicBool},
        mpsc::Receiver,
        Arc, Mutex,
    },
    thread::sleep,
    time::{Duration, Instant},
};

pub fn worker(
    account: &Account,
    config: &Selection,
    select_task: &Receiver<String>,
    selected_courses: Arc<Mutex<Vec<String>>>,
    logined: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let mut first_run = true;
    let mut last_try_login = Instant::now();
    let login_retry_interval = Duration::from_secs(config.login_retry_interval.unwrap_or(30000));

    let client = client::ClientBuilder::new().build()?;

    println!("Worker started");

    loop {
        let details = client.refresh_details()?;
        let is_logined = details.is_logined();

        logined.store(is_logined, atomic::Ordering::Relaxed);

        if !is_logined {
            println!("Not logined");
            if !first_run && last_try_login.elapsed() < login_retry_interval {
                println!("Waiting...");
                sleep(login_retry_interval - last_try_login.elapsed());
            }

            println!("Try login...");
            // try to login, if not logged in
            let login_by_secret = login::LoginBySecret::new(&account.ntustsecret);
            client.login(&login_by_secret)?;

            last_try_login = Instant::now();
        } else {
            {
                // update selected courses

                let course_nos: Vec<_> = details
                    .courses()?
                    .iter()
                    .map(|c| c.course_no.clone())
                    .collect();

                let mut selected_courses = selected_courses.lock().unwrap();
                selected_courses.clear();
                selected_courses.extend(course_nos);
            }

            // print account information
            let name = details.name()?;
            let class = details.class()?;

            println!("={:=<50}=", "Selection Worker Report");
            println!(" report time: {:?}", chrono::offset::Local::now());
            println!(" logined: {is_logined}");
            println!(" name: {name}");
            println!(" class: {class}");

            println!(" courses:");
            details
                .courses()?
                .iter()
                .for_each(move |c| println!("  - {: <10} {}", c.course_no, c.name));

            // tasks

            println!("Waiting new task...");

            let timeout_duration =
                Duration::from_millis(config.session_refresh_interval.unwrap_or(60000));
            let start_time = Instant::now();
            let result = select_task.recv_timeout(timeout_duration);

            match result {
                Ok(course_no) => 'proccess_task: {
                    // println!("Received message: {}", course_no);
                    println!("Received task: {}", course_no);

                    if selected_courses.lock().unwrap().contains(&course_no) {
                        println!("course {} already selected, cancel task", course_no);
                        sleep(timeout_duration - start_time.elapsed());
                        break 'proccess_task;
                    }

                    println!("Selecting {}...", course_no);

                    let mode = match config.mode.as_str() {
                        "pre" => SelectMode::Pre,
                        "started" => SelectMode::Started,
                        "custom" => SelectMode::Custom(
                            config.custom_select_page_url.as_ref().unwrap().as_str(),
                            config.custom_select_api_url.as_ref().unwrap().as_str(),
                        ),
                        _ => panic!("Invalid select mode"),
                    };

                    match client.select_course(mode, &course_no) {
                        Ok(result) => match result.result() {
                            Ok(result_page) => println!(
                                "Select {} operation occurred, result msg: {}",
                                course_no,
                                result_page.result_message().unwrap_or(
                                    "no message (typically indicates success)".to_string()
                                )
                            ),

                            Err(err) => println!("Get {} select result failed: {}", course_no, err),
                        },

                        Err(err) => println!("Select {} failed: {}", course_no, err),
                    }
                }
                Err(_) => {
                    // println!("Timeout occurred");
                }
            }
        }

        first_run = false;
    }
}
