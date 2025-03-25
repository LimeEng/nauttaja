use flate2::{read::GzEncoder, write::GzDecoder, Compression};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read, Write},
    path::Path,
};
use time::OffsetDateTime;

const METADATA_FILE: &str = "nauttaja_metadata.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub name: Option<String>,
    pub notes: Option<String>,
    pub timestamp: OffsetDateTime,
}

pub fn bundle<P: AsRef<Path> + Clone>(
    source: P,
    target: impl Read + Write,
    metadata: &Metadata,
) -> io::Result<()> {
    let s = source.clone();
    let directory = s
        .as_ref()
        .file_name()
        .ok_or(io::Error::from(io::ErrorKind::InvalidData))?;

    let target = GzEncoder::new(target, Compression::best());

    let json = serde_json::to_string(&metadata)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(json.len().try_into().unwrap());
    header.set_cksum();

    let mut builder = tar::Builder::new(target);

    // Put the metadata first to make metadata access faster
    builder.append_data(&mut header, METADATA_FILE, json.as_bytes())?;
    builder.append_dir_all(directory, source)?;

    builder.finish()
}

pub fn unpack<P: AsRef<Path> + Clone>(source: impl Read + Write, target: P) -> io::Result<()> {
    let source = GzDecoder::new(source);

    let mut archive = tar::Archive::new(source);
    for file in archive.entries()? {
        let mut file = file?;
        if file.path()?.to_string_lossy() != METADATA_FILE {
            file.unpack_in(target.clone())?;
        }
    }
    Ok(())
}

pub fn extract_metadata(archive: impl Read + Write) -> io::Result<Metadata> {
    let archive = GzDecoder::new(archive);
    let metadata = file_content(archive, METADATA_FILE)?;
    serde_json::from_str(&metadata)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Failed to extract metadata"))
}

fn file_content(archive: impl Read, needle: &str) -> io::Result<String> {
    for file in tar::Archive::new(archive).entries()? {
        let mut file = file?;
        if file.path()?.to_string_lossy() == needle {
            let mut target = String::new();
            file.read_to_string(&mut target)?;
            return Ok(target);
        }
    }
    Err(io::Error::from(io::ErrorKind::NotFound))
}
