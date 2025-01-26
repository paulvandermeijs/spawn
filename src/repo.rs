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

    info!(
        "Checking out into {:?} ...",
        prepare_checkout.repo().work_dir().expect("should be there")
    );

    let (repo, _) =
        prepare_checkout.main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;
    info!(
        "Repo cloned into {:?}",
        repo.work_dir().expect("directory pre-created")
    );

    let remote = repo
        .find_default_remote(gix::remote::Direction::Fetch)
        .expect("always present after clone")?;

    info!(
        "Default remote: {} -> {}",
        remote
            .name()
            .expect("default remote is always named")
            .as_bstr(),
        remote
            .url(gix::remote::Direction::Fetch)
            .expect("should be the remote URL")
            .to_bstring(),
    );

    Ok(())
}
