use eyre::WrapErr;

pub fn mkdir(parent: &std::path::Path, name: &str) -> eyre::Result<std::path::PathBuf> {
    let path = parent.join(name);

    std::fs::create_dir_all(&path)
        .wrap_err_with(|| format!("failed to create {name}: {path:?}"))?;
    Ok(path)
}

fn keyring_entry(id: &str) -> eyre::Result<keyring::Entry> {
    keyring::Entry::new("FTNet", id)
        .wrap_err_with(|| format!("failed to create keyring Entry for {id}"))
}

pub fn save_secret(secret_key: &iroh::SecretKey) -> eyre::Result<()> {
    let public = secret_key.public().to_string();
    Ok(keyring_entry(public.as_str())?.set_secret(&secret_key.to_bytes())?)
}

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

pub type FrameReader =
    tokio_util::codec::FramedRead<iroh::endpoint::RecvStream, tokio_util::codec::LinesCodec>;

pub fn frame_reader(recv: iroh::endpoint::RecvStream) -> FrameReader {
    tokio_util::codec::FramedRead::new(recv, tokio_util::codec::LinesCodec::new())
}

pub fn id52_to_public_key(id: &str) -> eyre::Result<iroh::PublicKey> {
    use eyre::WrapErr;

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

    let client = reqwest::Client::new();

    {
        let mut file = tokio::fs::File::create(dir.join("template.zip")).await?;
        let url =
            format!("https://www.fifthtry.com/ft2/api/site/download?site-slug={template_slug}");

        download(&client, &url, &mut file).await?;

        tracing::info!("template zip downloaded");
    }

    {
        let mut version_file = tokio::fs::File::create(dir.join("version")).await?;
        let version_url =
            format!("https://www.fifthtry.com/ft2/ops/last-hash/?site-id={template_slug}");

        let res = client.get(version_url).send().await?;
        let version = res.text().await?;
        version_file.write_all(version.as_bytes()).await?;

        tracing::info!(version = %version, "version downloaded");
    }

    // create template dir and unzip the template.zip
    let template_dir = mkdir(dir, "template")?;

    let zip_file  = std::fs::File::open(dir.join("template.zip"))?;
    let mut archive = zip::ZipArchive::new(zip_file)?;

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


    Ok(())
}

#[tracing::instrument(skip_all)]
async fn download(
    client: &reqwest::Client,
    url: &str,
    file: &mut tokio::fs::File,
) -> eyre::Result<()> {
    use tokio::io::AsyncWriteExt;
    use tokio_stream::StreamExt;

    let res = client.get(url).send().await?.error_for_status()?;

    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let item = item?;
        file.write_all(&item).await?;
    }

    Ok(())
}
