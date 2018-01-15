extern crate bitness;
#[macro_use] extern crate log;
extern crate winreg;

mod signtool;

use std::path::{Path, PathBuf};

pub struct CodeSign {
    signtool_path: PathBuf,
    certificate_thumbprint: String,
    timestamp_url: String,
}

impl CodeSign {
    pub fn new(certificate_thumbprint: &str, timestamp_url: &str) -> Result<Self, &'static str> {
        Ok(Self {
            signtool_path: signtool::locate_signtool()?,
            certificate_thumbprint: certificate_thumbprint.to_owned(),
            timestamp_url: timestamp_url.to_owned(),
        })
    }

    pub fn sign<P: AsRef<Path>>(&self, path: P) -> Result<(), &'static str> {
        use std::process::Command;

        // Convert path to string reference, as we need to pass it as a commandline parameter to signtool
        let path_str = path.as_ref().to_str().unwrap();

        // Construct SignTool command
        let mut cmd = Command::new(&self.signtool_path);
        cmd.arg("sign");
        cmd.args(&["/fd", "sha256"]);
        cmd.args(&["/sha1", &self.certificate_thumbprint]);
        cmd.args(&["/t", &self.timestamp_url]);
        cmd.arg(path_str);

        debug!("Executing SignTool command: {:?}", cmd);

        // Execute SignTool command
        let output = cmd.output().map_err(|_| "Error executing SignTool!")?;

        debug!("Output: {:?}", &output);

        if !output.status.success() {
            error!("{}", String::from_utf8(output.stderr).unwrap());
            return Err("Error signing file");
        }

        // We good.
        Ok(())
    }
}
