// SPDX-FileCopyrightText: 2026 Christina Sørensen
// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: 2023-2026 Christina Sørensen, eza contributors
// SPDX-FileCopyrightText: 2014 Benjamin Sago
// SPDX-License-Identifier: MIT
use std::{
    io::{self, Write},
    iter::Map,
    unimplemented,
};

use crate::{
    fs::{Dir, DotFilter, File, feature::git::GitCache},
    output::{details, table::Column},
};

#[derive(PartialEq, Eq, Debug)]
pub struct Options {
    pub details: Option<details::Options>,
}

pub struct Render<'a> {
    pub git: Option<&'a GitCache>,

    pub deref_links: bool,
    pub total_size: bool,
    pub git_ignoring: bool,

    pub dots: DotFilter,
    pub opts: &'a Options,
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
        match &self.opts.details {
            None => {
                let fnames: Vec<String> = files.iter().map(|f| self.render_file(f)).collect();
                write!(w, "[{}]", fnames.join(","))?;
            }
            Some(_) => {
                let fnames: Vec<String> = files
                    .iter()
                    .map(|f| format!("\"{}\":{}", f.name, self.render_file(f)))
                    .collect();
                write!(w, "{{{}}}", fnames.join(","))?;
            }
        }
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
        return match &self.opts.details {
            None => format!("\"{}\"", f.name),
            Some(o) => self.render_file_long(f, o),
        };
    }

    fn render_file_long(&self, f: &File<'a>, o: &details::Options) -> String {
        let fobj = JsonFileObject::create_for_for_file(
            f,
            o.table.as_ref().unwrap().columns.collect(false, false),
        );

        fobj.render()
    }
}

struct JsonFileObject {
    /// Reusing the table column to map everything we want to be displayed
    internal: Vec<(Column, String)>,
}

impl JsonFileObject {
    /// Render a json object with the columns in the map
    fn render(self) -> String {
        self.internal
            .iter()
            .map(|(c, v)| format!("\"{}\": {}", c.header(), v))
            .collect::<Vec<String>>()
            .join(",")
    }

    fn create_for_for_file<'a>(f: &File<'a>, columns: Vec<Column>) -> Self {
        let mut res = Self { internal: vec![] };

        columns.iter().for_each(|c| res.add_column(f, c));

        return res;
    }

    fn add_column<'a>(&mut self, f: &File<'a>, c: &Column) {
        match c {
            c => unimplemented!("{:?}", c),
        }
    }
}
