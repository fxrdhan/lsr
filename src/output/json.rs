// SPDX-FileCopyrightText: 2026 Christina Sørensen
// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: 2023-2026 Christina Sørensen, eza contributors
// SPDX-FileCopyrightText: 2014 Benjamin Sago
// SPDX-License-Identifier: MIT
use std::{
    io::{self, Write},
    unimplemented,
};

use crate::fs::{Dir, DotFilter, File, feature::git::GitCache};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Options {
    pub long: bool,
}

pub struct Render<'a> {
    pub git: Option<&'a GitCache>,

    pub deref_links: bool,
    pub total_size: bool,
    pub git_ignoring: bool,

    pub dots: DotFilter,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(
        &self,
        files: Vec<File<'a>>,
        mut dirs: Vec<Dir>,
        w: &mut W,
    ) -> io::Result<()> {
        match (files.len(), dirs.len()) {
            (0, 1) => {
                let dir = dirs.get_mut(0).unwrap();
                self.render_directory(dir, w)
            }
            (_, 0) => self.render_files(files, w),
            (0, _) => self.render_directories(dirs, w),
            (_, _) => self.render_files_directories(files, dirs, w),
        }?;
        Ok(())
    }

    fn render_files<W: Write>(&self, files: Vec<File<'a>>, w: &mut W) -> io::Result<()> {
        let fnames: Vec<String> = files.iter().map(|f| self.render_file(f)).collect();
        write!(w, "[{}]", fnames.join(","))?;
        Ok(())
    }

    fn render_directory<W: Write>(&self, dir: &'a mut Dir, w: &mut W) -> io::Result<()> {
        let dir = dir.read()?;
        let files: Vec<File<'a>> = dir
            .files(
                self.dots,
                self.git,
                self.git_ignoring,
                self.deref_links,
                self.total_size,
            )
            .collect();

        self.render_files(files, w)?;
        Ok(())
    }

    fn render_directories<W: Write>(&self, dirs: Vec<Dir>, w: &mut W) -> io::Result<()> {
        write!(w, "{{")?;
        let mut first = true;
        for mut dir in dirs {
            if first {
                first = false;
            } else {
                write!(w, ",")?;
            }
            write!(w, "\"{}\":", dir.path.display().to_string())?;
            self.render_directory(&mut dir, w)?;
        }
        write!(w, "}}")?;
        Ok(())
    }

    fn render_files_directories<W: Write>(
        &self,
        files: Vec<File<'a>>,
        dirs: Vec<Dir>,
        w: &mut W,
    ) -> io::Result<()> {
        write!(w, "{{\"files\":")?;
        self.render_files(files, w)?;
        write!(w, ", \"directories\":")?;
        self.render_directories(dirs, w)?;
        write!(w, "}}")?;
        Ok(())
    }

    fn render_file(&self, f: &File<'a>) -> String {
        return format!("\"{}\"", f.name);
    }
}
