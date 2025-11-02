use domain::*;
use serde::Deserialize;
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Deserialize)]
struct AurResponse<T> {
    results: Vec<T>,
}

#[derive(Deserialize)]
struct AurPkg {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Version")]
    version: String,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "NumVotes")]
    votes: Option<u32>,
    #[serde(rename = "Maintainer")]
    maintainer: Option<String>,
    #[serde(rename = "LastModified")]
    last_modified: Option<u64>,
}

pub struct AurBackend;
impl AurBackend {
    pub fn new() -> Self {
        Self
    }
}

fn ts(opt: Option<u64>) -> Option<SystemTime> {
    opt.map(|t| UNIX_EPOCH + std::time::Duration::from_secs(t))
}

fn parse_srcinfo_deps(srcinfo: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in srcinfo.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("depends = ") {
            out.push(strip_ver(v));
        } else if let Some(v) = line.strip_prefix("makedepends = ") {
            out.push(strip_ver(v));
        }
    }
    out.sort();
    out.dedup();
    out
}

fn strip_ver(s: &str) -> String {
    s.split(|c| c == '<' || c == '>' || c == '=')
        .next()
        .unwrap_or(s)
        .trim()
        .to_string()
}

fn find_built_pkg(dir: &PathBuf) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some("zst"))
}

fn validate_pkg_path(p: &PathBuf) -> bool {
    p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("zst")
}

impl PackageBackend for AurBackend {
    fn refresh(&self, _sink: &ProgressSink, _cancel: &CancelToken) -> Result<()> {
        Ok(())
    }

    fn search(
        &self,
        q: &str,
        sink: &ProgressSink,
        _cancel: &CancelToken,
    ) -> Result<Vec<PackageSummary>> {
        let q = q.trim();
        if q.len() < 2 {
            sink.send(Progress {
                job_id: 0,
                stage: Stage::Searching,
                percent: None,
                bytes: None,
                log: Some("AUR: query too short (<2), ignoring".into()),
                warning: true,
            })
            .ok();
            return Ok(vec![]);
        }
        sink.send(Progress {
            job_id: 0,
            stage: Stage::Searching,
            percent: None,
            bytes: None,
            log: Some(format!("AUR search: {q}")),
            warning: false,
        })
        .ok();

        let url = format!(
            "https://aur.archlinux.org/rpc/?v=5&type=search&arg={}",
            urlencoding::encode(q)
        );
        let mut resp = ureq::get(&url)
            .call()
            .map_err(|e| Error::Network(e.to_string()))?;
        let resp: AurResponse<AurPkg> = resp
            .body_mut()
            .read_json()
            .map_err(|e| Error::Network(e.to_string()))?;
        Ok(resp
            .results
            .into_iter()
            .map(|p| PackageSummary {
                id: PackageId {
                    name: p.name,
                    source: Source::Aur,
                },
                version: p.version,
                description: p.description.unwrap_or_default(),
                installed: false,
                popular: p.votes,
                last_updated: ts(p.last_modified),
            })
            .collect())
    }

    fn details(
        &self,
        id: &PackageId,
        _sink: &ProgressSink,
        _cancel: &CancelToken,
    ) -> Result<PackageDetails> {
        let url = format!(
            "https://aur.archlinux.org/rpc/?v=5&type=info&arg[]={}",
            urlencoding::encode(&id.name)
        );
        let mut resp = ureq::get(&url)
            .call()
            .map_err(|e| Error::Network(e.to_string()))?;
        let resp: AurResponse<AurPkg> = resp
            .body_mut()
            .read_json()
            .map_err(|e| Error::Network(e.to_string()))?;
        let p = resp
            .results
            .into_iter()
            .next()
            .ok_or_else(|| Error::Aur("not found".into()))?;
        let summary = PackageSummary {
            id: PackageId {
                name: p.name,
                source: Source::Aur,
            },
            version: p.version,
            description: p.description.unwrap_or_default(),
            installed: false,
            popular: p.votes,
            last_updated: ts(p.last_modified),
        };
        Ok(PackageDetails {
            summary,
            depends: vec![],
            opt_depends: vec![],
            homepage: None,
            maintainer: p.maintainer,
            size_install: None,
            size_download: None,
        })
    }

    fn install(&self, id: &PackageId, sink: &ProgressSink, _cancel: &CancelToken) -> Result<()> {
        sink.send(Progress {
            job_id: 0,
            stage: Stage::Building,
            percent: None,
            bytes: None,
            log: Some(format!("building {}", id.name)),
            warning: false,
        })
        .ok();

        let work = tempfile::tempdir().map_err(|e| Error::Internal(e.to_string()))?;
        let dir = work.path().join(&id.name);

        // clone
        let status = Command::new("git")
            .args([
                "clone",
                &format!("https://aur.archlinux.org/{}.git", id.name),
                dir.to_str().unwrap(),
            ])
            .status()
            .map_err(|e| Error::Internal(e.to_string()))?;
        if !status.success() {
            return Err(Error::Aur("git clone failed".into()));
        }

        // Generate .SRCINFO without shell redirection
        let out = Command::new("makepkg")
            .arg("--printsrcinfo")
            .current_dir(&dir)
            .output()
            .map_err(|e| Error::Internal(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Aur("printsrcinfo failed".into()));
        }
        let mut f =
            fs::File::create(dir.join(".SRCINFO")).map_err(|e| Error::Internal(e.to_string()))?;
        f.write_all(&out.stdout)
            .map_err(|e| Error::Internal(e.to_string()))?;

        // Best-effort repo deps preinstall (makepkg -s will also handle them)
        let srcinfo = String::from_utf8_lossy(&out.stdout);
        let deps = parse_srcinfo_deps(&srcinfo);
        if !deps.is_empty() {
            let _ = Command::new("pkexec")
                .args(["pacman", "-S", "--noconfirm", "--needed"])
                .args(deps.iter().map(|s| s.as_str()))
                .status();
        }

        // build (no -i)
        let status = Command::new("makepkg")
            .args(["-s", "--noconfirm"])
            .current_dir(&dir)
            .status()
            .map_err(|e| Error::Internal(e.to_string()))?;
        if !status.success() {
            return Err(Error::Aur("makepkg failed".into()));
        }

        // find artifact and install via pkexec -U
        let pkg =
            find_built_pkg(&dir).ok_or_else(|| Error::Aur("no built package found".into()))?;
        if !validate_pkg_path(&pkg) {
            return Err(Error::Aur("invalid built package path".into()));
        }
        let code = Command::new("pkexec")
            .args(["pacman", "-U", "--noconfirm", pkg.to_str().unwrap()])
            .status()
            .map_err(|e| Error::Priv(e.to_string()))?;
        if code.success() {
            Ok(())
        } else {
            Err(Error::Priv("pacman -U failed".into()))
        }
    }

    fn remove(&self, id: &PackageId, _sink: &ProgressSink, _cancel: &CancelToken) -> Result<()> {
        let code = Command::new("pkexec")
            .args(["pacman", "-Rns", "--noconfirm", &id.name])
            .status()
            .map_err(|e| Error::Priv(e.to_string()))?;
        if code.success() {
            Ok(())
        } else {
            Err(Error::Priv("remove failed".into()))
        }
    }

    fn upgrades(&self, _sink: &ProgressSink, _cancel: &CancelToken) -> Result<Vec<PackageSummary>> {
        Ok(vec![])
    }
}
