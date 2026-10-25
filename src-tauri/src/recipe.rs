//! Deterministic, copy-only recipes. A preview is registered server-side and consumed once.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

const PLAN_LIFETIME: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Debug, Serialize)]
pub struct PlannedFile {
    source: String,
    destination: String,
    bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Preview {
    plan_id: String,
    recipe_id: String,
    output_root: String,
    items: Vec<PlannedFile>,
    warnings: Vec<String>,
    total_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct RunResult {
    completed: Vec<String>,
    failed: Vec<String>,
}

#[derive(Clone)]
struct SourceStamp {
    canonical: PathBuf,
    length: u64,
    modified: SystemTime,
    digest: [u8; 32],
}

struct ApprovedPlan {
    preview: Preview,
    stamps: Vec<SourceStamp>,
    root: PathBuf,
    root_modified: SystemTime,
    created: SystemTime,
}

#[derive(Default)]
pub struct PlanStore {
    plans: Mutex<HashMap<String, ApprovedPlan>>,
}

fn bucket(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "heic" | "svg" | "bmp" | "tiff" => "Images",
        "pdf" | "doc" | "docx" | "txt" | "md" | "rtf" | "xls" | "xlsx" | "ppt" | "pptx" | "csv" => {
            "Documents"
        }
        "mp4" | "mov" | "mkv" | "avi" | "mp3" | "wav" | "flac" | "m4a" => "Media",
        "zip" | "7z" | "rar" | "tar" | "gz" => "Archives",
        _ => "Other",
    }
}

fn digest_file(path: &Path) -> io::Result<[u8; 32]> {
    let mut input = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hash.finalize().into())
}

fn stamp(path: &Path) -> Result<SourceStamp, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!("Not an ordinary file: {}", path.display()));
    }
    let canonical =
        fs::canonicalize(path).map_err(|e| format!("Cannot resolve {}: {e}", path.display()))?;
    let modified = metadata
        .modified()
        .map_err(|e| format!("Cannot inspect {}: {e}", path.display()))?;
    let digest = digest_file(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    let after = fs::symlink_metadata(path)
        .map_err(|e| format!("Cannot recheck {}: {e}", path.display()))?;
    if after.len() != metadata.len()
        || after.modified().ok() != Some(modified)
        || fs::canonicalize(path).ok() != Some(canonical.clone())
    {
        return Err(format!(
            "File changed while preparing preview: {}",
            path.display()
        ));
    }
    Ok(SourceStamp {
        canonical,
        length: metadata.len(),
        modified,
        digest,
    })
}

fn path_exists(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("Cannot inspect {}: {e}", path.display())),
    }
}

fn take_unique(dest: PathBuf, reserved: &mut HashSet<String>) -> Result<PathBuf, String> {
    let parent = dest.parent().ok_or("Missing output parent")?;
    let stem = dest.file_stem().unwrap_or_default().to_string_lossy();
    let extension = dest.extension().map(|s| s.to_string_lossy());
    for i in 0..100_000 {
        let candidate = if i == 0 {
            dest.clone()
        } else {
            let name = match &extension {
                Some(ext) => format!("{stem} ({i}).{ext}"),
                None => format!("{stem} ({i})"),
            };
            parent.join(name)
        };
        let key = candidate.to_string_lossy().to_lowercase();
        if !path_exists(&candidate)? && reserved.insert(key) {
            return Ok(candidate);
        }
    }
    Err("Too many duplicate destination names.".into())
}

fn prepare(
    paths: &[String],
    recipe_id: &str,
    output_directory: &str,
) -> Result<ApprovedPlan, String> {
    if paths.is_empty() {
        return Err("Choose at least one file.".into());
    }
    if paths.len() > 10_000 {
        return Err("This alpha supports up to 10,000 files per run.".into());
    }
    if !matches!(recipe_id, "organize" | "rename" | "collect") {
        return Err("Unknown recipe.".into());
    }
    let root =
        fs::canonicalize(output_directory).map_err(|_| "Choose an existing destination folder.")?;
    if !root.is_dir() {
        return Err("Choose an existing destination folder.".into());
    }
    let root_modified = fs::metadata(&root)
        .and_then(|metadata| metadata.modified())
        .map_err(|e| format!("Cannot inspect output folder: {e}"))?;
    let project_dir = match recipe_id {
        "organize" => "Intent - Organized",
        "rename" => "Intent - Renamed",
        _ => "Intent - Handoff",
    };
    let mut reserved = HashSet::new();
    let mut seen_parents = HashSet::new();
    let mut seen = HashSet::new();
    let mut items = Vec::new();
    let mut stamps = Vec::new();
    let mut total_bytes = 0_u64;
    for path in paths {
        let source = Path::new(path);
        let source_stamp = stamp(source)?;
        if !seen.insert(source_stamp.canonical.clone()) {
            continue;
        }
        let name = source.file_name().ok_or("A file is missing its name.")?;
        let parent = root.join(project_dir);
        let parent = if recipe_id == "organize" {
            parent.join(bucket(source))
        } else {
            parent
        };
        check_parents(&root, &parent.join("placeholder"))?;
        if seen_parents.insert(parent.clone()) {
            match fs::read_dir(&parent) {
                Ok(entries) => {
                    for entry in entries {
                        let entry = entry.map_err(|e| format!("Cannot read output folder: {e}"))?;
                        reserved.insert(entry.path().to_string_lossy().to_lowercase());
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(format!(
                        "Cannot read output folder {}: {e}",
                        parent.display()
                    ))
                }
            }
        }
        let target_name = if recipe_id == "rename" {
            format!("{:03}-{}", items.len() + 1, name.to_string_lossy())
        } else {
            name.to_string_lossy().to_string()
        };
        let dest = take_unique(parent.join(target_name), &mut reserved)?;
        total_bytes = total_bytes
            .checked_add(source_stamp.length)
            .ok_or("Total input size exceeds supported range.")?;
        items.push(PlannedFile {
            source: source.display().to_string(),
            destination: dest.display().to_string(),
            bytes: source_stamp.length,
        });
        stamps.push(source_stamp);
    }
    let preview = Preview {
        plan_id: Uuid::new_v4().to_string(),
        recipe_id: recipe_id.to_string(),
        output_root: root.display().to_string(),
        items,
        warnings: vec!["This plan expires after 15 minutes. If an input or destination changes, preview again. Originals remain untouched.".into()],
        total_bytes,
    };
    Ok(ApprovedPlan {
        preview,
        stamps,
        root,
        root_modified,
        created: SystemTime::now(),
    })
}

fn check_parents(root: &Path, destination: &Path) -> Result<(), String> {
    let parent = destination.parent().ok_or("Missing output parent")?;
    let relative = parent
        .strip_prefix(root)
        .map_err(|_| "Output escaped the approved folder.")?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() || !meta.is_dir() => {
                return Err(format!("Unsafe output folder: {}", current.display()))
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(format!(
                    "Cannot inspect output folder {}: {e}",
                    current.display()
                ))
            }
        }
    }
    Ok(())
}

fn preflight(plan: &ApprovedPlan) -> Result<(), String> {
    if plan.created.elapsed().unwrap_or(PLAN_LIFETIME) >= PLAN_LIFETIME {
        return Err("Preview expired. Preview again.".into());
    }
    if fs::canonicalize(&plan.root).ok() != Some(plan.root.clone())
        || fs::metadata(&plan.root)
            .and_then(|metadata| metadata.modified())
            .ok()
            != Some(plan.root_modified)
    {
        return Err("Output folder changed. Preview again.".into());
    }
    for (item, expected) in plan.preview.items.iter().zip(&plan.stamps) {
        let source = Path::new(&item.source);
        let actual = stamp(source)?;
        if actual.canonical != expected.canonical
            || actual.length != expected.length
            || actual.modified != expected.modified
            || actual.digest != expected.digest
        {
            return Err(format!(
                "Input changed after preview: {}. Preview again.",
                source.display()
            ));
        }
        let destination = Path::new(&item.destination);
        check_parents(&plan.root, destination)?;
        if path_exists(destination)? {
            return Err(format!(
                "Destination is now occupied: {}. Preview again.",
                destination.display()
            ));
        }
    }
    Ok(())
}

fn ensure_parents(root: &Path, destination: &Path) -> io::Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing output parent"))?;
    let relative = parent.strip_prefix(root).map_err(|_| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Output escaped approved folder",
        )
    })?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        match fs::create_dir(&current) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
        let meta = fs::symlink_metadata(&current)?;
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Unsafe output folder",
            ));
        }
    }
    Ok(())
}

fn copy_verified(
    source: &Path,
    destination: &Path,
    expected: &SourceStamp,
    root: &Path,
) -> io::Result<()> {
    ensure_parents(root, destination)?;
    let actual = stamp(source).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if actual.canonical != expected.canonical
        || actual.length != expected.length
        || actual.modified != expected.modified
        || actual.digest != expected.digest
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Input changed after preview",
        ));
    }
    if path_exists(destination).map_err(io::Error::other)? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Destination became occupied",
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing output parent"))?;
    let temporary = parent.join(format!(".intent-{}.partial", Uuid::new_v4()));
    let copy_result = (|| -> io::Result<()> {
        let mut input = File::open(source)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let mut hash = Sha256::new();
        let mut length = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            output.write_all(&buffer[..count])?;
            hash.update(&buffer[..count]);
            length += count as u64;
        }
        output.sync_all()?;
        if length != expected.length || <[u8; 32]>::from(hash.finalize()) != expected.digest {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Input changed during copy",
            ));
        }
        let after = stamp(source).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if after.canonical != expected.canonical
            || after.length != expected.length
            || after.modified != expected.modified
            || after.digest != expected.digest
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Input changed during copy",
            ));
        }
        // A same-filesystem hard link publishes the complete temporary file without replacement.
        fs::hard_link(&temporary, destination)?;
        Ok(())
    })();
    let _ = fs::remove_file(&temporary);
    copy_result
}

impl PlanStore {
    pub fn preview(
        &self,
        paths: &[String],
        recipe_id: &str,
        output_directory: &str,
    ) -> Result<Preview, String> {
        let plan = prepare(paths, recipe_id, output_directory)?;
        let response = plan.preview.clone();
        let mut plans = self.plans.lock().map_err(|_| "Plan store unavailable")?;
        plans.retain(|_, existing| {
            existing
                .created
                .elapsed()
                .is_ok_and(|age| age < PLAN_LIFETIME)
        });
        plans.insert(response.plan_id.clone(), plan);
        Ok(response)
    }

    pub fn execute(&self, plan_id: &str) -> Result<RunResult, String> {
        let plan = self
            .plans
            .lock()
            .map_err(|_| "Plan store unavailable")?
            .remove(plan_id)
            .ok_or("Preview unavailable. Preview again.")?;
        preflight(&plan)?;
        let mut result = RunResult {
            completed: vec![],
            failed: vec![],
        };
        for (item, stamp) in plan.preview.items.iter().zip(&plan.stamps) {
            match copy_verified(
                Path::new(&item.source),
                Path::new(&item.destination),
                stamp,
                &plan.root,
            ) {
                Ok(()) => result.completed.push(item.destination.clone()),
                Err(err) => result.failed.push(format!("{}: {err}", item.source)),
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn text(path: &Path, content: &[u8]) {
        fs::write(path, content).unwrap();
    }
    fn one(root: &Path, source: &Path, recipe: &str) -> (PlanStore, Preview) {
        let store = PlanStore::default();
        let preview = store
            .preview(
                &[source.display().to_string()],
                recipe,
                root.to_str().unwrap(),
            )
            .unwrap();
        (store, preview)
    }
    #[test]
    fn all_recipes_copy_without_changing_originals() {
        for recipe in ["organize", "rename", "collect"] {
            let dir = tempfile::tempdir().unwrap();
            let source = dir.path().join("picture.PNG");
            text(&source, b"source bytes");
            let (store, preview) = one(dir.path(), &source, recipe);
            if recipe == "organize" {
                assert!(preview.items[0].destination.contains("Images"));
            }
            let result = store.execute(&preview.plan_id).unwrap();
            assert_eq!(result.completed.len(), 1);
            assert!(result.failed.is_empty());
            assert_eq!(fs::read(&source).unwrap(), b"source bytes");
            assert_eq!(fs::read(&result.completed[0]).unwrap(), b"source bytes");
            assert!(store.execute(&preview.plan_id).is_err());
        }
    }
    #[test]
    fn duplicate_names_have_distinct_outputs() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        fs::create_dir(&a).unwrap();
        fs::create_dir(&b).unwrap();
        let first = a.join("same.txt");
        let second = b.join("same.txt");
        text(&first, b"A");
        text(&second, b"B");
        let store = PlanStore::default();
        let preview = store
            .preview(
                &[first.display().to_string(), second.display().to_string()],
                "collect",
                dir.path().to_str().unwrap(),
            )
            .unwrap();
        assert_ne!(preview.items[0].destination, preview.items[1].destination);
        let result = store.execute(&preview.plan_id).unwrap();
        assert_eq!(result.completed.len(), 2);
        assert_eq!(fs::read(&result.completed[0]).unwrap(), b"A");
        assert_eq!(fs::read(&result.completed[1]).unwrap(), b"B");
    }

    #[test]
    fn hundred_file_batch_preserves_every_source() {
        let dir = tempfile::tempdir().unwrap();
        let input_dir = dir.path().join("inputs");
        let output_dir = dir.path().join("outputs");
        fs::create_dir(&input_dir).unwrap();
        fs::create_dir(&output_dir).unwrap();
        let paths: Vec<String> = (0..100)
            .map(|index| {
                let path = input_dir.join(format!("résumé-{index:03}.txt"));
                text(&path, format!("source content {index}").as_bytes());
                path.display().to_string()
            })
            .collect();
        let original_digests: Vec<_> = paths
            .iter()
            .map(|path| digest_file(Path::new(path)).unwrap())
            .collect();
        let store = PlanStore::default();
        let preview = store
            .preview(&paths, "collect", output_dir.to_str().unwrap())
            .unwrap();
        assert_eq!(preview.items.len(), 100);
        let result = store.execute(&preview.plan_id).unwrap();
        assert_eq!(result.completed.len(), 100);
        assert!(result.failed.is_empty());
        for (index, path) in paths.iter().enumerate() {
            assert_eq!(
                digest_file(Path::new(path)).unwrap(),
                original_digests[index]
            );
            assert_eq!(
                digest_file(Path::new(&result.completed[index])).unwrap(),
                original_digests[index]
            );
        }
    }
    #[test]
    fn changed_source_fails_before_writing() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("input.txt");
        text(&source, b"before");
        let (store, preview) = one(dir.path(), &source, "collect");
        text(&source, b"after!");
        assert!(store
            .execute(&preview.plan_id)
            .unwrap_err()
            .contains("changed"));
        assert!(!Path::new(&preview.items[0].destination).exists());
    }
    #[test]
    fn collision_after_preview_fails_without_replanning() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("input.txt");
        text(&source, b"source");
        fs::create_dir(dir.path().join("Intent - Handoff")).unwrap();
        let (store, preview) = one(dir.path(), &source, "collect");
        let destination = Path::new(&preview.items[0].destination);
        text(destination, b"other process");
        assert!(store
            .execute(&preview.plan_id)
            .unwrap_err()
            .contains("occupied"));
        assert_eq!(fs::read(destination).unwrap(), b"other process");
        assert_eq!(fs::read(source).unwrap(), b"source");
    }
    #[test]
    fn existing_case_variant_gets_distinct_name() {
        let dir = tempfile::tempdir().unwrap();
        let input_dir = dir.path().join("input");
        let output_dir = dir.path().join("Intent - Handoff");
        fs::create_dir(&input_dir).unwrap();
        fs::create_dir(&output_dir).unwrap();
        let source = input_dir.join("same.txt");
        text(&source, b"source");
        let occupied = output_dir.join("SAME.txt");
        text(&occupied, b"occupied");
        let (store, preview) = one(dir.path(), &source, "collect");
        assert!(preview.items[0].destination.contains("same (1).txt"));
        let result = store.execute(&preview.plan_id).unwrap();
        assert_eq!(result.completed.len(), 1);
        assert_eq!(fs::read(&occupied).unwrap(), b"occupied");
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("folder");
        fs::create_dir(&folder).unwrap();
        let store = PlanStore::default();
        let root = dir.path().to_str().unwrap();
        assert!(store.preview(&[], "collect", root).is_err());
        assert!(store
            .preview(&[folder.display().to_string()], "collect", root)
            .is_err());
        assert!(store
            .preview(
                &[dir.path().join("missing").display().to_string()],
                "collect",
                root
            )
            .is_err());
        assert!(store.preview(&["a".into()], "shell", root).is_err());
    }
    #[test]
    fn expired_plan_and_unknown_id_never_write() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("input.txt");
        text(&source, b"keep original");
        let (store, preview) = one(dir.path(), &source, "collect");
        let expired_at = SystemTime::now() - PLAN_LIFETIME - Duration::from_secs(1);
        store
            .plans
            .lock()
            .unwrap()
            .get_mut(&preview.plan_id)
            .unwrap()
            .created = expired_at;
        assert!(store
            .execute(&preview.plan_id)
            .unwrap_err()
            .contains("expired"));
        assert!(store.execute("unrecognized-id").is_err());
        assert!(!Path::new(&preview.items[0].destination).exists());
        assert_eq!(fs::read(source).unwrap(), b"keep original");
    }

    #[test]
    fn same_plan_cannot_execute_twice_concurrently() {
        use std::sync::{Arc, Barrier};
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("input.txt");
        text(&source, b"one copy only");
        let store = Arc::new(PlanStore::default());
        let preview = store
            .preview(
                &[source.display().to_string()],
                "collect",
                dir.path().to_str().unwrap(),
            )
            .unwrap();
        let barrier = Arc::new(Barrier::new(3));
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let store = Arc::clone(&store);
                let barrier = Arc::clone(&barrier);
                let id = preview.plan_id.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    store.execute(&id)
                })
            })
            .collect();
        barrier.wait();
        let results: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            fs::read(&preview.items[0].destination).unwrap(),
            b"one copy only"
        );
        assert_eq!(fs::read(source).unwrap(), b"one copy only");
    }

    #[test]
    fn separate_concurrent_plans_never_overwrite_each_other() {
        use std::sync::{Arc, Barrier};
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("input.txt");
        text(&source, b"source bytes");
        fs::create_dir(dir.path().join("Intent - Handoff")).unwrap();
        let store = Arc::new(PlanStore::default());
        let paths = [source.display().to_string()];
        let first = store
            .preview(&paths, "collect", dir.path().to_str().unwrap())
            .unwrap();
        let second = store
            .preview(&paths, "collect", dir.path().to_str().unwrap())
            .unwrap();
        assert_eq!(first.items[0].destination, second.items[0].destination);
        let barrier = Arc::new(Barrier::new(3));
        let handles: Vec<_> = [first.plan_id, second.plan_id]
            .into_iter()
            .map(|id| {
                let store = Arc::clone(&store);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    store.execute(&id)
                })
            })
            .collect();
        barrier.wait();
        let results: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        let copied: usize = results
            .iter()
            .map(|result| result.as_ref().map_or(0, |run| run.completed.len()))
            .sum();
        assert_eq!(copied, 1);
        assert_eq!(
            fs::read(&first.items[0].destination).unwrap(),
            b"source bytes"
        );
        assert_eq!(fs::read(source).unwrap(), b"source bytes");
    }

    #[test]
    fn malformed_path_is_rejected_without_panic() {
        let dir = tempfile::tempdir().unwrap();
        let store = PlanStore::default();
        assert!(store
            .preview(
                &["invalid\0path".into()],
                "collect",
                dir.path().to_str().unwrap()
            )
            .is_err());
        assert!(store
            .preview(&["missing.txt".into()], "collect", "invalid\0output")
            .is_err());
    }

    #[cfg(windows)]
    #[test]
    fn output_junction_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let root = dir.path().join("output");
        let outside = dir.path().join("outside");
        fs::create_dir(&root).unwrap();
        fs::create_dir(&outside).unwrap();
        text(&source, b"source bytes");
        let junction = root.join("Intent - Handoff");
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&outside)
            .status()
            .unwrap();
        assert!(status.success(), "could not create test junction");
        assert!(PlanStore::default()
            .preview(
                &[source.display().to_string()],
                "collect",
                root.to_str().unwrap()
            )
            .is_err());
        assert!(fs::read_dir(&outside).unwrap().next().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn source_symlink_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("real.txt");
        text(&source, b"safe");
        let link = dir.path().join("link.txt");
        std::os::unix::fs::symlink(&source, &link).unwrap();
        assert!(PlanStore::default()
            .preview(
                &[link.display().to_string()],
                "collect",
                dir.path().to_str().unwrap()
            )
            .is_err());
    }
}
