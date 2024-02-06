//! Custom [`Write`] implementation to set politics for writing into files.
//! For more information see [`RollingFileWriter::new`]
use std::{
    collections::LinkedList,
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::{ErrorKind, Write},
    ops::{Add, Sub},
    path::PathBuf,
    sync::{PoisonError, RwLock},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Copy)]
enum RollingStrategy {
    None,
    TimeBased(Duration),
    SizeBased(usize),
    Mixed(Duration, usize),
}

/// Log writer to the segmented file.
/// Contains also policy to keep only fixed number of files or some newest with fixed age.
pub struct RollingFileWriter {
    file: RwLock<Option<File>>,
    strategy: RollingStrategyImpl,
    retention: RetentionPolicyImpl,
    zipper: ZipFiles,
    file_name: String,
    file_extension: String,
    file_path: String,
    full_file_name: Option<String>,
    enable_zip: bool,
}

impl Drop for RollingFileWriter {
    fn drop(&mut self) {
        let mut file_opt = self.get_writer().take();
        if let Some(file) = &mut file_opt {
            let _ = file.flush();
        }
    }
}

impl RollingFileWriter {
    /// Creates new writer
    /// # Arguments
    /// * 'file_path' - path to log files folder. Could be absolute or relative.
    /// * 'file_name' - name of active file.
    /// * 'interval' - optional interval to split file into chunks with fixed interval.
    /// * 'limit' - optional interval of time which means maximum interval when log file is available for recoding.
    ///   When this interval exceed active file is renamed to chunk file with format [file_name]-yy-MM-dd-HH-mm-ss[.ext].
    /// * 'max_age - maximum age of kept files.
    /// * 'max_count' - maximum count of kept files.
    /// * 'enable_zip' - flag to enable/disable archiving files
    /// # Returns
    /// New structure.
    pub fn new(
        file_path: impl Into<String>,
        file_name: impl Into<String>,
        interval: Option<Duration>,
        limit: Option<usize>,
        max_age: Option<Duration>,
        max_count: Option<usize>,
        enable_zip: bool,
    ) -> Self {
        let file_path = file_path.into();
        let file_path = if file_path.ends_with('/') || file_path.ends_with('\\') {
            file_path
        } else {
            format!("{file_path}/")
        };
        let mut file_name = file_name.into();
        let mut path = PathBuf::new();
        path.push(file_name.as_str());
        let file_extension = path
            .extension()
            .map(OsStr::to_os_string)
            .map(|v| v.into_string().unwrap_or_default())
            .unwrap_or_default();
        if !file_extension.is_empty() {
            file_name = file_name.replace(format!(".{}", file_extension.as_str()).as_str(), "");
        }

        let strategy = RollingStrategyImpl::new(interval, limit);
        let retention = RetentionPolicyImpl::new(max_count, max_age);
        let mut result = Self {
            file: RwLock::new(None),
            strategy,
            retention,
            file_name,
            file_path,
            file_extension,
            full_file_name: None,
            zipper: ZipFiles::default(),
            enable_zip,
        };
        result.full_file_name = Some(result.build_active_file_name());
        result.retention.init(
            result.file_path.as_str(),
            result.file_name.as_str(),
            result.file_extension.as_str(),
        );
        let _ = result.rename_file(None);
        let _ = result.create_file();
        result
    }
    fn get_writer(&mut self) -> &mut Option<File> {
        self.file.get_mut().unwrap_or_else(PoisonError::into_inner)
    }
    fn set_writer(&mut self, file: File) {
        self.file
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
            .replace(file);
        self.strategy.created = SystemTime::now();
    }
    fn build_active_file_name(&mut self) -> String {
        let path = self.file_path.as_str();
        let file_path = match std::fs::create_dir_all(path) {
            Ok(_) => path,
            Err(_) => {
                self.file_path = "./".to_owned();
                "./"
            }
        };
        let mut path = PathBuf::new();
        path.push(file_path);
        path.push(self.file_name.as_str());
        path.set_extension(self.file_extension.as_str());
        if let Some(fname) = path.to_str() {
            fname.to_owned()
        } else {
            self.file_path = ".".to_owned();
            format!("./{}", self.file_name.as_str())
        }
    }
    fn create_file(&mut self) -> Result<(), std::io::Error> {
        let full_file_name = self.build_active_file_name();
        let _ = std::fs::remove_file(&full_file_name);
        self.set_writer(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&full_file_name)?,
        );
        self.full_file_name = Some(full_file_name);
        Ok(())
    }

    fn rename_file(&mut self, suffix: Option<String>) -> std::io::Result<()> {
        if let Some(full_file_name) = self.full_file_name.take() {
            if let Some(mut file) = self.get_writer().take() {
                file.flush()?;
            }
            let suffix = if let Some(suf) = suffix {
                suf
            } else if let Some(time) = SystemTime::now().checked_sub(Duration::from_secs(1)) {
                FileNameUtils::time_to_str(time)
            } else {
                let metadata = File::open(full_file_name.as_str())?.metadata()?;
                let created = metadata.created()?;
                FileNameUtils::time_to_str(created)
            };
            let mut new_file_name = FileNameUtils::build_file_name_with_suffix(self, &suffix);
            if let Ok(metadata) = std::fs::metadata(new_file_name.as_str()) {
                if metadata.is_file() {
                    new_file_name = FileNameUtils::get_new_file_name_if_exists(self, &suffix);
                }
            }
            std::fs::rename(full_file_name.as_str(), new_file_name.as_str())?;
            if let Some(time) = FileNameUtils::get_time_from_name(
                new_file_name
                    .replace(self.file_path.as_str(), "")
                    .replace(self.file_name.as_str(), "")
                    .replace(self.file_extension.as_str(), ""),
            ) {
                if self.enable_zip {
                    let zip_file = format!("{}.zip", new_file_name.as_str());
                    self.zipper.zip_file(
                        new_file_name.as_str(),
                        zip_file.as_str(),
                        self.file_path.as_str(),
                    );
                    self.retention.new_item(new_file_name, time, Some(zip_file));
                } else {
                    self.retention.new_item(new_file_name, time, None);
                }
            }
        }
        Ok(())
    }
}

impl Write for RollingFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(file) = self.get_writer() {
            file.write(buf)
        } else {
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(file) = self.get_writer() {
            file.flush()
        } else {
            Ok(())
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn write_all(&mut self, mut buf: &[u8]) -> std::io::Result<()> {
        if self.strategy.should_rollover(buf.len()) {
            self.rename_file(Some(FileNameUtils::time_to_str(self.strategy.created)))?;
            self.create_file()?;
        }
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(std::io::Error::new(
                        ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ));
                }
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

struct FileNameUtils {}
impl FileNameUtils {
    fn time_to_str(time: SystemTime) -> String {
        chrono::DateTime::<Utc>::from(time)
            .format("-%y-%m-%d-%H-%M-%S")
            .to_string()
    }
    fn get_time_from_name(name: String) -> Option<DateTime<Utc>> {
        let mut name = name;

        let patterns = if name.ends_with('.') {
            ["-%y-%m-%d-%H-%M-%S.%z", "-%y-%m-%d-%H-%M-%S.%f.%z"]
        } else {
            ["-%y-%m-%d-%H-%M-%S%z", "-%y-%m-%d-%H-%M-%S.%f%z"]
        };
        name.push_str("+0000");
        let result = chrono::DateTime::parse_from_str(name.as_str(), patterns[0])
            .or_else(|_| chrono::DateTime::parse_from_str(name.as_str(), patterns[1]))
            .ok()
            .map(|v| v.to_utc());
        result
    }
    fn build_file_name_with_suffix(writer: &RollingFileWriter, suffix: &String) -> String {
        if !writer.file_extension.is_empty() {
            format!(
                "{}{}{}.{}",
                &writer.file_path, &writer.file_name, &suffix, &writer.file_extension
            )
        } else {
            format!("{}{}{}", &writer.file_path, &writer.file_name, &suffix)
        }
    }
    fn build_file_name_with_suffix_and_num(
        writer: &RollingFileWriter,
        suffix: &String,
        num: usize,
    ) -> String {
        if !writer.file_extension.is_empty() {
            format!(
                "{}{}{}.{:03}.{}",
                &writer.file_path, &writer.file_name, &suffix, num, &writer.file_extension
            )
        } else {
            format!(
                "{}{}{:03}.{}",
                &writer.file_path, &writer.file_name, &suffix, num
            )
        }
    }
    fn get_new_file_name_if_exists(writer: &RollingFileWriter, suffix: &String) -> String {
        if let Ok(dir) = std::fs::read_dir(writer.file_path.as_str()) {
            let count = dir
                .filter(|v| match v {
                    Ok(entry) => {
                        if let Ok(meta) = entry.metadata() {
                            if let Some(path) = entry.path().to_str() {
                                return path.contains(suffix)
                                    && path.contains(writer.file_name.as_str())
                                    && path.contains(writer.file_extension.as_str())
                                    && meta.is_file();
                            }
                        }
                        false
                    }
                    Err(_) => false,
                })
                .count();
            Self::build_file_name_with_suffix_and_num(writer, suffix, count)
        } else {
            Self::build_file_name_with_suffix_and_num(writer, suffix, 0)
        }
    }
}

struct RollingStrategyImpl {
    strategy: RollingStrategy,
    pub(self) created: SystemTime,
    size: usize,
}

impl RollingStrategyImpl {
    /// Creates new [`RollingStrategyImpl`].
    pub fn new(interval: Option<Duration>, limit: Option<usize>) -> Self {
        let strategy = match (interval, limit) {
            (None, None) => RollingStrategy::None,
            (None, Some(limit)) => RollingStrategy::SizeBased(limit),
            (Some(interval), None) => RollingStrategy::TimeBased(interval),
            (Some(interval), Some(limit)) => RollingStrategy::Mixed(interval, limit),
        };
        Self {
            strategy,
            created: SystemTime::now(),
            size: 0,
        }
    }
    pub(self) fn should_rollover(&mut self, required_size: usize) -> bool {
        match self.strategy {
            RollingStrategy::None => false,
            RollingStrategy::TimeBased(interval) => {
                let now = SystemTime::now();
                if self.created.add(interval) < now {
                    self.created = now;
                    true
                } else {
                    false
                }
            }
            RollingStrategy::SizeBased(limit) => {
                self.size += required_size;
                if self.size > limit {
                    self.size = required_size;
                    true
                } else {
                    false
                }
            }
            RollingStrategy::Mixed(interval, limit) => {
                let now = SystemTime::now();
                self.size += required_size;
                if (self.created.add(interval) < now) || (self.size > limit) {
                    self.created = now;
                    self.size = required_size;
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RetentionPolicy {
    NoLimit,
    Count(usize),
    Age(Duration),
    Complex(usize, Duration),
}

#[derive(Debug)]
#[allow(clippy::type_complexity)]
struct RetentionPolicyImpl {
    policy: RetentionPolicy,
    queue: RwLock<LinkedList<(String, DateTime<Utc>, Option<String>)>>,
}

impl RetentionPolicyImpl {
    /// Creates nes [`RetentionPolicyImpl`].
    pub fn new(count: Option<usize>, age: Option<Duration>) -> Self {
        let policy = match (count, age) {
            (None, None) => RetentionPolicy::NoLimit,
            (None, Some(age)) => RetentionPolicy::Age(age),
            (Some(count), None) => RetentionPolicy::Count(count),
            (Some(count), Some(age)) => RetentionPolicy::Complex(count, age),
        };
        Self {
            policy,
            queue: RwLock::new(LinkedList::<_>::new()),
        }
    }
    fn delete_files(&mut self, files: Vec<String>) {
        for file in &files {
            let _ = std::fs::remove_file(file);
        }
    }
    fn process_by_time(&mut self, max_age: Duration) {
        let eldest_made = Utc::now().sub(max_age);
        let mut temp = Vec::<String>::new();
        {
            let queue = self.queue.get_mut().unwrap_or_else(PoisonError::into_inner);
            while !queue.is_empty() {
                if let Some(first) = queue.front() {
                    if first.1 > eldest_made {
                        break;
                    }
                }
                if let Some(mut first) = queue.pop_front() {
                    if let Some(add) = first.2.take() {
                        temp.push(add)
                    }
                    temp.push(first.0);
                } else {
                    break;
                }
            }
        }
        self.delete_files(temp);
    }
    fn process_by_size(&mut self, max_count: usize) {
        let mut temp = Vec::<String>::new();
        {
            let queue = self.queue.get_mut().unwrap_or_else(PoisonError::into_inner);
            while queue.len() > max_count {
                if let Some((name, _, add)) = queue.pop_front() {
                    temp.push(name);
                    if let Some(add) = add {
                        temp.push(add);
                    }
                }
            }
        }
        self.delete_files(temp);
    }
    fn process(&mut self) {
        match self.policy {
            RetentionPolicy::NoLimit => {}
            RetentionPolicy::Count(max_size) => self.process_by_size(max_size),
            RetentionPolicy::Age(duration) => self.process_by_time(duration),
            RetentionPolicy::Complex(max_size, duration) => {
                self.process_by_size(max_size);
                self.process_by_time(duration);
            }
        }
    }
    fn new_item(&mut self, file_name: String, date: DateTime<Utc>, add: Option<String>) {
        if self.policy != RetentionPolicy::NoLimit {
            {
                let queue = self.queue.get_mut().unwrap_or_else(PoisonError::into_inner);
                queue.push_back((file_name, date, add));
            }
            self.process();
        }
    }

    fn init(&mut self, log_path: &str, name: &str, ext: &str) {
        if self.policy == RetentionPolicy::NoLimit {
            return;
        }
        if let Ok(dir) = std::fs::read_dir(log_path) {
            let mut match_files = dir
                .filter_map(|v| match v {
                    Ok(entry) => {
                        if let Ok(meta) = entry.metadata() {
                            if let Some(path) = entry.path().to_str() {
                                if path.contains(name) && path.contains(ext) && meta.is_file() {
                                    if let Some(time) = FileNameUtils::get_time_from_name(
                                        path.replace(log_path, "")
                                            .replace(name, "")
                                            .replace(ext, "")
                                            .replace(".zip", ""),
                                    ) {
                                        return Some((path.to_owned(), time, None));
                                    }
                                }
                            }
                        }
                        None
                    }
                    Err(_) => None,
                })
                .collect::<Vec<_>>();
            match_files.sort_by(|a, b| b.1.cmp(&a.1));
            {
                let queue = self.queue.get_mut().unwrap_or_else(PoisonError::into_inner);
                while !match_files.is_empty() {
                    if let Some(entry) = match_files.pop() {
                        queue.push_back(entry);
                    }
                }
            }
            self.process();
        }
    }
}

#[derive(Default)]
struct ZipFiles {
    handle: Option<JoinHandle<()>>,
}
impl ZipFiles {
    pub(self) fn zip_file(&mut self, file_name: &str, dest: &str, path: &str) {
        let old_handle = self.handle.take();
        let file_name = file_name.to_owned();
        let dest = dest.to_owned();
        let path = path.to_owned();
        let handle = thread::spawn(move || {
            if let Some(old_handle) = old_handle {
                if !old_handle.is_finished() {
                    let _ = old_handle.join();
                }
            }
            let _ = Self::zip_file_impl(file_name, dest, path);
        });
        self.handle.replace(handle);
    }
    fn zip_file_impl(
        file_name: String,
        dest: String,
        path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let zip_file_path = std::path::Path::new(dest.as_str());
        let zip_file = File::create(zip_file_path)?;
        let mut zip = zip::ZipWriter::new(zip_file);
        zip.start_file(
            file_name.replace(path.as_str(), "").as_str(),
            zip::write::FileOptions::default(),
        )?;
        let buffer = std::fs::read(file_name.as_str())?;
        zip.write_all(buffer.as_slice())?;
        zip.finish()?;
        std::fs::remove_file(file_name)?;
        Ok(())
    }
}
