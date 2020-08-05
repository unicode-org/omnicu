use clap::{App, Arg, ArgGroup};
use icu_cldr_json_data_provider::CldrDataProvider;
use icu_cldr_json_data_provider::CldrPaths;
use icu_data_provider::icu_data_key;
use icu_data_provider::iter::IterableDataProvider;
use icu_fs_data_provider::export::fs_exporter;
use icu_fs_data_provider::export::serializers;
use icu_fs_data_provider::export::FilesystemExporter;
use icu_fs_data_provider::manifest;
use std::ffi::OsStr;
use std::fmt;
use std::path::PathBuf;

// #[derive(Debug)]
enum Error {
    Unsupported(&'static str),
    ExportError(icu_fs_data_provider::error::Error),
    DataProviderError(icu_data_provider::error::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Unsupported(message) => write!(f, "Unsupported: {}", message),
            Error::ExportError(error) => write!(f, "{}", error),
            Error::DataProviderError(error) => write!(f, "{}", error),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self as &dyn fmt::Display).fmt(f)
    }
}

impl From<icu_fs_data_provider::error::Error> for Error {
    fn from(err: icu_fs_data_provider::error::Error) -> Error {
        Error::ExportError(err)
    }
}

impl From<icu_data_provider::error::Error> for Error {
    fn from(err: icu_data_provider::error::Error) -> Error {
        Error::DataProviderError(err)
    }
}

fn main() -> Result<(), Error> {
    let matches = App::new("ICU4X Data Exporter")
        .version("0.0.1")
        .author("The ICU4X Project Developers")
        .about("Export CLDR JSON into the ICU4X data schema")
        .arg(
            Arg::with_name("VERBOSE")
                .short("v")
                .long("verbose")
                .help("Enable verbose logging."),
        )
        .arg(
            Arg::with_name("DRY_RUN")
                .short("n")
                .long("dry-run")
                .help("Do not touch the filesystem (consider using with -v)."),
        )
        .arg(
            Arg::with_name("ALIASING")
                .long("aliasing")
                .takes_value(true)
                .possible_value("none")
                .possible_value("symlink")
                .help("Sets the aliasing mode of the output on the filesystem."),
        )
        .arg(
            Arg::with_name("OVERWRITE")
                .short("W")
                .long("overwrite")
                .help("Delete the output directory before writing data."),
        )
        .arg(
            Arg::with_name("CLDR_CORE")
                .long("cldr-core")
                .value_name("PATH")
                .help(
                    "Path to cldr-core JSON: \
                    https://github.com/unicode-cldr/cldr-core",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("KEY")
                .short("k")
                .long("keys")
                .multiple(true)
                .takes_value(true)
                .help("Include this data key in the output. Also see --key-file."),
        )
        .arg(
            Arg::with_name("KEY_FILE")
                .short("i")
                .long("key-file")
                .takes_value(true)
                .help(
                    "Path to text file with data keys to include, one per line. Empty lines and \
                    lines starting with '#' are ignored. Also see --key.",
                ),
        )
        .arg(
            Arg::with_name("ALL_KEYS")
                .short("A")
                .long("all-keys")
                .help("Include all keys known to ICU4X."),
        )
        .group(
            ArgGroup::with_name("KEYS")
                .arg("KEY")
                .arg("KEY_FILE")
                .arg("ALL_KEYS")
                .required(true),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .short("o")
                .long("out")
                .help(
                    "Path to output data directory. Must be empty or non-existent, unless \
                    --overwrite is present, in which case the directory is deleted first.",
                )
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    if !matches.is_present("ALL_KEYS") {
        return Err(Error::Unsupported(
            "Lists of keys are not yet supported (see #192)",
        ));
    }

    if matches.is_present("DRY_RUN") {
        return Err(Error::Unsupported("Dry-run is not yet supported"));
    }

    // TODO: Build up this list from --keys and --key-file
    let keys = [
        icu_data_key!(plurals: cardinal@1),
        icu_data_key!(plurals: ordinal@1),
    ];

    let output_path = PathBuf::from(
        matches
            .value_of_os("OUTPUT")
            .unwrap_or(OsStr::new("/tmp/icu4x_json")),
    );

    let mut cldr_paths = CldrPaths::default();

    if let Some(path) = matches.value_of("CLDR_CORE") {
        cldr_paths.cldr_core = Ok(path.into());
    }

    let provider = CldrDataProvider::new(&cldr_paths);

    let json_serializer = Box::new(serializers::JsonSerializer);

    let mut exporter_options = fs_exporter::Options::default();
    exporter_options.root = output_path;
    exporter_options.aliasing = match matches.value_of("ALIASING") {
        Some(value) => match value {
            "none" => manifest::AliasOption::NoAliases,
            "symlink" => manifest::AliasOption::Symlink,
            _ => unreachable!(),
        },
        None => manifest::AliasOption::NoAliases,
    };
    exporter_options.overwrite = if matches.is_present("OVERWRITE") {
        fs_exporter::OverwriteOption::RemoveAndReplace
    } else {
        fs_exporter::OverwriteOption::CheckEmpty
    };
    exporter_options.verbose = matches.is_present("VERBOSE");
    let mut json_file_writer = FilesystemExporter::try_new(json_serializer, &exporter_options)?;

    for key in keys.iter() {
        let result = provider.export_key(key, &mut json_file_writer);
        // Ensure flush() is called, even when the result is an error
        json_file_writer.flush()?;
        result?;
    }

    Ok(())
}
