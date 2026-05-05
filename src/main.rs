use std::path::Path;

use clap::Parser;
use log::info;
use tomo::cli::Cli;
use tomo::config::Alarm;
use tomo::config::Config;
use tomo::error::AppError;
use tomo::log::setup_logging;
use tomo::model::Pomodoro;
use tomo::repo::Repos;
use tomo::service::DesktopNotifyService;
use tomo::service::NotifyService;
use tomo::service::SoundService;
use tomo::service::alarm::AlarmService;
use tomo::ui::Runner;

type Repo = Box<dyn Repos>;
type Sound = Box<dyn SoundService<SoundType = Alarm>>;
type Notify = Box<dyn NotifyService>;
type View = Box<dyn Runner>;

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    let conf = Config::load()?;
    setup_logging(&conf.logs_path)?;
    color_eyre::install().unwrap();
    info!("initializing {} v{}", tomo::APP_NAME, tomo::APP_VERSION);

    let repo = repo(&conf.db_path);
    let sound = alarm();
    let notify = notify();
    let pomo = pomodoro(&cli, &conf);

    repo.session().close_all_sessions().unwrap();

    let mut runner = view(conf, repo, sound, notify, pomo);
    info!("starting view");
    runner.run().unwrap();
    Ok(())
}

fn view(conf: Config, repo: Repo, sound: Sound, notify: Notify, pomo: Pomodoro) -> View {
    use tomo::ui::core::AppCore;
    use tomo::ui::tui::TuiEffectHandler;
    use tomo::ui::tui::TuiRunner;

    let effect = TuiEffectHandler::new(sound, notify, repo);
    let core = AppCore::new(pomo, conf, effect);

    Box::new(TuiRunner::new(core).unwrap())
}

fn repo(path: &Path) -> Repo {
    use tomo::repo::sqlite::SqliteDb;
    use tomo::repo::sqlite::SqliteRepos;

    let url = format!("sqlite://{}", path.display());
    let db = SqliteDb::new(url).unwrap();
    let repo = SqliteRepos::new(db);
    Box::new(repo)
}

fn pomodoro(cli: &Cli, conf: &Config) -> Pomodoro {
    let timer = conf.pomodoro.timer.clone();

    let focus = cli.focus.unwrap_or(timer.focus);
    let long_break = cli.long_break.unwrap_or(timer.long);
    let short_break = cli.short_break.unwrap_or(timer.short);
    let long_interval = cli.long_interval.unwrap_or(timer.long_interval);

    Pomodoro::new(focus, long_break, short_break, long_interval)
}

fn alarm() -> Sound {
    let alarm = AlarmService::new();
    Box::new(alarm)
}

fn notify() -> Notify {
    Box::new(DesktopNotifyService)
}
