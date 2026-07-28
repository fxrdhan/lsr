// SPDX-FileCopyrightText: 2026 Christina Sørensen
// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: 2023-2026 Christina Sørensen, eza contributors
// SPDX-FileCopyrightText: 2014 Benjamin Sago
// SPDX-License-Identifier: MIT
use std::io::{self, Write};

use crate::{
    fs::{Dir, DotFilter, File, feature::git::GitCache, fields as f},
    output::{
        details,
        render::{PermissionsPlusRender, TimeRender},
        table::{Column, ENVIRONMENT, Environment, Options as TableOptions},
    },
};

#[derive(PartialEq, Eq, Debug)]
pub struct Options {
    /// Options for the --long option itself
    pub details: Option<details::Options>,
}

pub struct Render<'a> {
    pub git: Option<&'a GitCache>,

    pub deref_links: bool,
    pub total_size: bool,

    pub dots: DotFilter,
    pub opts: &'a Options,

    pub git_ignoring: bool,
    pub git_repos: bool,

    environment: &'a Environment,
}

impl<'a> Render<'a> {
    pub fn new(
        git: Option<&'a GitCache>,

        deref_links: bool,
        total_size: bool,

        dots: DotFilter,
        opts: &'a Options,

        git_ignoring: bool,
        git_repos: bool,
    ) -> Self {
        /// Should not cause problem as usage of the global at both places should not happen, but maybe need advice on how to better handle that ?
        let environment = &*ENVIRONMENT;

        Self {
            git,
            deref_links,
            total_size,
            dots,
            opts,
            git_ignoring,
            git_repos,
            environment,
        }
    }

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
                    .map(|f| format!("\"{}\":{{{}}}", f.name, self.render_file(f)))
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
        if let Some(table_opts) = &o.table {
            let fobj = JsonFileObject::create_for_for_file(
                f,
                table_opts,
                self.git.is_some(),
                self.git_repos,
                self.environment,
            );
            fobj.render()
        } else {
            String::new()
        }
    }
}

struct JsonFileObject<'a> {
    /// Reusing the table column to map everything we want to be displayed
    internal: Vec<(Column, String)>,

    options: &'a TableOptions,
}

impl<'a> JsonFileObject<'a> {
    /// Render a json object with the columns in the map
    fn render(self) -> String {
        self.internal
            .iter()
            .map(|(c, v)| format!("\"{}\": {}", c.header(), v))
            .collect::<Vec<String>>()
            .join(",")
    }

    fn create_for_for_file(
        f: &File<'a>,
        options: &'a TableOptions,
        actually_enable_git: bool,
        git_repos: bool,
        env: &Environment,
    ) -> Self {
        let mut res = Self {
            internal: vec![],
            options,
        };

        let columns = options.columns.collect(actually_enable_git, git_repos);

        columns.iter().for_each(|c| res.add_column(f, c, env));

        return res;
    }

    fn add_column(&mut self, f: &File, c: &Column, env: &Environment) {
        let column_opt = self.get_column(f, c, env);

        if let Some(column) = column_opt {
            self.internal.push((*c, format!("\"{column}\"")))
        }
    }

    fn get_column(&self, f: &File, c: &Column, env: &Environment) -> Option<String> {
        match c {
            /// TODO handle xattrs
            Column::Permissions => Some(f.permissions_plus(false).render_json()),
            Column::Timestamp(time_type) => Some(
                time_type
                    .get_corresponding_time(f)
                    .render_json(env.time_offset, self.options.time_format.clone()),
            ),
            c => None,
        }
    }
}
