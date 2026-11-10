use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;
use tauri::Emitter;

const DATA_FILES: [&str; 3] = ["settings", "library", "covers"];

#[derive(Default)]
struct LaunchFiles(Mutex<Vec<String>>);

fn epub_args(args: Vec<String>, cwd: &Path) -> Vec<String> {
    args.into_iter()
        .filter(|arg| arg.to_lowercase().ends_with(".epub"))
        .map(|arg| {
            let path = PathBuf::from(&arg);
            if path.is_absolute() { path } else { cwd.join(path) }
        })
        .filter(|path| path.is_file())
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

fn push_launch_files(files: &LaunchFiles, args: Vec<String>, cwd: &Path) {
    let found = epub_args(args, cwd);
    if let Ok(mut list) = files.0.lock() {
        list.extend(found);
    }
}

fn data_path(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    if !DATA_FILES.contains(&name) {
        return Err(format!("Invalid data file: {name}"));
    }
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join(format!("{name}.json")))
}

#[tauri::command]
fn load_data(app: tauri::AppHandle, name: String) -> Result<Option<String>, String> {
    let path = data_path(&app, &name)?;
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(&path).map(Some).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_data(app: tauri::AppHandle, name: String, contents: String) -> Result<(), String> {
    let path = data_path(&app, &name)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, contents).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_epub(path: String) -> Result<tauri::ipc::Response, String> {
    if !path.to_lowercase().ends_with(".epub") {
        return Err("Not an EPUB file".into());
    }
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    Ok(tauri::ipc::Response::new(bytes))
}

#[tauri::command]
fn epub_size(path: String) -> Result<u64, String> {
    if !path.to_lowercase().ends_with(".epub") {
        return Err("Not an EPUB file".into());
    }
    fs::metadata(&path).map(|m| m.len()).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_epub_range(path: String, offset: u64, len: u64) -> Result<tauri::ipc::Response, String> {
    if !path.to_lowercase().ends_with(".epub") {
        return Err("Not an EPUB file".into());
    }
    if len > 256 * 1024 * 1024 {
        return Err("Range too large".into());
    }
    let end = offset.checked_add(len).ok_or("Range out of bounds")?;
    let mut file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let file_len = file.metadata().map_err(|e| e.to_string())?.len();
    if end > file_len {
        return Err("Range out of bounds".into());
    }
    file.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; len as usize];
    file.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(tauri::ipc::Response::new(buf))
}

#[tauri::command]
fn take_launch_files(files: tauri::State<LaunchFiles>) -> Vec<String> {
    files.0.lock().map(|mut list| std::mem::take(&mut *list)).unwrap_or_default()
}

#[tauri::command]
fn check_paths(paths: Vec<String>) -> Vec<bool> {
    paths.iter().map(|p| Path::new(p).is_file()).collect()
}

#[tauri::command]
fn check_dirs(paths: Vec<String>) -> Vec<bool> {
    paths.iter().map(|p| Path::new(p).is_dir()).collect()
}

#[tauri::command]
fn scan_epubs(dir: String) -> Result<Vec<String>, String> {
    let root = PathBuf::from(dir);
    if !root.is_dir() {
        return Err("Folder not found".into());
    }
    let mut out = Vec::new();
    let mut stack = vec![(root, 0u32)];
    while let Some((folder, depth)) = stack.pop() {
        let entries = match fs::read_dir(&folder) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };
            if file_type.is_dir() && depth < 2 {
                stack.push((path, depth + 1));
            } else if file_type.is_file() {
                let is_epub = path
                    .extension()
                    .map(|ext| ext.eq_ignore_ascii_case("epub"))
                    .unwrap_or(false);
                if is_epub {
                    out.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }
    out.sort_by_cached_key(|p| {
        Path::new(p)
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    });
    Ok(out)
}

#[tauri::command]
fn toggle_fullscreen(window: tauri::Window) -> Result<(), String> {
    let fullscreen = window.is_fullscreen().map_err(|e| e.to_string())?;
    window.set_fullscreen(!fullscreen).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            push_launch_files(app.state::<LaunchFiles>().inner(), argv, Path::new(&cwd));
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("open-files", ());
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(LaunchFiles::default())
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            push_launch_files(app.state::<LaunchFiles>().inner(), args, &std::env::current_dir().unwrap_or_default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_data,
            save_data,
            read_epub,
            epub_size,
            read_epub_range,
            take_launch_files,
            check_paths,
            check_dirs,
            scan_epubs,
            toggle_fullscreen
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
