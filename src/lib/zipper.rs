// Copyright (C) 2016-2022 Élisabeth HENRY.
//
// This file is part of Crowbook.
//
// Crowbook is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published
// by the Free Software Foundation, either version 2.1 of the License, or
// (at your option) any later version.
//
// Caribon is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received ba copy of the GNU Lesser General Public License
// along with Crowbook.  If not, see <http://www.gnu.org/licenses/>.

use crate::error::{Error, Result};

use std::fs::{self, DirBuilder, File};
use std::io;
use std::io::Write;
use std::ops::Drop;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Struct used to create zip (using filesystem and zip command)
pub struct Zipper {
    args: Vec<String>,
    path: PathBuf,
}

impl Zipper {
    /// Creates new zipper
    ///
    /// # Arguments
    /// * `path`: the path to a temporary directory
    /// (zipper will create a random dir in it and clean it later)
    pub fn new(path: &str) -> Result<Zipper> {
        let uuid = uuid::Uuid::new_v4();
        let zipper_path = Path::new(path).join(uuid.as_simple().to_string());

        DirBuilder::new()
            .recursive(true)
            .create(&zipper_path)
            .map_err(|_| {
                Error::zipper(lformat!(
                    "could not create temporary directory in {path}",
                    path = path
                ))
            })?;

        Ok(Zipper {
            args: vec![],
            path: zipper_path,
        })
    }

    /// writes a content to a temporary file
    pub fn write<P: AsRef<Path>>(&mut self, path: P, content: &[u8], add_args: bool) -> Result<()> {
        let path = path.as_ref();
        let file = format!("{}", path.display());
        if path.starts_with("..") || path.is_absolute() {
            return Err(Error::zipper(lformat!(
                "file {file} refers to an absolute or a parent \
                                               path.
This is forbidden because we are supposed \
                                               to create a temporary file in a temporary dir.",
                file = file
            )));
        }
        let dest_file = self.path.join(path);
        let dest_dir = dest_file.parent().unwrap();
        if fs::metadata(dest_dir).is_err() {
            // dir does not exist, create it
            DirBuilder::new()
                .recursive(true)
                .create(&dest_dir)
                .map_err(|_| {
                    Error::zipper(lformat!(
                        "could not create temporary directory in {path}",
                        path = dest_dir.display()
                    ))
                })?;
        }

        if let Ok(mut f) = File::create(&dest_file) {
            if f.write_all(content).is_ok() {
                if add_args {
                    self.args.push(file);
                }
                Ok(())
            } else {
                Err(Error::zipper(lformat!(
                    "could not write to temporary file {file}",
                    file = file
                )))
            }
        } else {
            Err(Error::zipper(lformat!(
                "could not create temporary file {file}",
                file = file
            )))
        }
    }

    /// Unzip a file and deletes it afterwards
    #[cfg(feature = "odt")]
    pub fn unzip(&mut self, file: &str) -> Result<()> {
        let output = Command::new("unzip")
            .current_dir(&self.path)
            .arg(file)
            .output()
            .map_err(|e| {
                Error::zipper(lformat!(
                    "failed to execute unzip on {file}: {error}",
                    file = file,
                    error = e
                ))
            });

        output?;

        fs::remove_file(self.path.join(file))
            .map_err(|_| Error::zipper(lformat!("failed to remove file {file}", file = file)))
    }

    /// run command and copy content of file output (supposed to result from the command) to current dir
    pub fn run_command(
        &mut self,
        mut command: Command,
        command_name: &str,
        in_file: &str,
        out: &mut dyn Write,
    ) -> Result<String> {
        let res_output = command.output().map_err(|e| {
            debug!(
                "{}",
                lformat!(
                    "output for command {name}:\n{error}",
                    name = command_name,
                    error = e
                )
            );
            Error::zipper(lformat!(
                "failed to run command '{name}'",
                name = command_name
            ))
        });
        let output = res_output?;
        if output.status.success() {
            let mut file = File::open(self.path.join(in_file)).map_err(|_| {
                debug!(
                    "{}",
                    lformat!(
                        "could not open result of command '{command}'\n\
                                           Command output:\n\
                                           {output}'",
                        command = command_name,
                        output = String::from_utf8_lossy(&output.stderr)
                    )
                );
                Error::zipper(lformat!(
                    "could not open result of command '{command}'",
                    command = command_name
                ))
            })?;
            io::copy(&mut file, out).map_err(|_| {
                Error::zipper(lformat!("error copying file '{file}'", file = in_file))
            })?;

            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            debug!(
                "{}",
                lformat!(
                    "{command} didn't return succesfully: {output}",
                    command = command_name,
                    output = String::from_utf8_lossy(&output.stderr)
                )
            );
            Err(Error::zipper(lformat!(
                "{command} didn't return succesfully",
                command = command_name
            )))
        }
    }

    /// zip all files in zipper's tmp dir to a given file name and write to odt file
    #[cfg(feature = "odt")]
    pub fn generate_odt(&mut self, command_name: &str, odt_file: &mut dyn Write) -> Result<String> {
        let mut command = Command::new(command_name);
        command.current_dir(&self.path);
        command.arg("-r");
        command.arg("result.odt");
        command.arg(".");
        self.run_command(command, command_name, "result.odt", odt_file)
    }

    /// generate a pdf file into given file name
    pub fn generate_pdf(
        &mut self,
        command_name: &str,
        tex_file: &str,
        pdf_file: &mut dyn Write,
    ) -> Result<String> {
        // first pass
        let mut command = Command::new(command_name);
        command.current_dir(&self.path).arg(tex_file);
        let _ = command.output();

        // second pass
        let _ = command.output();

        // third pass
        // let mut command = Command::new(command_name);
        // command.current_dir(&self.path);
        // command.arg(tex_file);
        self.run_command(command, command_name, "result.pdf", pdf_file)
    }
}

impl Drop for Zipper {
    fn drop(&mut self) {
        if let Err(err) = fs::remove_dir_all(&self.path) {
            println!(
                "Error in zipper: could not delete temporary directory {}, error: {}",
                self.path.to_string_lossy(),
                err
            );
        }
    }
}
