use crossbeam_channel as chan;
use parking_lot::Mutex;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    Repo,
    Aur,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub name: String,
    pub source: Source,
}

#[derive(Clone, Debug)]
pub struct PackageSummary {
    pub id: PackageId,
    pub version: String,
    pub description: String,
    pub installed: bool,
    pub popular: Option<u32>,
    pub last_updated: Option<SystemTime>,
}

#[derive(Clone, Debug)]
pub struct PackageDetails {
    pub summary: PackageSummary,
    pub depends: Vec<String>,
    pub opt_depends: Vec<String>,
    pub homepage: Option<String>,
    pub maintainer: Option<String>,
    pub size_install: Option<u64>,
    pub size_download: Option<u64>,
}

#[derive(Clone, Debug)]
pub enum Stage {
    Queued,
    Refreshing,
    Searching,
    Resolving,
    Downloading,
    Building,
    Installing,
    Removing,
    Verifying,
    Cleaning,
    Finished,
    Failed,
}

#[derive(Clone, Debug)]
pub struct Progress {
    pub job_id: u64,
    pub stage: Stage,
    pub percent: Option<f32>,
    pub bytes: Option<(u64, u64)>,
    pub log: Option<String>,
    pub warning: bool,
}

#[derive(Clone, Debug)]
pub enum Event {
    SearchResults {
        query: String,
        items: Vec<PackageSummary>,
    },
    Details {
        item: PackageDetails,
    },
    Upgrades {
        items: Vec<PackageSummary>,
    },
    /// Sent when the system package state likely changed (install/remove/upgrade).
    SystemChanged,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("network: {0}")]
    Network(String),
    #[error("alpm: {0}")]
    Alpm(String),
    #[error("aur: {0}")]
    Aur(String),
    #[error("privilege: {0}")]
    Priv(String),
    #[error("cancelled")]
    Cancelled,
    #[error("internal: {0}")]
    Internal(String),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct CancelToken(Arc<AtomicBool>);
impl CancelToken {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst)
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
pub type ProgressSink = chan::Sender<Progress>;

pub trait PackageBackend: Send + Sync {
    fn refresh(&self, sink: &ProgressSink, cancel: &CancelToken) -> Result<()>;
    fn search(
        &self,
        q: &str,
        sink: &ProgressSink,
        cancel: &CancelToken,
    ) -> Result<Vec<PackageSummary>>;
    fn details(
        &self,
        id: &PackageId,
        sink: &ProgressSink,
        cancel: &CancelToken,
    ) -> Result<PackageDetails>;
    fn install(&self, id: &PackageId, sink: &ProgressSink, cancel: &CancelToken) -> Result<()>;
    fn remove(&self, id: &PackageId, sink: &ProgressSink, cancel: &CancelToken) -> Result<()>;
    fn upgrades(&self, sink: &ProgressSink, cancel: &CancelToken) -> Result<Vec<PackageSummary>>;
    fn upgrade(&self, id: &PackageId, sink: &ProgressSink, cancel: &CancelToken) -> Result<()>;
    fn upgrade_all(&self, sink: &ProgressSink, cancel: &CancelToken) -> Result<()>;
}

#[derive(Clone, Copy, Debug)]
pub enum JobKind {
    Refresh,
    Search,
    Details,
    Install,
    Remove,
    Upgrades,
    Upgrade,
    UpgradeAll,
}

#[derive(Clone, Debug)]
pub enum JobPayload {
    None,
    Query(String),
    Package(PackageId),
}

#[derive(Clone, Debug)]
pub struct Job {
    pub id: u64,
    pub kind: JobKind,
    pub payload: JobPayload,
    pub created_at: SystemTime,
    pub cancel: CancelToken,
}

static TXN_MUTEX: Mutex<()> = Mutex::new(());

pub struct Executor {
    repo: Arc<dyn PackageBackend>,
    aur: Arc<dyn PackageBackend>,
    tx_prog: chan::Sender<Progress>,
    tx_evt: chan::Sender<Event>,
    rx_jobs: chan::Receiver<Job>,
}

impl Executor {
    pub fn new(
        repo: Arc<dyn PackageBackend>,
        aur: Arc<dyn PackageBackend>,
        tx_prog: chan::Sender<Progress>,
        tx_evt: chan::Sender<Event>,
        rx_jobs: chan::Receiver<Job>,
    ) -> Self {
        Self {
            repo,
            aur,
            tx_prog,
            tx_evt,
            rx_jobs,
        }
    }

    pub fn run(self) {
        std::thread::spawn(move || {
            while let Ok(job) = self.rx_jobs.recv() {
                let sink = self.tx_prog.clone();
                let tx_evt = self.tx_evt.clone();
                let cancel = job.cancel.clone();
                let send = |p: Progress| {
                    let _ = sink.send(p);
                };

                let repo = &self.repo;
                let aur = &self.aur;
                let pick = |payload: &JobPayload| -> &dyn PackageBackend {
                    match payload {
                        JobPayload::Package(id) if id.source == Source::Aur => &*self.aur,
                        _ => &*self.repo,
                    }
                };

                send(Progress {
                    job_id: job.id,
                    stage: Stage::Queued,
                    percent: None,
                    bytes: None,
                    log: None,
                    warning: false,
                });

                let run_job = || -> Result<()> {
                    match job.kind {
                        JobKind::Refresh => pick(&job.payload).refresh(&sink, &cancel),
                        JobKind::Search => {
                            let q = if let JobPayload::Query(q) = &job.payload {
                                q.trim().to_string()
                            } else {
                                String::new()
                            };
                            if q.len() < 2 {
                                let _ = tx_evt.send(Event::SearchResults {
                                    query: q,
                                    items: vec![],
                                });
                                return Ok(());
                            }

                            let mut any_ok = false;
                            let mut items: Vec<PackageSummary> = Vec::new();

                            // Repo
                            match repo.search(&q, &sink, &cancel) {
                                Ok(mut v) => {
                                    items.append(&mut v);
                                    any_ok = true;
                                }
                                Err(e) => {
                                    let _ = sink.send(Progress {
                                        job_id: job.id,
                                        stage: Stage::Searching,
                                        percent: None,
                                        bytes: None,
                                        log: Some(format!("repo search failed: {e}")),
                                        warning: true,
                                    });
                                }
                            }

                            // AUR
                            match aur.search(&q, &sink, &cancel) {
                                Ok(mut v) => {
                                    items.append(&mut v);
                                    any_ok = true;
                                }
                                Err(e) => {
                                    let _ = sink.send(Progress {
                                        job_id: job.id,
                                        stage: Stage::Searching,
                                        percent: None,
                                        bytes: None,
                                        log: Some(format!("AUR search failed: {e}")),
                                        warning: true,
                                    });
                                }
                            }

                            // If both failed, bubble a failure to the final Progress; otherwise continue.
                            if !any_ok {
                                return Err(Error::Alpm("all backends failed".into()));
                            }

                            items.sort_by(|a, b| a.id.name.cmp(&b.id.name));
                            tx_evt
                                .send(Event::SearchResults { query: q, items })
                                .map_err(|e| Error::Internal(e.to_string()))?;
                            Ok(())
                        }
                        JobKind::Details => {
                            if let JobPayload::Package(id) = &job.payload {
                                let det = pick(&job.payload).details(id, &sink, &cancel)?;
                                tx_evt
                                    .send(Event::Details { item: det })
                                    .map_err(|e| Error::Internal(e.to_string()))?;
                            }
                            Ok(())
                        }
                        JobKind::Install => {
                            let _g = TXN_MUTEX.lock();
                            if let JobPayload::Package(id) = &job.payload {
                                pick(&job.payload).install(id, &sink, &cancel)
                            } else {
                                Ok(())
                            }
                        }
                        JobKind::Remove => {
                            let _g = TXN_MUTEX.lock();
                            if let JobPayload::Package(id) = &job.payload {
                                pick(&job.payload).remove(id, &sink, &cancel)
                            } else {
                                Ok(())
                            }
                        }
                        JobKind::Upgrades => {
                            // Collect from both repo and AUR, but don’t fail the whole job
                            let mut items: Vec<PackageSummary> = Vec::new();
                            match repo.upgrades(&sink, &cancel) {
                                Ok(mut v) => items.append(&mut v),
                                Err(e) => {
                                    let _ = sink.send(Progress {
                                        job_id: job.id,
                                        stage: Stage::Verifying,
                                        percent: None,
                                        bytes: None,
                                        log: Some(format!("repo upgrades failed: {e}")),
                                        warning: true,
                                    });
                                }
                            }
                            match aur.upgrades(&sink, &cancel) {
                                Ok(mut v) => items.append(&mut v),
                                Err(e) => {
                                    let _ = sink.send(Progress {
                                        job_id: job.id,
                                        stage: Stage::Verifying,
                                        percent: None,
                                        bytes: None,
                                        log: Some(format!("AUR upgrades failed: {e}")),
                                        warning: true,
                                    });
                                }
                            }
                            // Sort A–Z for stability; UI can re-sort
                            items.sort_by(|a, b| a.id.name.cmp(&b.id.name));
                            tx_evt
                                .send(Event::Upgrades { items })
                                .map_err(|e| Error::Internal(e.to_string()))?;
                            Ok(())
                        }
                        JobKind::Upgrade => {
                            let _g = TXN_MUTEX.lock();
                            if let JobPayload::Package(id) = &job.payload {
                                pick(&job.payload).upgrade(id, &sink, &cancel)
                            } else {
                                Ok(())
                            }
                        }
                        JobKind::UpgradeAll => {
                            let _g = TXN_MUTEX.lock();
                            // Minimal: perform repo full system upgrade; AUR can be expanded later.
                            repo.upgrade_all(&sink, &cancel)?;
                            // If you want AUR mass-upgrade later, we can iterate aur.upgrades() and call aur.upgrade(..).
                            Ok(())
                        }
                    }
                };

                let res = run_job();
                if res.is_ok() {
                    match job.kind {
                        JobKind::Install
                        | JobKind::Remove
                        | JobKind::Upgrade
                        | JobKind::UpgradeAll => {
                            let _ = tx_evt.send(Event::SystemChanged);
                        }
                        _ => {}
                    }
                }
                send(Progress {
                    job_id: job.id,
                    stage: if res.is_ok() {
                        Stage::Finished
                    } else {
                        Stage::Failed
                    },
                    percent: Some(1.0),
                    bytes: None,
                    log: res.as_ref().err().map(|e| e.to_string()),
                    warning: res.is_err(),
                });
            }
        });
    }
}
