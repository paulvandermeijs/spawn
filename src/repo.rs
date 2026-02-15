use anyhow::Result;
use log::info;
use std::path::Path;

pub(crate) fn clone(repo_url: &str, dst: &Path) -> Result<()> {
    // SAFETY: The closure doesn't use mutexes or memory allocation, so it should be safe to call from a signal handler.
    unsafe {
        gix::interrupt::init_handler(1, || {})?;
    }
    std::fs::create_dir_all(dst)?;
    let url = gix::url::parse(repo_url.into())?;

    info!("Url: {:?}", url.to_bstring());

    let mut prepare_clone = gix::prepare_clone(url, dst)?;

    info!("Cloning {repo_url:?} into {dst:?}...");

    let (mut prepare_checkout, _) = prepare_clone
        .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

    if let Some(work_dir) = prepare_checkout.repo().work_dir() {
        info!("Checking out into {work_dir:?} ...");
    }

    let (repo, _) =
        prepare_checkout.main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

    if let Some(work_dir) = repo.work_dir() {
        info!("Repo cloned into {work_dir:?}");
    }

    if let Some(remote) = repo.find_default_remote(gix::remote::Direction::Fetch) {
        let remote = remote?;

        if let (Some(name), Some(url)) = (remote.name(), remote.url(gix::remote::Direction::Fetch))
        {
            info!("Default remote: {} -> {}", name.as_bstr(), url.to_bstring());
        }
    }

    Ok(())
}
