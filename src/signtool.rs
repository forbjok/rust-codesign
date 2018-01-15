use std::path::{Path, PathBuf};

use bitness::{self, Bitness};
use winreg::RegKey;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY};

pub fn locate_signtool() -> Result<PathBuf, &'static str> {
    let installed_roots_key_path = Path::new(r"SOFTWARE\Microsoft\Windows Kits\Installed Roots");
    debug!("Opening 'Installed Roots' key: {:?}", installed_roots_key_path);

    // Open 32-bit HKLM "Installed Roots" key
    let installed_roots_key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(
            installed_roots_key_path,
            KEY_READ | KEY_WOW64_32KEY
        ).map_err(|_| "Error opening Software registry key!")?;

    debug!("Getting Windows 10 Kits root path.");
    let kits_root_10_path: String = installed_roots_key.get_value("KitsRoot10").map_err(|_| "Error getting Windows 10 Kits root path!")?;
    debug!("Windows 10 Kits root path found: {}", &kits_root_10_path);

    let kits_root_10_bin_path = Path::new(&kits_root_10_path).join("bin");

    let mut installed_kits: Vec<String> = Vec::new();
    let mut kit_bin_paths: Vec<PathBuf> = Vec::new();

    for k in installed_roots_key.enum_keys() {
        let kit = k.map_err(|_| "No kit!")?;

        debug!("Found installed kit: {}", kit);
        installed_kits.push(kit);
    }

    // Sort installed kits
    installed_kits.sort();

    /* Iterate through installed kit version keys in reverse (from newest to oldest),
       adding their bin paths to the list.
       Windows SDK 10 v10.0.15063.468 and later will have their signtools located there. */
    for installed_kit in installed_kits.iter().rev() {
        let kit_bin_path = kits_root_10_bin_path.join(&installed_kit);

        kit_bin_paths.push(kit_bin_path.to_path_buf());
    }

    /* Add kits root bin path. For Windows SDK 10 versions earlier than v10.0.15063.468, signtool will be located there. */
    kit_bin_paths.push(kits_root_10_bin_path.to_path_buf());

    // Choose which version of SignTool to use based on OS bitness
    let arch_dir = match bitness::os_bitness() {
        Bitness::X86_32 => "x86",
        Bitness::X86_64 => "x64",
        _ => panic!("Unsupported OS!")
    };

    /* Iterate through all bin paths, checking for existence of a SignTool executable. */
    for kit_bin_path in kit_bin_paths.iter() {
        /* Construct SignTool path. */
        let signtool_path = kit_bin_path.join(arch_dir).join("signtool.exe");

        /* Check if SignTool exists at this location. */
        if signtool_path.exists() {
            info!("SignTool found at: {:?}", signtool_path);

            // SignTool found. Return it.
            return Ok(signtool_path.to_path_buf());
        }
    }

    error!("No SignTool found!");
    Err("No SignTool found!")
}
