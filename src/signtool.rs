use std::path::{Path, PathBuf};

use bitness::{self, Bitness};
use winreg::RegKey;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY};

use ::*;

pub struct SignTool {
    signtool_path: PathBuf,
}

impl SignTool {
    pub fn locate_latest() -> CodeSignResult<SignTool> {
        Ok(SignTool {
            signtool_path: locate_signtool()?,
        })
    }

    pub fn sign<P: AsRef<Path>>(&self, path: P, params: &SignParams) -> CodeSignResult<()> {
        use std::process::Command;

        // Convert path to string reference, as we need to pass it as a commandline parameter to signtool
        let path_str = path.as_ref().to_str().unwrap();

        // Construct SignTool command
        let mut cmd = Command::new(&self.signtool_path);
        cmd.arg("sign");
        cmd.args(&["/fd", &params.digest_algorithm]);
        cmd.args(&["/sha1", &params.certificate_thumbprint]);

        if let Some(ref timestamp_url) = params.timestamp_url {
            cmd.args(&["/t", timestamp_url]);
        }

        cmd.arg(path_str);

        debug!("Executing SignTool command: {:?}", cmd);

        // Execute SignTool command
        let output = cmd.output()?;

        debug!("Output: {:?}", &output);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
            error!("{}", &stderr);

            Err(CodeSignError::SignToolError {
                exit_code: output.status.code().unwrap_or(-1),
                stderr: stderr,
            })?;
        }

        // We good.
        Ok(())
    }
}

fn locate_signtool() -> CodeSignResult<PathBuf> {
    const INSTALLED_ROOTS_REGKEY_PATH: &str = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
    const KITS_ROOT_REGVALUE_NAME: &str = r"KitsRoot10";

    let installed_roots_key_path = Path::new(INSTALLED_ROOTS_REGKEY_PATH);

    // Open 32-bit HKLM "Installed Roots" key
    let installed_roots_key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(
            installed_roots_key_path,
            KEY_READ | KEY_WOW64_32KEY
        ).map_err(|_| format!("Error opening registry key: {}", INSTALLED_ROOTS_REGKEY_PATH))?;

    // Get the Windows SDK root path
    let kits_root_10_path: String = installed_roots_key.get_value(KITS_ROOT_REGVALUE_NAME)
        .map_err(|_| format!("Error getting {} value from registry!", KITS_ROOT_REGVALUE_NAME))?;

    // Construct Windows SDK bin path
    let kits_root_10_bin_path = Path::new(&kits_root_10_path).join("bin");

    let mut installed_kits: Vec<String> = installed_roots_key.enum_keys()
        /* Report and ignore errors, pass on values. */
        .filter_map(|res| match res {
            Ok(v) => Some(v),
            Err(err) => {
                error!("Error enumerating installed root keys: {}", err.to_string());
                None
            }
        })
        .inspect(|kit| debug!("Found installed kit: {}", kit))
        .collect();

    // Sort installed kits
    installed_kits.sort();

    /* Iterate through installed kit version keys in reverse (from newest to oldest),
       adding their bin paths to the list.
       Windows SDK 10 v10.0.15063.468 and later will have their signtools located there. */
    let mut kit_bin_paths: Vec<PathBuf> = installed_kits.iter().rev()
        .map(|kit| kits_root_10_bin_path.join(kit).to_path_buf())
        .collect();

    /* Add kits root bin path.
       For Windows SDK 10 versions earlier than v10.0.15063.468, signtool will be located there. */
    kit_bin_paths.push(kits_root_10_bin_path.to_path_buf());

    // Choose which version of SignTool to use based on OS bitness
    let arch_dir = match bitness::os_bitness()? {
        Bitness::X86_32 => "x86",
        Bitness::X86_64 => "x64",
        _ => Err("Unsupported OS!".to_owned())?
    };

    /* Iterate through all bin paths, checking for existence of a SignTool executable. */
    for kit_bin_path in &kit_bin_paths {
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
    Err("No SignTool found!".to_owned())?
}
