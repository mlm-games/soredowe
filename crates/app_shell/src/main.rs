use crossbeam_channel as chan;
#[cfg(target_os = "linux")]
use log::error;
use repose_core::Modifier;
use std::{rc::Rc, sync::Arc};

use app_ui::{
    root_view,
    state::{Action, Store},
};
use domain::config::Settings;
use domain::{Executor, PackageBackend};
use repose_platform::{AppConfig, run_desktop_app_with_config};
use repose_ui::overlay::{OverlayHandle, SnackbarController};

#[cfg(all(feature = "backend-alpm", target_os = "linux"))]
use backend_alpm::AlpmBackend;
#[cfg(all(feature = "backend-appimage", target_os = "linux"))]
use backend_appimage::AppImageBackend;
#[cfg(all(feature = "backend-aur", target_os = "linux"))]
use backend_aur::AurBackend;
#[cfg(all(feature = "backend-flatpak", target_os = "linux"))]
use backend_flatpak::FlatpakBackend;
#[cfg(all(feature = "backend-packagekit", target_os = "linux"))]
use backend_packagekit::PackageKitBackend;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let settings = Settings::load();

    let (tx_jobs, rx_jobs) = chan::unbounded();
    let (tx_prog, rx_prog) = chan::unbounded();
    let (tx_evt, rx_evt) = chan::unbounded();

    #[allow(unused_mut)]
    let mut backends: Vec<(&'static str, Arc<dyn PackageBackend>)> = Vec::new();

    #[cfg(all(feature = "backend-alpm", target_os = "linux"))]
    if settings.enable_repo {
        let repo: Arc<dyn PackageBackend> = Arc::new(AlpmBackend::new());
        backends.push((repo.name(), repo));
    }

    #[cfg(all(feature = "backend-packagekit", target_os = "linux"))]
    match PackageKitBackend::new() {
        Ok(pk) => {
            let pk: Arc<dyn PackageBackend> = Arc::new(pk);
            backends.push((pk.name(), pk));
        }
        Err(e) => error!("PackageKit backend unavailable: {e}"),
    }

    #[cfg(all(feature = "backend-aur", target_os = "linux"))]
    if settings.enable_aur {
        let aur: Arc<dyn PackageBackend> = Arc::new(AurBackend::new());
        backends.push((aur.name(), aur));
    }

    #[cfg(all(feature = "backend-flatpak", target_os = "linux"))]
    if settings.enable_flatpak {
        match FlatpakBackend::new(true) {
            Ok(fp) => {
                let flatpak: Arc<dyn PackageBackend> = Arc::new(fp);
                backends.push((flatpak.name(), flatpak));
            }
            Err(e) => error!("Flatpak backend unavailable: {e}"),
        }
    }

    #[cfg(all(feature = "backend-appimage", target_os = "linux"))]
    if settings.enable_appimage {
        let appimage: Arc<dyn PackageBackend> = Arc::new(AppImageBackend::new());
        backends.push((appimage.name(), appimage));
    }

    Executor::new(backends, tx_prog.clone(), tx_evt.clone(), rx_jobs).run();

    let overlay = OverlayHandle::new();
    let snackbar = SnackbarController::new(overlay.clone());

    let store = Rc::new(Store::new(tx_jobs, Some(snackbar), settings));
    run_desktop_app_with_config(
        move |_sched, _ctx| {
            while let Ok(p) = rx_prog.try_recv() {
                store.dispatch(Action::Progress(p));
            }
            while let Ok(e) = rx_evt.try_recv() {
                store.dispatch(Action::Event(e));
            }
            overlay.host(Modifier::new().fill_max_size(), root_view(store.clone()))
        },
        AppConfig {
            window_title: "Soredowe".into(),
            ..AppConfig::default()
        },
    )
}
