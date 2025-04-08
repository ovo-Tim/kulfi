use eyre::WrapErr;

pub fn mkdir(parent: &std::path::Path, name: &str) -> eyre::Result<std::path::PathBuf> {
    let path = parent.join(name);

    std::fs::create_dir_all(&path)
        .wrap_err_with(|| format!("failed to create {name}: {path:?}"))?;
    Ok(path)
}

/// Download the package given its [template_slug] and put it in [dir]/template/ directory.
/// [dir]/version is a text file created that contains the checkpoint of the template.
#[tracing::instrument]
pub async fn download_package_template(
    dir: &std::path::Path,
    template_slug: String,
) -> eyre::Result<()> {
    use tokio::io::AsyncWriteExt;
    use tokio_stream::StreamExt;

    let client = reqwest::Client::new();

    let version = {
        let mut file = tokio::fs::File::create(dir.join("template.zip")).await?;
        let url = format!("https://www.fifthtry.com/{template_slug}.zip");
        let res = client.get(url).send().await?.error_for_status()?;

        let version = res
            .headers()
            .get("etag")
            .unwrap()
            .to_str()
            .unwrap()
            // remove the quotes from the etag if they exist
            .trim_matches('"')
            .to_string();

        tracing::info!(package_checkpoint = %version);

        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let item = item?;
            file.write_all(&item).await?;
        }

        tracing::info!("template zip downloaded");

        version
    };

    {
        let mut version_file = tokio::fs::File::create(dir.join("version")).await?;
        version_file.write_all(version.as_bytes()).await?;

        tracing::info!("version file written");
    }

    // create template dir and unzip the template.zip
    let template_dir = mkdir(dir, "template")?;

    let zip_file = std::fs::File::open(dir.join("template.zip"))?;
    let mut archive = zip::ZipArchive::new(zip_file)?;

    tracing::info!("unzipping {} files", archive.len());

    // TODO: use tokio::fs and tokio::io
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = template_dir.join(file.name());

        if file.is_dir() {
            tokio::fs::create_dir_all(&outpath).await?;
        } else {
            if let Some(parent) = outpath.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    tracing::info!("template unzipped");

    drop(archive);
    tokio::fs::remove_file(dir.join("template.zip")).await?;

    tracing::info!("template zip removed");

    Ok(())
}

/// Synchronously copy a directory and its contents to a new location.
#[tracing::instrument]
pub fn copy_dir(src: &std::path::Path, dest: &std::path::Path) -> eyre::Result<()> {
    use std::fs;

    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if path.is_dir() {
            copy_dir(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// Runs the fastn binary with the given arguments in the specified directory.
/// Assumes that the fastn binary is in the PATH.
///
/// The stdout of the command is captured and returned as a String.
///
/// # Example
///
/// ```rust,ignore
/// run_fastn("~/my-fastn-project/", &["update"]);
/// ```
#[tracing::instrument]
pub fn run_fastn(dir: &std::path::Path, args: &[&str]) -> eyre::Result<String> {
    let mut cmd = std::process::Command::new("fastn");
    cmd.current_dir(dir);
    cmd.args(args);
    cmd.stdout(std::process::Stdio::piped());

    let output = cmd.output()?;

    tracing::info!("fastn command done. {}", output.status);

    let output_str = String::from_utf8_lossy(&output.stdout);

    tracing::debug!("fastn command output: {output_str}",);

    if !output.status.success() {
        return Err(eyre::eyre!("fastn update failed"));
    }

    Ok(output_str.to_string())
}
