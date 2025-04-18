use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}, ops::Range, path::{Path, PathBuf}};

use mp4iter::{Atom, Mp4Error};
// use mp4iter::Udta;
use time::{PrimitiveDateTime, Duration};
use walkdir::WalkDir;

use crate::{constants::FIT_DEFAULT_DATETIME, files::has_extension, types::FileId, Fit, FitError, GpsMetadata};
use super::{
    {FitSession, FitSessions},
    VirbFile,
};

#[derive(Debug, Clone, Default)]
pub struct VirbSession{
    // pub session: Vec<VirbFile>,
    // pub fit: PathBuf,
    pub virb: Vec<VirbFile>,
    pub fit: FitSession,
    /// Date time for first message in fit.
    /// Note that this does not represent
    /// start of session.
    pub t0: Option<PrimitiveDateTime>,
    /// Time for start of session, relative to `.t0`.
    pub start: Option<Duration>,
    /// Time for end of session, relative to `.t0`.
    pub end: Option<Duration>,
    processed: bool // defaults to false
}

impl Hash for VirbSession {
    /// A combined hash of the UUIDs for each, respective
    /// `VirbFile` in this session.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.virb.iter()
            .map(|f| f.uuid.to_owned())
            .collect::<Vec<_>>()
            .hash(state)
    }
}

impl PartialEq for VirbSession {
    fn eq(&self, other: &Self) -> bool {
        // self.virb == other.virb && self.fit == other.fit && self.t0 == other.t0 && self.start == other.start && self.end == other.end && self.processed == other.processed
        let mut uuid_self: Vec<&str> = self.virb.iter().map(|f| f.uuid.as_str()).collect();
        uuid_self.sort();
        let mut uuid_other: Vec<&str> = other.virb.iter().map(|f| f.uuid.as_str()).collect();
        uuid_other.sort();

        // println!("SELF:  {uuid_self:?}");
        // println!("OTHER: {uuid_other:?}");

        uuid_self == uuid_other
    }
}

impl VirbSession {
    /// Determine recording sessions in `dir` (recursive).
    /// Does not parse FIT-files if located, only paths are set.
    pub fn sessions_from_path(dir: &Path, verbose: bool) -> Vec<Self> {
        let mut uuid2virbfile: HashMap<String, VirbFile> = HashMap::new();
        let mut fit_sessions_vec: Vec<FitSessions> = Vec::new();

        let mut count = 0;

        // 1. Go through files, must complete before sessions can be derived
        for result in WalkDir::new(dir) {
            let path = match result {
                Ok(f) => f.path().to_owned(),
                Err(_) => continue
            };

            // Ignoring errors for now
            if has_extension(&path, "mp4") {
                if let Ok(vf) = VirbFile::new(&path, None) {
                    if verbose {
                        count += 1;
                        println!("{:4}. [MP4] {}", count, path.display());
                    }
                    uuid2virbfile.entry(vf.uuid.to_owned())
                        .or_insert(vf).mp4 = Some(path.to_owned()); // sets path twice...
                }
            } else if has_extension(&path, "glv") {
                if let Ok(vf) = VirbFile::new(&path, None) {
                    if verbose {
                        count += 1;
                        println!("{:4}. [GLV] {}", count, path.display());
                    }
                    uuid2virbfile.entry(vf.uuid.to_owned())
                        .or_insert(vf).glv = Some(path.to_owned()); // sets path twice...
                }
            } else if has_extension(&path, "fit") {
                if let Ok(fit_sessions) = FitSessions::new(&path) {
                    if verbose {
                        count += 1;
                        println!("{:4}. [FIT] {} ({})", count, path.display(), fit_sessions.len());
                    }
                    for fit_session in fit_sessions.iter() {
                        fit_session.iter()
                            .for_each(|u| {
                                if let Ok(vf) = VirbFile::new(&fit_session.path, Some(u)) {
                                    uuid2virbfile.entry(u.to_owned())
                                        .or_insert(vf).fit = Some(fit_session.path.to_owned())
                                }
                            });
                    }
                    fit_sessions_vec.push(fit_sessions);
                }
            }
        }

        // 2. Match and group files into VirbSessions via FitSession UUIDs.
        let mut virb_sessions: Vec<Self> = Vec::new();
        
        for fit_sessions in fit_sessions_vec.iter() {
            
            for fit_session in fit_sessions.iter() {

                let mut virb_session = Self::default();

                let virb: Vec<VirbFile> = fit_session.iter()
                    .filter_map(|uuid| uuid2virbfile.get(uuid))
                    .cloned()
                    .collect();

                virb_session.virb = virb;
                virb_session.fit = fit_session.to_owned();

                // Ensures a FIT-file with at least one corresponding
                // video file was found (.GLV or .MP4)
                if virb_session.matched() {
                    virb_sessions.push(virb_session);
                }
                
            }
        }

        // If any duplicate FIT files are found sessions will also be duplicated
        // since UUID in FIT file is used to locate corresponding video clips
        virb_sessions.sort_by_key(|s| s.virb.first().map(|f| f.uuid.to_owned()));
        virb_sessions.dedup();

        virb_sessions
    }

    /// Returns `true` if each file in session
    /// has a FIT-file and at least one corresponding video file.
    fn matched(&self) -> bool {
        if self.virb.is_empty() {
            return false
        }

        let matched = self.virb.iter()
            .map(|vf| vf.matched())
            .collect::<Vec<_>>();
        
        if matched.is_empty() {
            false
        } else if matched.iter().any(|m| m == &false) {
            false
        } else {
            true
        }
    }

    /// Search `dir` to match all files for the recording session
    /// `mp4_path` is part of.
    pub fn from_mp4(mp4_path: &Path, dir: &Path, verbose: bool) -> Option<Self> {
        let virbfile = VirbFile::new(mp4_path, None).ok()?;

        Self::from_uuid(&virbfile.uuid, dir, verbose)
    }

    /// Derives all files for the recording session that contains
    /// `uuid`. `dir` is used as starting
    /// point to search recursively for matches.
    pub fn from_uuid(uuid: &str, dir: &Path, verbose: bool) -> Option<Self> {
        let virbsessions = VirbSession::sessions_from_path(dir, verbose);
        for virbsession in virbsessions.iter() {
            if virbsession.contains(&uuid) {
                return Some(virbsession.to_owned())
            }
        }

        None
    }

    fn get_from_index(sessions: &[Self], index: usize, offset_hours: Option<i64>) -> Result<Self, FitError> {
        if let Some(session) = sessions.get(index) {
            let mut session = session.to_owned();
            if !session.processed {
                session.process(offset_hours.unwrap_or(0))?;
            }
            return Ok(session)
        }
        Err(FitError::NoSuchSession)
    }

    fn get_from_uuid(sessions: &[Self], uuid: &str, offset_hours: Option<i64>) -> Result<Self, FitError> {
        // .contains() does not accept &str, only &String, see: https://github.com/rust-lang/rust/issues/42671
        // .find(|session| session.uuid().contains(uuid))
        if let Some(session) = sessions.iter().find(|session| session.uuid().iter().any(|u| u == uuid)) {
            let mut session = session.to_owned();
            if !session.processed {
                session.process(offset_hours.unwrap_or(0))?;
            }
            return Ok(session)
        }
        Err(FitError::NoSuchSession)
    }

    // Get basename for naming e.g. output dir,
    // derived from filestem from first clip in session
    // prioritizing high-res filename.
    pub fn basename(&self) -> Option<String> {
        let path = self.virb.first()
            .and_then(|vf| vf.mp4().or_else(|| vf.glv()));

        if let Some(p) = path {
            p.file_stem()
                .map(|s| s.to_string_lossy().to_string())
        } else {
            None
        }
    }

    /// Extracts MP4 `udta` atom.
    /// The `udta` data fields are
    /// returned as raw bytes.
    // pub fn meta(&self) -> Vec<Udta> {
    // pub fn meta(&self) -> Result<Vec<(String, Vec<u8>)>, FitError> {
    pub fn meta(&self) -> Vec<(String, Vec<u8>)> {
        self.virb.iter()
            .filter_map(|v| v.meta().ok()) // errors discarded...
            .flatten()
            .collect()
    }

    /// Returns number of video clips in session.
    pub fn len(&self) -> usize {
        self.virb.len()
    }

    /// Returns `true` is session contains no clips.
    pub fn is_empty(&self) -> bool {
        self.virb.is_empty()
    }

    /// Returns `true` if the session
    /// contains a clip with specified `uuid`.
    pub fn contains(&self, uuid: &str) -> bool {
        self.virb.iter()
            .any(|virbfile| virbfile.uuid == uuid)
    }

    pub fn dedup(&mut self) {

    }

    /// Returns path to FIT-file.
    pub fn fit_path(&self) -> PathBuf {
        self.fit.path.to_owned()
    }

    /// Returns FIT-data if set.
    pub fn fit(&self) -> Option<&Fit> {
        self.fit.fit.as_ref()
    }

    /// Get device serial number,
    /// extracted from corresponding FIT-file.
    pub fn serial(&self) -> Result<u32, FitError> {
        let fit = Fit::parse(&self.fit_path(), Some(0), false)?;
        let file_id = FileId::from_fit(&fit, None)?;

        Ok(file_id.serial_number)
    }

    // fn first_vid_any(&self) -> Option<PathBuf> {
    //     self.virb.first()?.path()
    // }

    /// Returns video clip paths,
    /// prioritizing high-resolution MP4.
    pub fn paths(&self) -> Vec<PathBuf> {
        self.virb.iter()
            .filter_map(|v| v.path().map(|p| p.to_owned()))
            .collect()
    }

    /// Returns paths to high-resolution MP4-clips if set (`.MP4`).
    pub fn mp4(&self) -> Vec<PathBuf> {
        self.virb.iter()
            .filter_map(|v| v.mp4().map(|p| p.to_owned()))
            .collect()
    }

    /// Returns paths to low-resolution MP4-clips if set (`.GLV`).
    pub fn glv(&self) -> Vec<PathBuf> {
        self.virb.iter()
            .filter_map(|v| v.glv().map(|p| p.to_owned()))
            .collect()
    }

    /// Returns all GPS data filtered on session timespan.
    pub fn gps(&self) -> Result<Vec<GpsMetadata>, FitError> {
        if let Some(fit) = self.fit.fit.as_ref() {
            return fit.gps(Some(&self.range()))
        }
        Err(FitError::NoData)
    }

    /// Returns indeces corresponding to session FIT data slice.
    /// Generated from camera event start end event types, where
    /// 0 = session start, and 2 = session end.
    /// See FIT SDK Profile.xlsx.
    pub fn range(&self) -> Range<usize> {
        self.fit.range()
    }

    /// Parse FIT-file and assign to `self.data`.
    fn parse(&mut self) -> Result<(), FitError> {
        // self.data = Some(Fit::new(&self.fit.path)?);
        self.fit.parse()?;
        Ok(())
    }

    fn set_time(&mut self, offset_hours: i64) -> Result<(), FitError> {
        if let Some(fit) = &self.fit.fit {
            self.t0 = Some(fit.t0(offset_hours, false)?);

            let cam = fit.camera(Some(&self.range()))?;

            // Fit::sessions() generates start/end index range based on camera event type
            // with 0 = session start, 2 = session end.
            // VirbSession::range() reflects this, so first()/last() should be ok to use.
            self.start = cam.first()
                .map(|evt| {
                    Duration::seconds(evt.timestamp as i64)
                    + Duration::milliseconds(evt.timestamp_ms as i64)
                });
            self.end = cam.last()
                .map(|evt| {
                    Duration::seconds(evt.timestamp as i64)
                    + Duration::milliseconds(evt.timestamp_ms as i64)
                });
        }

        Ok(())
    }

    /// Parses and processes matched FIT-file, then populates data, and time fields.
    pub fn process(&mut self, offset_hours: i64) -> Result<(), FitError> {
        self.fit.derive()?;
        self.set_time(offset_hours)?;

        Ok(())
    }

    /// Returns timestamp for start of session.
    pub fn start(&self) -> Option<PrimitiveDateTime> {
        // Some(self.t0? + self.start?)
        self.virb.first()?.created()
    }

    /// Returns timestamp for end of session.
    pub fn end(&self) -> Option<PrimitiveDateTime> {
        // Some(self.t0? + self.end?)
        Some(self.start()? + self.video_duration()?)
    }

    /// Returns start and end timespan as a tuple
    /// `(PrimitiveDateTime, PrimitiveDateTime)`.
    pub fn timespan(&self) -> Option<(PrimitiveDateTime, PrimitiveDateTime)> {
        let start = self.start()?;
        Some((start, start + self.video_duration()?))
    }

    /// Returns total session duration
    /// by summing durations for all video clips in session.
    pub fn video_duration(&self) -> Option<time::Duration> {
        if self.virb.len() == 1 {
            self.virb.first().and_then(|s| s.duration())
        } else {
            Some(self.virb.iter()
                .filter_map(|s| s.duration()) // not great for error reporting...
                .fold(time::Duration::ZERO, |acc, dur| acc + dur))
            // self.virb.iter()
            //     .map(|s| s.duration()?)
            //     .sum()
        }

        // None
    }

    /// Returns total session duration
    /// derived from FIT camera events.
    /// May differ slightly from video duration.
    pub fn fit_duration(&self) -> Option<time::Duration> {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            return Some(end - start)
        }

        None
    }

    /// Returns all unique UUIDs in recording session
    /// in chronological order.
    pub fn uuid(&self) -> Vec<String> {
        self.virb.iter()
            .map(|v| v.uuid.to_owned())
            .collect()
    }

    /// Extracts UUID from linked FIT-file if set.
    pub fn uuid_from_fit(&self) -> Result<Vec<String>, FitError> {
        if let Some(data) = &self.fit.fit {
            let cam = data.camera(Some(&self.range()))?;
            let mut uuids: Vec<_> = cam.iter()
                .map(|evt| evt.camera_file_uuid.to_owned())
                .collect();
            uuids.dedup();

            Ok(uuids)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}