use eyre::WrapErr;

pub fn mkdir(parent: &std::path::Path, name: &str) -> eyre::Result<std::path::PathBuf> {
    let path = parent.join(name);

    std::fs::create_dir_all(&path)
        .wrap_err_with(|| format!("failed to create {name}: {path:?}"))?;
    Ok(path)
}

// TODO: convert it to use id52 (we will store id52 in keyring)
fn keyring_entry(id: &str) -> eyre::Result<keyring::Entry> {
    keyring::Entry::new("FTNet", id)
        .wrap_err_with(|| format!("failed to create keyring Entry for {id}"))
}

// TODO: convert it to use id52 (we will store id52 in keyring)
pub fn save_secret(secret_key: &iroh::SecretKey) -> eyre::Result<()> {
    let public = secret_key.public().to_string();
    Ok(keyring_entry(public.as_str())?.set_secret(&secret_key.to_bytes())?)
}

// TODO: convert it to use id52 (we will store id52 in keyring)
pub fn get_secret(id: &str) -> eyre::Result<iroh::SecretKey> {
    let entry = keyring_entry(id)?;
    let secret = entry
        .get_secret()
        .wrap_err_with(|| format!("keyring: secret not found for {id}"))?;

    if secret.len() != 32 {
        return Err(eyre::anyhow!(
            "keyring: secret has invalid length: {}",
            secret.len()
        ));
    }

    let bytes: [u8; 32] = secret.try_into().unwrap(); // unwrap ok as already asserted
    Ok(iroh::SecretKey::from_bytes(&bytes))
}

pub fn create_public_key(store: bool) -> eyre::Result<iroh::PublicKey> {
    let mut rng = rand::rngs::OsRng;
    let secret_key = iroh::SecretKey::generate(&mut rng);
    // we do not want to keep secret key in memory, only in keychain
    if store {
        save_secret(&secret_key).wrap_err_with(|| "failed to store secret key to keychain")?;
    }
    Ok(secret_key.public())
}

pub async fn get_endpoint(id: &str) -> eyre::Result<iroh::Endpoint> {
    let secret_key = ftnet::utils::get_secret(id)
        .wrap_err_with(|| format!("failed to get secret key from keychain for {id}"))?;

    match iroh::Endpoint::builder()
        .discovery_n0()
        .discovery_local_network()
        .alpns(vec![ftnet::APNS_IDENTITY.into()])
        .secret_key(secret_key)
        .bind()
        .await
    {
        Ok(ep) => Ok(ep),
        Err(e) => {
            // https://github.com/n0-computer/iroh/issues/2741
            // this is why you MUST NOT use anyhow::Error etc. in library code.
            Err(eyre::anyhow!("failed to bind to iroh network2: {e:?}"))
        }
    }
}

pub type FrameReader =
    tokio_util::codec::FramedRead<iroh::endpoint::RecvStream, tokio_util::codec::LinesCodec>;

pub fn frame_reader(recv: iroh::endpoint::RecvStream) -> FrameReader {
    tokio_util::codec::FramedRead::new(recv, tokio_util::codec::LinesCodec::new())
}

pub fn id52_to_public_key(id: &str) -> eyre::Result<iroh::PublicKey> {
    let bytes = data_encoding::BASE32_DNSSEC.decode(id.as_bytes())?;
    if bytes.len() != 32 {
        return Err(eyre::anyhow!(
            "read: id has invalid length: {}",
            bytes.len()
        ));
    }

    let bytes: [u8; 32] = bytes.try_into().unwrap(); // unwrap ok as already asserted

    iroh::PublicKey::from_bytes(&bytes).wrap_err_with(|| "failed to parse id to public key")
}

pub fn public_key_to_id52(key: &iroh::PublicKey) -> String {
    data_encoding::BASE32_DNSSEC.encode(key.as_bytes())
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
