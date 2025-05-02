/// We will use few templates actually, skeleton.html, show-folder.html, show-file.html. We
/// will loop through each folder and file using the show-file/folder, and then pass the joined
/// HTML to folder-skeleton.html.
pub fn render_folder(path: &std::path::Path) -> eyre::Result<String> {
    let mut html = String::new();
    // for content of this folder
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            html.push_str(&show_folder(path)?);
        } else {
            html.push_str(&show_file(path)?);
        }
    }

    return Ok(format!(
        include_str!("skeleton.html"),
        content = html,
        path = path.to_string_lossy()
    ));

    macro_rules! render_template {
        ($file:expr, $path:expr) => {{
            let path = $path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| eyre::anyhow!("failed to convert path to str"))?;

            Ok(format!(include_str!($file), path = path))
        }};
    }

    fn show_file(path: std::path::PathBuf) -> eyre::Result<String> {
        render_template!("show-file.html", path)
    }

    fn show_folder(path: std::path::PathBuf) -> eyre::Result<String> {
        render_template!("show-folder.html", path)
    }
}
