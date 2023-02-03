use crate::Opts;
use anyhow::{Context, Error, Result};
use chrono::SecondsFormat;
use clap::{Parser, ValueHint};
use indicatif::{ProgressBar, ProgressStyle};
use mongodb::bson::{doc, from_document, oid::ObjectId, DateTime};
use mongodb_gridfs::{
  options::{GridFSBucketOptions, GridFSFindOptions, GridFSUploadOptions, ProgressUpdate},
  GridFSBucket,
};
use node_rs::{db::get_database, settings::Settings};
use serde::Deserialize;
use std::{
  fmt,
  fs::File,
  io::{stdout, Write},
  sync::Arc,
};
use tokio_stream::StreamExt;

#[derive(Parser, Debug)]
pub(crate) struct FsOptions {
  #[clap(subcommand)]
  command: FsSubCommand,
}

#[derive(Parser, Debug)]
#[clap()]
enum FsSubCommand {
  /// List files
  #[clap(version)]
  Ls(LsOptions),
  /// Copy files and return their ids.
  #[clap(version)]
  Put(PutOptions),
  /// Copy files on disk.
  #[clap(version)]
  Get(GetOptions),
}

#[derive(Parser, Debug)]
pub(crate) struct LsOptions {
  /// sort by upload time, newest first
  #[clap(short, long)]
  time: bool,
  /// sort by file size, largest first
  #[clap(short, long)]
  size: bool,
  /// reverse order while sorting
  #[clap(short, long)]
  reverse: bool,
  /// use a long listing format
  #[clap(short, long)]
  long: bool,
  /// Files to list
  files: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct PutOptions {
  /// Files to copy
  #[clap(value_hint(ValueHint::FilePath))]
  files: Vec<String>,
  /// Don't display progress
  #[clap(short, long)]
  quiet: bool,
}

#[derive(Parser, Debug)]
pub(crate) struct GetOptions {
  /// Files to copy
  ids: Vec<String>,
  /// Don't display progress
  #[clap(short, long)]
  quiet: bool,
  /// Stream the content on the stdout
  #[clap(short('c'), long)]
  stdout: bool,
  /// overide the filenames
  #[clap(short, long, conflicts_with("stdout"))]
  outputs: Vec<String>,
}

pub(crate) async fn fs(fs_opts: &FsOptions, _opt: &Opts, settings: &Settings) -> Result<()> {
  match &fs_opts.command {
    FsSubCommand::Ls(opts) => ls(opts, settings).await,
    FsSubCommand::Put(opts) => put(opts, settings).await,
    FsSubCommand::Get(opts) => get(opts, settings).await,
  }
}

#[derive(Deserialize, Debug)]
struct GridFile {
  // #[serde(rename(deserialize = "_id"))]
  _id: ObjectId,
  #[serde(rename(deserialize = "uploadDate"))]
  upload_date: DateTime,
  filename: String,
  length: i32,
}

impl fmt::Display for GridFile {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{:<24} {:>10} {:<8} {:<64}",
      self._id.to_hex(),
      self.length,
      self
        .upload_date
        .to_chrono()
        .to_rfc3339_opts(SecondsFormat::Millis, true),
      self.filename,
    )
  }
}

struct Progression {
  progress: ProgressBar,
}

impl ProgressUpdate for Progression {
  fn update(&self, position: usize) {
    self.progress.set_position(position as u64);
  }
}

async fn ls(opts: &LsOptions, settings: &Settings) -> Result<()> {
  let spin = ProgressBar::new_spinner();
  spin.enable_steady_tick(150);
  spin.set_message("Fetching files");
  let db = get_database(settings).await.context("Accessing mongo")?;
  let bucket = GridFSBucket::new(
    db,
    Some(
      GridFSBucketOptions::builder()
        .bucket_name(settings.database.files_bucket.clone())
        .build(),
    ),
  );

  let mut options = GridFSFindOptions::default();
  if opts.size {
    options.sort = Some(doc! {"length": ( if opts.reverse {1} else {-1}) })
  } else if opts.time {
    options.sort = Some(doc! {"uploadDate": ( if opts.reverse {1} else {-1}) })
  } else {
    options.sort = Some(doc! {"filename": ( if opts.reverse {-1} else {1}) })
  }

  let mut cursor = bucket
    .find(doc! {}, options)
    .await
    .context("Can't fetch the file list.")?;
  spin.finish_and_clear();

  while let Some(result) = cursor.next().await {
    match result {
      Ok(document) => {
        let file: GridFile = from_document(document).context("Can't decode the files")?;
        if opts.long {
          println!("{}", file);
        } else {
          println!("{}", file._id);
        }
      }
      Err(e) => return Err(e.into()),
    }
  }
  Ok::<(), Error>(())
}

async fn put(opts: &PutOptions, settings: &Settings) -> Result<()> {
  let spin = ProgressBar::new_spinner();
  spin.enable_steady_tick(150);
  spin.set_message("Connecting");
  let db = get_database(settings).await.context("Accessing mongo")?;
  let bucket = GridFSBucket::new(
    db,
    Some(
      GridFSBucketOptions::builder()
        .bucket_name(settings.database.files_bucket.clone())
        .build(),
    ),
  );
  spin.finish_and_clear();

  for filename in opts.files.clone() {
    let metadate = std::fs::metadata(&filename)?;
    let spin = ProgressBar::new(metadate.len());
    spin.set_style(
      ProgressStyle::default_bar()
        .template("{msg} [{wide_bar}] {bytes}/{total_bytes} ({elapsed}/{eta})")
        .progress_chars("=> "),
    );
    spin.set_message(&filename);
    let file = tokio::fs::File::open(&filename)
      .await
      .context(format!("Opening file \"{}\"", &filename))?;
    let id = bucket
      .clone()
      .upload_from_stream(
        &filename,
        file,
        Some(
          GridFSUploadOptions::builder()
            .progress_tick(Some(Arc::new(Progression {
              progress: spin.clone(),
            })))
            .build(),
        ),
      )
      .await?;
    spin.finish_and_clear();
    println!("{}", id.to_hex());
  }
  Ok::<(), Error>(())
}

async fn get(opts: &GetOptions, settings: &Settings) -> Result<()> {
  let mut spin = ProgressBar::hidden();
  if !opts.quiet {
    spin = ProgressBar::new_spinner();
    spin.enable_steady_tick(150);
    spin.set_message("Connecting");
  }
  let db = get_database(settings).await.context("Accessing mongo")?;
  let bucket = GridFSBucket::new(
    db,
    Some(
      GridFSBucketOptions::builder()
        .bucket_name(settings.database.files_bucket.clone())
        .build(),
    ),
  );
  spin.finish_and_clear();
  let mut outputs_iter = opts.outputs.iter();
  for id in opts.ids.clone() {
    let file_document = bucket
      .find(
        doc! {"_id":ObjectId::parse_str(&id).context("Incorrect object id format")?},
        GridFSFindOptions::default(),
      )
      .await?
      .next()
      .await;
    if let Some(file_document) = file_document {
      let file_document = file_document?;
      let mut filename = "/dev/null"; // FIXME
      if !opts.stdout {
        if let Some(output_filename) = outputs_iter.next() {
          filename = output_filename;
        } else {
          filename = file_document
            .get_str("filename")
            .context(format!("Filename not found for document {:?}", id))?;
        }
      }
      let mut spin = ProgressBar::hidden();
      if !opts.quiet {
        spin = ProgressBar::new(
          file_document
            .get_i64("length")
            .context(format!("Size not found for document {:?}", id))? as u64,
        );
        spin.set_style(
          ProgressStyle::default_bar()
            .template("{msg} [{wide_bar}] {bytes}/{total_bytes} ({elapsed}/{eta})")
            .progress_chars("=> "),
        );
        spin.set_message(filename);
      }

      let mut file = File::create(filename).context(format!("Opening file \"{}\"", &filename))?;
      let mut cursor = bucket
        .open_download_stream(ObjectId::parse_str(&id).context("Incorrect object id format")?)
        .await?;
      let mut position: u64 = 0;
      while let Some(doc) = cursor.next().await {
        if opts.stdout {
          stdout()
            .write_all(&doc)
            .context("Can't write data to stdout")?;
        } else {
          file.write(&doc).context("Can't write data to file")?;
        }
        position += doc.len() as u64;
        spin.set_position(position);
      }
      spin.finish_and_clear();
      if !opts.stdout {
        println!("{}", id);
      }
    }
  }
  spin.finish_and_clear();
  Ok::<(), Error>(())
}
