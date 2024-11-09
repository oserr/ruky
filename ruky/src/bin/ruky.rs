use log::LevelFilter;
use ruky::random_eng::RandomEng;
use std::sync::Arc;
use tokio::runtime::Builder;
use uzi::conf::Config;
use uzi::eng::EngController;
use uzi::engtx::UziOut;

fn main() {
    println!("Running the ruky random search engine...");
    let mut config = Config::new();
    config.id_name = "Ruky chess engine".into();
    config.id_author = "Omar Serrano".into();
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("UCI in/out thread")
        .build()
        .expect("Unable to build the tokio runtime.");
    let uzi_out = Arc::new(UziOut::from(Arc::new(runtime)));
    let eng = RandomEng::new(uzi_out.clone());
    let mut eng_controller = EngController::create(eng, uzi_out, config);
    if let Err(_) = simple_logging::log_to_file("ruky.log", LevelFilter::max()) {
        eprintln!("Unable to initialize logging.");
    }
    if let Err(_) = eng_controller.run() {
        eprintln!("EngController returned an error.");
    }
}
