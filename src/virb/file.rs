//! Representation of a Garmin VIRB video clip with a matched FIT file.

use std::path::{Path, PathBuf};

use mp4iter::Mp4;
use time::PrimitiveDateTime;

use crate::{files::has_extension, types::FileId, Fit, FitError};

/// Represents a VIRB video clip (high and/or low-resolution)
/// and its corresponding FIT-file.
#[derive(Debug, Clone, Default, Hash)]
pub struct VirbFile {
    /// High resolution MP4
    pub mp4: Option<PathBuf>,
    /// Low resolution MP4 (`.GLV`)
    pub glv: Option<PathBuf>,
    /// FIT path
    pub fit: Option<PathBuf>,
    /// UUID
    pub uuid: String,
}

impl PartialEq for VirbFile {
    fn eq(&self, other: &Self) -> bool {
        // self.mp4 == other.mp4 && self.glv == other.glv && self.fit == other.fit && self.uuid == other.uuid
        self.uuid == other.uuid
    }
}

impl VirbFile {
    /// New `VirbFile` from path to MP4-clip.
    pub fn new(path: &Path, uuid: Option<&str>) -> Result<Self, FitError> {
        let mut virbfile = VirbFile::default();
        virbfile.set_path(path);
        virbfile.set_uuid(uuid)?;

        Ok(virbfile)
    }

    /// Get video path.
    /// Prioritizes high-resolution video.
    pub fn path(&self) -> Option<PathBuf> {
        if self.mp4.is_some() {
            self.mp4.to_owned()
        } else {
            self.glv.to_owned()
        }
    }

    /// Returns data for the MP4 user data atom `udta`
    /// from either the low or high resolution video files.
    // pub fn meta(&self) -> Result<Udta, FitError> {
    pub fn meta(&self) -> Result<Vec<(String, Vec<u8>)>, FitError> {
        if let Some(path) = &self.path() {
            let mut mp4 = Mp4::new(&path)?;
            let cursors = mp4.user_data_cursors()?;
            cursors
                .into_iter()
                .map(|(name, cursor)| Ok((name, cursor.into_inner())))
                .collect::<Result<Vec<(String, Vec<u8>)>, FitError>>()
        } else {
            Err(FitError::PathNotSet)
        }
    }

    /// Sets path by checking extention.
    /// Does nothing if not a `.mp4`, `.glv`, or `.fit`.
    /// Case agnostic.
    pub fn set_path(&mut self, path: &Path) {
        if has_extension(path, "mp4") {
            self.mp4 = Some(path.to_owned());
        }
        if has_extension(path, "glv") {
            self.glv = Some(path.to_owned());
        }
        if has_extension(path, "fit") {
            self.fit = Some(path.to_owned());
        }
    }

    /// Sets UUID. If `uuid` is `None`, UUID is extracted from GLV or MP4-file if set.
    fn set_uuid(&mut self, uuid: Option<&str>) -> Result<(), FitError> {
        match uuid {
            Some(u) => self.uuid = u.to_owned(),
            None => {
                if let Some(path) = self.mp4() {
                    self.uuid = Self::uuid_mp4(path).map_err(|_| FitError::InvalidVirbMp4)?
                } else if let Some(path) = self.glv() {
                    self.uuid = Self::uuid_mp4(path).map_err(|_| FitError::InvalidVirbMp4)?
                }
            }
        }
        Ok(())
    }

    /// Get MP4 path if set.
    pub fn mp4(&self) -> Option<&Path> {
        self.mp4.as_deref()
    }

    /// Get GLV path if set.
    pub fn glv(&self) -> Option<&Path> {
        self.glv.as_deref()
    }

    /// Get FIT path if set.
    pub fn fit(&self) -> Option<&Path> {
        self.fit.as_deref()
    }

    /// Get device serial number,
    /// extracted from corresponding FIT-file.
    pub fn serial(&self) -> Result<u32, FitError> {
        let fit = Fit::parse(
            self.fit().ok_or_else(|| FitError::PathNotSet)?,
            Some(0),
            false,
        )?;
        let file_id = FileId::from_fit(&fit, None)?;

        Ok(file_id.serial_number)
    }

    /// Extract Garmin VIRB UUID
    /// as a string from an MP4-file.
    pub fn uuid_mp4(mp4_path: &Path) -> Result<String, FitError> {
        let mut mp4 = Mp4::new(&mp4_path)?;
        let uuid = mp4.find_user_data("uuid")?.read_to_string()?;
        Ok(uuid)
    }

    /// Returns duration of linked clip/s, either MP4 or low-res GLV,
    /// depending on which is set, prioritising high-res MP4-file.
    pub fn duration(&self) -> Option<time::Duration> {
        // perhaps use proper error instead
        let video = self.mp4().or_else(|| self.glv());

        if let Some(vid) = video {
            return Mp4::new(vid).ok()?.duration(false).ok();
        }

        None
    }

    /// Returns duration of linked clip/s, either MP4 or low-res GLV,
    /// depending on which is set, prioritising high-res MP4-file.
    pub fn created(&self) -> Option<PrimitiveDateTime> {
        // perhaps use proper error instead
        let video = self.mp4().or_else(|| self.glv());

        if let Some(vid) = video {
            return Mp4::new(vid).ok()?.creation_time(false).ok();
        }

        None
    }

    pub fn timespan(&self) -> Option<(PrimitiveDateTime, PrimitiveDateTime)> {
        let video = self.mp4().or_else(|| self.glv());

        if let Some(vid) = video {
            let mut mp4 = Mp4::new(vid).ok()?;
            let start = mp4.creation_time(false).ok()?;
            let duration = mp4.duration(true).ok()?; // may not be necessary to seek to start first depending on location

            return Some((start, start + duration));
        }

        None
    }

    // pub fn first_frame(&self) -> Result<Duration, FitError> {
    //     let path = self.path().ok_or(FitError::PathNotSet)?;
    //     let mut mp4 = Mp4::new(&path)?;
    //     // Garmin AVC
    //     mp4.reset()?;

    //     // let tmcd = mp4.tmcd("GoPro TCD")?;
    //     let tmcd = mp4.tmcd("Garmin AVC")?; // no tmcd sub-atom in Virb files

    //     let offset = tmcd.offsets.first()
    //         .ok_or_else(|| FitError::InvalidVirbMp4)?;

    //     let unscaled_time = mp4.read_type_at::<u32>(offset.position, Endian::Big)?;

    //     let duration = (unscaled_time as f64 / tmcd.number_of_frames as f64).seconds();

    //     Ok(duration)
    // }

    /// Returns `true` if the specified MP4-file
    /// matches `VirbFile.uuid`.
    pub fn is_match(&self, mp4_path: &Path) -> Result<bool, FitError> {
        let uuid = Self::uuid_mp4(mp4_path)?;
        Ok(self.uuid == uuid)
    }

    /// Returns `true` if a FIT-file and
    /// at least one corresponding video file
    /// is set (`.GLV` or `.MP4`).
    pub(crate) fn matched(&self) -> bool {
        (self.mp4().is_some() || self.glv().is_some()) && self.fit().is_some()
    }
}
