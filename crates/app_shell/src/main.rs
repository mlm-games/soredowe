use crossbeam_channel as chan;
use std::sync::Arc;

use app_ui::{
    root_view,
    state::{Action, Store},
};
use backend_aur::AurBackend;
use backend_pacman::PacmanCli;
use compose_platform::run_desktop_app;
use domain::{Executor, PackageBackend};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (tx_jobs, rx_jobs) = chan::unbounded();
    let (tx_prog, rx_prog) = chan::unbounded();
    let (tx_evt, rx_evt) = chan::unbounded();

    let repo_backend: Arc<dyn PackageBackend> = Arc::new(PacmanCli::new());
    let aur_backend: Arc<dyn PackageBackend> = Arc::new(AurBackend::new());
    Executor::new(
        repo_backend,
        aur_backend,
        tx_prog.clone(),
        tx_evt.clone(),
        rx_jobs,
    )
    .run();

    let store = std::rc::Rc::new(Store::new(tx_jobs));

    run_desktop_app(move |_sched| {
        while let Ok(p) = rx_prog.try_recv() {
            store.dispatch(Action::Progress(p));
        }
        while let Ok(e) = rx_evt.try_recv() {
            store.dispatch(Action::Event(e));
        }
        root_view(store.clone())
    })
}
