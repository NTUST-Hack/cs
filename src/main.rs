mod config;
// mod query;
// pub mod client;
mod select;

use std::{
    collections::HashSet,
    sync::{
        atomic::{self, AtomicBool, AtomicUsize},
        mpsc::sync_channel,
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::Duration,
};

use clap::{Arg, ArgMatches, Command};
use config::{load_config, Course};

fn get_matches() -> ArgMatches {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("CONFIG_PATH")
                .help("Sets the path to the configuration file")
                .required(true),
        )
        .get_matches()
}

fn main() -> anyhow::Result<()> {
    let matches = get_matches();
    let config_path = matches.get_one::<String>("config").unwrap();

    let config = load_config(&config_path)?;

    // let mut target_courses = match config.target {
    //     Some(target) => target.courses,
    //     None => vec![],
    // };

    let logined = Arc::new(AtomicBool::new(false));
    let logined_clone = Arc::clone(&logined);
    let (select_tx, select_rx) = sync_channel::<String>(1);
    let select_tx_mutex = Arc::new(Mutex::new(select_tx));

    let selected_courses: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let selected_courses_clone: Arc<Mutex<Vec<String>>> = Arc::clone(&selected_courses);

    let _select_worker = thread::spawn(move || {
        select::worker(
            &config.account,
            &config.selection,
            &select_rx,
            Arc::clone(&selected_courses),
            Arc::clone(&logined),
        )
    });

    let (query_tx, query_rx) = sync_channel(config.query.threads.try_into().unwrap());
    let query_rx_mutex = Arc::new(Mutex::new(query_rx));

    let times = Arc::new(AtomicUsize::new(0));
    let times_clone = times.clone();

    for _i in 0..config.query.threads {
        let q = q::blocking::ClientBuilder::new().build().unwrap();
        let select_tx = Arc::clone(&select_tx_mutex);
        let query_rx = Arc::clone(&query_rx_mutex);
        let times_clone = Arc::clone(&times_clone);

        let language = match &config.query.language {
            Some(lang) => lang.clone(),
            None => "zh".to_string(),
        };

        let semester = config.query.semester.clone();

        thread::spawn(move || loop {
            let course_no = String::from(query_rx.lock().unwrap().recv().unwrap());

            match q.query(semester.as_str(), course_no.as_str(), language.as_str()) {
                Ok(result) => {
                    times_clone.fetch_add(1, atomic::Ordering::Relaxed);

                    // println!(
                    //     "[thread {i}] {: <10} {:　<10} {}/{}",
                    //     result.course_no,
                    //     result.course_name,
                    //     result.choose_student,
                    //     result.restrict2
                    // );

                    let has_slot = result.choose_student < result.restrict2;
                    if has_slot {
                        // send select task to select worker thread
                        select_tx.lock().unwrap().send(course_no).unwrap();
                    }
                }
                Err(err) => println!("Query course failed: {}", err),
            };
            // println!("[thread {i}] course_no: {course_no}");
        });
    }

    let target_courses: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let target_courses_clone = Arc::clone(&target_courses);

    thread::spawn(move || {
        let times_clone = Arc::clone(&times_clone);

        let mut last_times = 0;
        let update_interval = Duration::from_secs(15);
        loop {
            let times = times_clone.load(atomic::Ordering::Relaxed);
            let add_times = times - last_times;
            let times_per_sec = add_times / update_interval.as_secs() as usize;
            last_times = times;

            println!("={:=<50}=", "Query Worker Report");
            println!(" report time: {:?}", chrono::offset::Local::now());
            println!(" total times: {times}");
            println!(" speed: {times_per_sec}/s");
            println!(" querying course:");
            target_courses_clone
                .lock()
                .unwrap()
                .iter()
                .for_each(|c| println!("  - {c}"));

            sleep(update_interval);
        }
    });

    loop {
        if !logined_clone.load(atomic::Ordering::Relaxed) {
            sleep(Duration::from_secs(1));
            continue;
        }

        let new_target_course = find_target_courses(
            &config.target.as_ref().expect("no target courses").courses,
            &selected_courses_clone.lock().unwrap(),
        );
        *target_courses.lock().unwrap() = new_target_course.clone();

        // println!("target courses: {:#?}", &target_courses);

        for c in new_target_course {
            query_tx.send(c).unwrap();
        }
    }

    // select_worker.join().unwrap().unwrap();
}

fn find_target_courses(vec1: &Vec<Course>, vec2: &Vec<String>) -> HashSet<String> {
    vec1.iter()
        .filter(|c| c.enabled)
        .filter(|c| !vec2.contains(&c.course_no))
        .map(|c| String::from(&c.course_no))
        .collect()
}
