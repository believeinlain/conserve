// Conserve backup system.
// Copyright 2015, 2016, 2017, 2018, 2019, 2020 Martin Pool.

//! Command-line entry point for Conserve backups.

use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use conserve::backup::BackupOptions;
use conserve::ReadTree;
use conserve::RestoreOptions;
use conserve::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "conserve",
    about = "A robust backup tool <https://github.com/sourcefrog/conserve/>",
    author
)]
enum Command {
    /// Copy source directory into an archive.
    Backup {
        /// Path of an existing archive.
        archive: PathBuf,
        /// Source directory to copy from.
        source: PathBuf,
        /// Print copied file names.
        #[structopt(long, short)]
        verbose: bool,
        #[structopt(long, short, number_of_values = 1)]
        exclude: Vec<String>,
    },

    Debug(Debug),

    /// Compare a stored tree to a source directory.
    Diff {
        archive: PathBuf,
        source: PathBuf,
        #[structopt(long, short)]
        backup: Option<BandId>,
        #[structopt(long, short, number_of_values = 1)]
        exclude: Vec<String>,
        /// Compare to the incomplete contents of an unfinished backup.
        #[structopt(long, requires = "backup")]
        incomplete: bool,
    },

    /// Create a new archive.
    Init {
        /// Path for new archive.
        archive: PathBuf,
    },

    /// List files in a stored tree or source directory, with exclusions.
    Ls {
        #[structopt(flatten)]
        stos: StoredTreeOrSource,
    },

    /// Copy a stored tree to a restore directory.
    Restore {
        archive: PathBuf,
        destination: PathBuf,
        #[structopt(long, short)]
        backup: Option<BandId>,
        #[structopt(long, short)]
        force_overwrite: bool,
        #[structopt(long, short)]
        verbose: bool,
        #[structopt(long, short, number_of_values = 1)]
        exclude: Vec<String>,
        #[structopt(long="only", short="i", number_of_values = 1)]
        include_only: Option<String>,
        /// Restore the incomplete contents of an unfinished backup.
        #[structopt(long, requires = "backup")]
        incomplete: bool,
    },

    /// Show the total size of files in a stored tree or source directory, with exclusions.
    Size {
        #[structopt(flatten)]
        stos: StoredTreeOrSource,
    },

    /// Check that an archive is internally consistent.
    Validate {
        /// Path of the archive to check.
        archive: PathBuf,
    },

    /// List backup versions in an archive.
    Versions {
        archive: PathBuf,
        /// Show only version names.
        #[structopt(long, short = "q")]
        short: bool,
        /// Show size of stored trees.
        #[structopt(long, short = "z", conflicts_with = "short")]
        sizes: bool,
    },
}

#[derive(Debug, StructOpt)]
struct StoredTreeOrSource {
    #[structopt(required_unless = "source")]
    archive: Option<PathBuf>,

    /// List files in a source directory rather than an archive.
    #[structopt(long, short, conflicts_with = "archive", required_unless = "archive")]
    source: Option<PathBuf>,

    #[structopt(long, short, conflicts_with = "source")]
    backup: Option<BandId>,

    #[structopt(long, short, number_of_values = 1)]
    exclude: Vec<String>,

    /// Measure the incomplete contents of an unfinished backup.
    #[structopt(long, requires = "backup")]
    incomplete: bool,
}

/// Show debugging information.
#[derive(Debug, StructOpt)]
enum Debug {
    /// Dump the index as json.
    Index {
        /// Path of the archive to read.
        archive: PathBuf,

        /// Backup version number.
        #[structopt(long, short)]
        backup: Option<BandId>,

        /// List the incomplete contents of an unfinished backup.
        #[structopt(long, requires = "backup")]
        incomplete: bool,
    },

    /// List all blocks.
    Blocks { archive: PathBuf },

    /// List all blocks referenced by any band.
    Referenced { archive: PathBuf },
}

impl Command {
    fn run(&self) -> Result<()> {
        let mut stdout = std::io::stdout();
        match self {
            Command::Backup {
                archive,
                source,
                verbose,
                exclude,
            } => {
                let options = BackupOptions {
                    print_filenames: *verbose,
                    excludes: excludes::from_strings(exclude)?,
                };
                let copy_stats = Archive::open_path(archive)?.backup(source, &options)?;
                ui::println("Backup complete.");
                copy_stats.summarize_backup(&mut stdout);
            }
            Command::Debug(Debug::Blocks { archive }) => {
                let mut bw = BufWriter::new(stdout);
                for hash in Archive::open_path(archive)?.block_dir().block_names()? {
                    writeln!(bw, "{}", hash)?;
                }
            }
            Command::Debug(Debug::Index {
                archive,
                backup,
                incomplete,
            }) => {
                let st = stored_tree_from_opt(archive, &backup, &Vec::new(), *incomplete)?;
                output::show_index_json(&st.band(), &mut stdout)?;
            }
            Command::Debug(Debug::Referenced { archive }) => {
                let mut bw = BufWriter::new(stdout);
                for hash in Archive::open_path(archive)?.referenced_blocks()? {
                    writeln!(bw, "{}", hash)?;
                }
            }
            Command::Diff {
                archive,
                source,
                backup,
                exclude,
                incomplete,
            } => {
                // TODO: Consider whether the actual files have changed.
                // TODO: Summarize diff.
                // TODO: Optionally include unchanged files.
                let excludes = excludes::from_strings(exclude)?;
                let st = stored_tree_from_opt(archive, backup, exclude, *incomplete)?;
                let lt = LiveTree::open(source)?.with_excludes(excludes);
                output::show_tree_diff(&mut conserve::iter_merged_entries(&st, &lt)?, &mut stdout)?;
            }
            Command::Init { archive } => {
                Archive::create_path(&archive)?;
                ui::println(&format!("Created new archive in {:?}", &archive));
            }
            Command::Ls { stos } => {
                if let Some(archive) = &stos.archive {
                    output::show_tree_names(
                        &stored_tree_from_opt(
                            archive,
                            &stos.backup,
                            &stos.exclude,
                            stos.incomplete,
                        )?,
                        &mut stdout,
                    )?;
                } else {
                    output::show_tree_names(
                        &live_tree_from_opt(stos.source.as_ref().unwrap(), &stos.exclude)?,
                        &mut stdout,
                    )?;
                }
            }
            Command::Restore {
                archive,
                destination,
                backup,
                verbose,
                force_overwrite,
                exclude,
                include_only,
                incomplete,
            } => {
                let band_selection = band_selection_policy_from_opt(backup, *incomplete);
                let archive = Archive::open_path(archive)?;
                
                
                let include_only = match include_only {
                    None => "/",
                    Some(path) => path
                };

                let options = RestoreOptions {
                    print_filenames: *verbose,
                    excludes: excludes::from_strings(exclude)?,
                    include_only: Apath::from(include_only),
                    band_selection,
                    overwrite: *force_overwrite,
                };


                let copy_stats = archive.restore(&destination, &options)?;
                ui::println("Restore complete.");
                copy_stats.summarize_restore(&mut stdout)?;
            
            
            }
            Command::Size { ref stos } => {
                ui::set_progress_phase(&"Measuring".to_string());
                let size = if let Some(archive) = &stos.archive {
                    stored_tree_from_opt(archive, &stos.backup, &stos.exclude, stos.incomplete)?
                        .size()?
                        .file_bytes
                } else {
                    live_tree_from_opt(stos.source.as_ref().unwrap(), &stos.exclude)?
                        .size()?
                        .file_bytes
                };
                ui::println(&conserve::bytes_to_human_mb(size));
            }
            Command::Validate { archive } => Archive::open_path(archive)?
                .validate()?
                .summarize(&mut stdout)?,
            Command::Versions {
                archive,
                short,
                sizes,
            } => {
                ui::enable_progress(false);
                let archive = Archive::open_path(archive)?;
                if *short {
                    output::show_brief_version_list(&archive, &mut stdout)?;
                } else {
                    output::show_verbose_version_list(&archive, *sizes, &mut stdout)?;
                }
            }
        }
        Ok(())
    }
}

fn stored_tree_from_opt(
    archive: &Path,
    backup: &Option<BandId>,
    exclude: &[String],
    incomplete: bool,
) -> Result<StoredTree> {
    let archive = Archive::open_path(archive)?;
    let policy = band_selection_policy_from_opt(backup, incomplete);
    Ok(archive
        .open_stored_tree(&policy)?
        .with_excludes(excludes::from_strings(exclude)?))
}

fn band_selection_policy_from_opt(
    backup: &Option<BandId>,
    incomplete: bool,
) -> BandSelectionPolicy {
    if let Some(band_id) = backup {
        BandSelectionPolicy::Specified(band_id.clone())
    } else if incomplete {
        BandSelectionPolicy::Latest
    } else {
        BandSelectionPolicy::LatestClosed
    }
}

fn live_tree_from_opt(source: &Path, exclude: &[String]) -> Result<LiveTree> {
    Ok(LiveTree::open(source)?.with_excludes(excludes::from_strings(exclude)?))
}

// fn read_tree_from_options(
//     archive: Option<&Path>,
//     backup: Option<&BandId>,
//     source: Option<&Path>,
// ) -> Result<Box<dyn ReadTree>> {
//     // TODO: Maybe move to ReadTree?
//     if let Some(archive) = archive {
//         stored_tree_from_opt(archive, &backup)
//     } else {
//         LiveTree::open(source.expect("source must be set if archive is not"))
//     }
//     // TODO: Excludes
//     // .with_excludes(excludes_from_option(subm)?))
// }

fn main() {
    ui::enable_progress(true);
    let result = Command::from_args().run();
    ui::clear_progress();
    if let Err(ref e) = result {
        ui::show_error(e);
        // // TODO: Perhaps always log the traceback to a log file.
        // if let Some(bt) = e.backtrace() {
        //     if std::env::var("RUST_BACKTRACE") == Ok("1".to_string()) {
        //         println!("{}", bt);
        //     }
        // }
        // Avoid Rust redundantly printing the error.
        std::process::exit(1);
    }
    // TODO: If the operation had >0 non-fatal errors, return a non-zero exit code.
}
