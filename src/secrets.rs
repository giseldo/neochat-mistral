use crate::storage::{data_dir, SECRET_FILE};
#[cfg(not(windows))]
use anyhow::bail;
#[cfg(unix)]
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
const KEYRING_SERVICE: &str = "com.neochat.api-key";
#[cfg(unix)]
const KEYRING_USERNAME: &str = "default";

pub fn secret_path() -> PathBuf {
    data_dir().join(SECRET_FILE)
}

#[cfg(windows)]
pub fn protect(plain: &[u8]) -> Result<Vec<u8>> {
    win::crypt(plain, true)
}

#[cfg(windows)]
pub fn unprotect(cipher: &[u8]) -> Result<Vec<u8>> {
    win::crypt(cipher, false)
}

#[cfg(not(windows))]
pub fn protect(plain: &[u8]) -> Result<Vec<u8>> {
    let _ = plain;
    bail!("o armazenamento protegido usa o cofre de credenciais do sistema")
}

#[cfg(not(windows))]
pub fn unprotect(cipher: &[u8]) -> Result<Vec<u8>> {
    let _ = cipher;
    bail!("o armazenamento protegido usa o cofre de credenciais do sistema")
}

pub fn save_api_key(key: &str) -> Result<()> {
    #[cfg(windows)]
    return save_windows(key);

    #[cfg(unix)]
    return save_keyring(key);

    #[cfg(all(not(windows), not(unix)))]
    {
        let _ = key;
        bail!("esta plataforma não possui armazenamento seguro configurado");
    }
}

#[cfg(windows)]
fn save_windows(key: &str) -> Result<()> {
    let path = secret_path();
    if key.trim().is_empty() {
        if path.exists() {
            fs::remove_file(&path)?;
        }
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, protect(key.trim().as_bytes())?)?;
    Ok(())
}

#[cfg(unix)]
fn keyring_entry() -> Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|error| anyhow::anyhow!("não foi possível acessar o Secret Service: {error}"))
}

#[cfg(unix)]
fn remove_legacy_file() -> Result<()> {
    match fs::remove_file(secret_path()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("não foi possível remover o segredo legado"),
    }
}

fn normalize_secret(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

#[cfg(unix)]
fn parse_legacy_secret(bytes: &[u8]) -> Option<String> {
    normalize_secret(std::str::from_utf8(bytes).ok()?)
}

#[cfg(unix)]
fn read_legacy_secret() -> Option<String> {
    parse_legacy_secret(&fs::read(secret_path()).ok()?)
}

#[cfg(unix)]
fn migrate_legacy_secret() -> Result<Option<String>> {
    let Some(key) = read_legacy_secret() else {
        return Ok(None);
    };

    let entry = keyring_entry()?;
    entry.set_password(&key).map_err(|error| {
        anyhow::anyhow!("não foi possível migrar o segredo para o Secret Service: {error}")
    })?;
    remove_legacy_file()?;
    Ok(Some(key))
}

#[cfg(unix)]
fn save_keyring(key: &str) -> Result<()> {
    let entry = keyring_entry()?;
    if key.trim().is_empty() {
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => remove_legacy_file(),
            Err(error) => Err(anyhow::anyhow!(
                "não foi possível remover a credencial do Secret Service: {error}"
            )),
        }
    } else {
        entry.set_password(key.trim()).map_err(|error| {
            anyhow::anyhow!("não foi possível salvar a credencial no Secret Service: {error}")
        })?;
        remove_legacy_file()
    }
}

#[cfg(unix)]
fn load_keyring() -> Option<String> {
    let entry = keyring_entry().ok()?;
    match entry.get_password() {
        Ok(key) => normalize_secret(&key),
        Err(keyring::Error::NoEntry) => migrate_legacy_secret().ok().flatten(),
        Err(_) => None,
    }
}

pub fn clear_api_key() -> Result<()> {
    save_api_key("")
}

pub fn load_api_key() -> Option<String> {
    #[cfg(windows)]
    {
        let bytes = fs::read(secret_path()).ok()?;
        let plain = unprotect(&bytes).ok()?;
        normalize_secret(std::str::from_utf8(&plain).ok()?)
    }

    #[cfg(unix)]
    return load_keyring();

    #[cfg(all(not(windows), not(unix)))]
    None
}

#[cfg(windows)]
mod win {
    use anyhow::{bail, Result};
    use std::ffi::c_void;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    pub fn crypt(data: &[u8], encrypt: bool) -> Result<Vec<u8>> {
        let mut input = data.to_vec();
        let blob_in = CRYPT_INTEGER_BLOB {
            cbData: input.len() as u32,
            pbData: input.as_mut_ptr(),
        };
        let mut blob_out = CRYPT_INTEGER_BLOB::default();

        let ok = unsafe {
            if encrypt {
                CryptProtectData(
                    &blob_in,
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut blob_out,
                )
            } else {
                CryptUnprotectData(
                    &blob_in,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut blob_out,
                )
            }
        };

        if ok == 0 {
            bail!("DPAPI falhou: {}", std::io::Error::last_os_error());
        }

        let result = if blob_out.pbData.is_null() {
            Vec::new()
        } else {
            unsafe {
                std::slice::from_raw_parts(blob_out.pbData, blob_out.cbData as usize).to_vec()
            }
        };
        unsafe {
            LocalFree(blob_out.pbData as *mut c_void);
        }
        Ok(result)
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn dpapi_roundtrip() {
        let plain = b"chave-secreta-da-mistral";
        let cipher = protect(plain).unwrap();
        assert_ne!(cipher, plain);
        let back = unprotect(&cipher).unwrap();
        assert_eq!(back, plain);
    }

    #[test]
    fn unprotect_rejects_garbage() {
        assert!(unprotect(b"nao-e-dpapi").is_err());
    }
}

#[cfg(all(test, unix))]
mod unix_tests {
    use super::*;

    #[test]
    fn legacy_secret_is_normalized_before_migration() {
        assert_eq!(
            parse_legacy_secret(b"  sk-test-secret \r\n"),
            Some("sk-test-secret".to_owned())
        );
    }

    #[test]
    fn malformed_or_empty_legacy_secret_is_rejected() {
        assert_eq!(parse_legacy_secret(b""), None);
        assert_eq!(parse_legacy_secret(b" \n\t"), None);
        assert_eq!(parse_legacy_secret(&[0xff, 0xfe]), None);
    }

    #[test]
    fn non_windows_protect_never_returns_plaintext() {
        assert!(protect(b"sk-test-secret").is_err());
        assert!(unprotect(b"sk-test-secret").is_err());
    }
}
