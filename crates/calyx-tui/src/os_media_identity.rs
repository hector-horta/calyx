//! Windows only: an app identity of calyx's own, so the media overlay says
//! "Calyx" with its icon instead of "Unknown application". The overlay names
//! a session by its AppUserModelID and, for an app outside the Store, looks
//! that id up on a Start Menu shortcut. So calyx sets the id for its process
//! and keeps `Start Menu\Programs\Calyx.lnk` (to this exe, with the id) up to date.

use std::path::{Path, PathBuf};
use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
use windows::Win32::System::Com::StructuredStorage::{PROPVARIANT, PropVariantClear};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
    IPersistFile,
};
use windows::Win32::System::Variant::VT_LPWSTR;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::UI::Shell::{
    IShellLinkW, SHStrDupW, SetCurrentProcessExplicitAppUserModelID, ShellLink,
};
use windows::core::{HSTRING, Interface};

/// calyx's AppUserModelID; readers of the media sessions see it as the app id.
pub const APP_ID: &str = "Calyx.TerminalPlayer";

/// `Start Menu\Programs\Calyx.lnk` of the current user.
fn shortcut_path() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    Some(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Calyx.lnk"))
}

/// Gives this process calyx's identity (call before the media controls are
/// created) and refreshes the shortcut on a background thread, once per run.
/// Best effort: on failure the overlay just keeps the generic name.
pub fn register() {
    // Once per run: the controls are created again for each listening session.
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(register_now);
}

fn register_now() {
    // SAFETY: a static, NUL-terminated id string.
    if let Err(e) = unsafe { SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(APP_ID)) } {
        tracing::debug!(error = %e, "could not set the app id");
    }
    let (Ok(exe), Some(lnk)) = (std::env::current_exe(), shortcut_path()) else {
        return;
    };
    std::thread::spawn(move || {
        if let Err(e) = write_shortcut(&exe, &lnk) {
            tracing::debug!(error = %e, "could not write the Start Menu shortcut");
        }
    });
}

/// Writes the shortcut `lnk` to `exe`, stamped with `System.AppUserModel.ID`.
fn write_shortcut(exe: &Path, lnk: &Path) -> windows::core::Result<()> {
    // SAFETY: COM calls on this thread's own apartment; every pointer passed
    // lives until the call returns, and the string is freed by PropVariantClear.
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        let result = (|| -> windows::core::Result<()> {
            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
            let target = HSTRING::from(exe.as_os_str());
            link.SetPath(&target)?;
            link.SetIconLocation(&target, 0)?;

            let store: IPropertyStore = link.cast()?;
            let mut value = PROPVARIANT::default();
            (*value.Anonymous.Anonymous).vt = VT_LPWSTR;
            (*value.Anonymous.Anonymous).Anonymous.pwszVal = SHStrDupW(&HSTRING::from(APP_ID))?;

            let set = store.SetValue(&PKEY_AppUserModel_ID, &value);
            PropVariantClear(&mut value)?;
            set?;
            store.Commit()?;
            link.cast::<IPersistFile>()?
                .Save(&HSTRING::from(lnk.as_os_str()), true)
        })();

        CoUninitialize();
        result
    }
}
