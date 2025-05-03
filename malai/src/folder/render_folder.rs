/// We will use few templates actually, skeleton.html, show-folder.html, show-file.html. We
/// will loop through each folder and file using the show-file/folder, and then pass the joined
/// HTML to folder-skeleton.html.
pub fn render_folder(path: &std::path::Path, base_path: &std::path::Path) -> eyre::Result<String> {
    let mut html = vec![];
    let display_path = relative_path(path, base_path);

    let (title, parent) = if display_path.is_empty() {
        ("Home".to_string(), "".to_string())
    } else {
        (
            display_path
                .strip_prefix("/")
                .unwrap_or(&display_path)
                .to_string(),
            "..".to_string(),
        )
    };

    // for content of this folder
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            html.push(show_folder(path, base_path)?);
        } else {
            html.push(show_file(path, base_path)?);
        }
    }

    html.sort();

    Ok(format!(
        include_str!("skeleton.html"),
        content = html.join(""),
        parent = parent,
        title = title
    ))
}

macro_rules! render_template {
    ($file:expr, $path:expr, $base:expr) => {{
        let name = $path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| eyre::anyhow!("failed to convert path to str"))?;

        let rpath = relative_path($path, $base);
        Ok(format!(include_str!($file), name = name, path = rpath))
    }};
}

fn show_file(path: std::path::PathBuf, base: &std::path::Path) -> eyre::Result<String> {
    render_template!("show-file.html", &path, base)
}

fn show_folder(path: std::path::PathBuf, base: &std::path::Path) -> eyre::Result<String> {
    render_template!("show-folder.html", &path, base)
}

fn relative_path(path: &std::path::Path, base_path: &std::path::Path) -> String {
    path.to_string_lossy()
        .replace(base_path.to_string_lossy().as_ref(), "")
}
