//! Endpoint-local regular-file admission. Reads run only on the retained source worker.
use crate::{AuthError, SourceControl, SourceFailure};
use std::{io::Read, path::Path};

#[cfg(windows)]
mod native {
    pub(super) const WINDOWS_CNG: bool = true;
}
#[cfg(not(windows))]
mod native {
    pub(super) const WINDOWS_CNG: bool = false;
}

// Existing regular-file route: keep FIFO open nonblocking on qualified Unix
// targets. Constants are the Darwin SDK and Linux x86_64/aarch64 ABI values;
// no OS handle, libc dependency or unsafe code enters the public surface.
#[cfg(target_os = "macos")]
mod native_open {
    use std::{
        fs::{File, OpenOptions},
        io,
        os::unix::fs::OpenOptionsExt,
        path::Path,
    };
    pub(super) fn open(path: &Path) -> io::Result<File> {
        OpenOptions::new().read(true).custom_flags(0x4).open(path)
    }
}
#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
mod native_open {
    use std::{
        fs::{File, OpenOptions},
        io,
        os::unix::fs::OpenOptionsExt,
        path::Path,
    };
    pub(super) fn open(path: &Path) -> io::Result<File> {
        OpenOptions::new().read(true).custom_flags(0x800).open(path)
    }
}
#[cfg(not(any(
    target_os = "macos",
    all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )
)))]
mod native_open {
    use std::{fs::File, io, path::Path};
    pub(super) fn open(path: &Path) -> io::Result<File> {
        File::open(path)
    }
}

pub(crate) fn read(
    path: &Path,
    buffer: &mut [u8],
    control: &SourceControl,
) -> Result<usize, AuthError> {
    control.check()?;
    if native::WINDOWS_CNG && refuses_device_path(&path.to_string_lossy()) {
        return Err(AuthError::Source(SourceFailure::NonRegular));
    }
    // Both checks matter: the pathname may be replaced before open. A blocked
    // filesystem/open remains owned by the registered worker, never by a poll.
    if !std::fs::metadata(path).map_err(clean)?.is_file() {
        return Err(AuthError::Source(SourceFailure::NonRegular));
    }
    control.check()?;
    let mut file = native_open::open(path).map_err(clean)?;
    if !file.metadata().map_err(clean)?.is_file() {
        return Err(AuthError::Source(SourceFailure::NonRegular));
    }
    let mut used = 0;
    while used < buffer.len() {
        control.check()?;
        let end = (used + 8192).min(buffer.len());
        let count = file.read(&mut buffer[used..end]).map_err(clean)?;
        control.check()?;
        if count == 0 {
            break;
        }
        used += count;
    }
    Ok(used)
}
fn clean(error: std::io::Error) -> AuthError {
    AuthError::Source(match error.kind() {
        std::io::ErrorKind::NotFound => SourceFailure::NotFound,
        std::io::ErrorKind::PermissionDenied => SourceFailure::AccessDenied,
        _ => SourceFailure::Io,
    })
}
pub(crate) fn check_native_form(text: &str) -> Result<(), AuthError> {
    if native::WINDOWS_CNG && !cng_reads(text) {
        return Err(AuthError::Source(SourceFailure::UnsupportedKeyForm));
    }
    Ok(())
}
// Exact existing WinCNG framing limit; compiled everywhere for negative fixtures.
pub(crate) fn cng_reads(text: &str) -> bool {
    const BEGIN: &str = "-----BEGIN RSA PRIVATE KEY-----";
    const END: &str = "-----END RSA PRIVATE KEY-----";
    let mut lines = text.split(['\r', '\n']);
    if lines.clone().any(|line| line.len() > 126) {
        return false;
    }
    if lines.find(|line| !line.trim().is_empty()) != Some(BEGIN) {
        return false;
    }
    for line in lines {
        if line.starts_with("-----BEGIN ") {
            return false;
        }
        if line.starts_with("-----END ") {
            return line == END;
        }
    }
    false
}
fn separator_at(bytes: &[u8], at: usize) -> bool {
    matches!(bytes.get(at), Some(b'\\' | b'/'))
}

/// The text after the Win32 namespace prefix `\\<marker>\`, in any of its eight separator spellings.
fn after_namespace(text: &str, marker: u8) -> Option<&str> {
    let bytes = text.as_bytes();
    (separator_at(bytes, 0)
        && separator_at(bytes, 1)
        && bytes.get(2) == Some(&marker)
        && separator_at(bytes, 3))
    .then(|| &text[4..])
}

/// Whether `text` starts with the NT namespace prefix `\??\`, in either separator.
fn in_nt_namespace(text: &str) -> bool {
    let bytes = text.as_bytes();
    separator_at(bytes, 0) && bytes.get(1..3) == Some(b"??") && separator_at(bytes, 3)
}

/// Whether `text`, a path, names a device or a pipe by its spelling: Windows opens `CON` and `\\.\pipe\x` as
/// devices, and some of them block. Compiled everywhere so that every platform's test run covers the table that
/// only the Windows arm of [`open`] consults.
fn refuses_device_path(text: &str) -> bool {
    let is_separator = |c: char| c == '\\' || c == '/';
    // The NT namespace is never a path a user writes, and the Win32 device namespace holds `\\.\NUL`,
    // `\\.\pipe\x` and `\\.\C:`.
    if in_nt_namespace(text) || after_namespace(text, b'.').is_some() {
        return true;
    }
    let mut rest = text;
    // The extended-length namespace names files as well as devices: only a drive or a UNC path is a file.
    if let Some(after) = after_namespace(text, b'?') {
        let bytes = after.as_bytes();
        let drive = bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && separator_at(bytes, 2);
        let unc =
            bytes.len() >= 4 && bytes[..3].eq_ignore_ascii_case(b"UNC") && separator_at(bytes, 3);
        if !drive && !unc {
            return true;
        }
        rest = after;
    }
    rest.split(is_separator)
        .enumerate()
        .any(|(index, component)| {
            // A leading drive, `C:` or the drive-relative `C:NUL`, is not a colon in a name. Any other colon names an
            // alternate data stream or a device (`NUL:`).
            let name = match component.as_bytes() {
                [letter, b':', ..] if index == 0 && letter.is_ascii_alphabetic() => &component[2..],
                _ => component,
            };
            name.contains(':') || reserved_name(name)
        })
}

/// Whether the file name `name` is one of the reserved DOS device names, with or without an extension.
fn reserved_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("").trim_end_matches(' ');
    let upper = stem.to_uppercase();
    match upper.as_str() {
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$" => return true,
        _ => {}
    }
    // COM1 to COM9 and LPT1 to LPT9, with the superscript digits Windows also reserves.
    let digits = [
        "1", "2", "3", "4", "5", "6", "7", "8", "9", "\u{b9}", "\u{b2}", "\u{b3}",
    ];
    ["COM", "LPT"].iter().any(|device| {
        upper
            .strip_prefix(device)
            .is_some_and(|number| digits.contains(&number))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_cng_guard_checks_the_exact_native_reader_forms_on_every_platform() {
        let rsa = "-----BEGIN RSA PRIVATE KEY-----\nAQID\n-----END RSA PRIVATE KEY-----\n";
        assert!(cng_reads(rsa));
        assert!(cng_reads(&rsa.replace('\n', "\r\n")));
        for malformed in [
            "-----BEGIN PRIVATE KEY-----\nAQID\n-----END PRIVATE KEY-----\n".to_owned(),
            "-----BEGIN OPENSSH PRIVATE KEY-----\nAQID\n-----END OPENSSH PRIVATE KEY-----\n"
                .to_owned(),
            format!("preamble\n{rsa}"),
            rsa.replace("AQID", &"A".repeat(127)),
            rsa.replace(
                "-----END RSA PRIVATE KEY-----",
                "-----END EC PRIVATE KEY-----",
            ),
            rsa.replace("AQID", "-----BEGIN EC PRIVATE KEY-----"),
        ] {
            assert!(!cng_reads(&malformed));
        }
    }
    #[test]
    fn windows_device_and_namespace_screen_runs_on_every_platform() {
        for name in [
            "CON",
            "nul.txt",
            "C:NUL",
            "COM1",
            "LPT9",
            "\\\\.\\pipe\\x",
            "//./pipe/x",
            "\\??\\C:\\x",
            "\\\\?\\GLOBALROOT\\Device\\Null",
            "file:stream",
        ] {
            assert!(refuses_device_path(name), "{name}");
        }
        for name in [
            "key",
            "C:\\keys\\native key é",
            "\\\\?\\C:\\keys\\key",
            "\\\\?\\UNC\\host\\share\\key",
            "COM10",
            "CONSOLE",
        ] {
            assert!(!refuses_device_path(name), "{name}");
        }
    }
}

#[cfg(all(
    test,
    any(
        target_os = "macos",
        all(
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64")
        )
    )
))]
mod nonblocking_tests {
    use super::*;
    use std::{
        fs,
        process::Command,
        sync::mpsc,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    #[test]
    fn fifo_replacement_cannot_block_the_qualified_native_open() {
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/tmp")
            .join(format!("native-open-{}-{tick}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let fifo = directory.join("fifo");
        assert!(
            Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        let (tx, rx) = mpsc::channel();
        let reader_path = fifo.clone();
        let worker = std::thread::spawn(move || {
            let result = native_open::open(&reader_path);
            tx.send(result).unwrap();
        });
        let first = rx.recv_timeout(Duration::from_secs(5));
        let prompt = first.is_ok();
        let opened = match first {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // If a regression blocked read-open, this writer releases it.
                // Always join the owned helper before asserting the failure.
                drop(fs::OpenOptions::new().write(true).open(&fifo).unwrap());
                rx.recv_timeout(Duration::from_secs(5)).unwrap()
            }
            Err(error) => {
                panic!("source-open helper failed: {error}");
            }
        };
        worker.join().unwrap();
        assert!(!opened.as_ref().unwrap().metadata().unwrap().is_file());
        drop(opened);
        fs::remove_dir_all(directory).unwrap();
        assert!(prompt, "native open waited for a FIFO writer");
    }
}
