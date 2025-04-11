# v 1.1.6
- Bumped crates.

# v 1.1.5
- ADDED `FileID`, which contains general device information, such as serial number.
- ADDED `VirbFile::serial()`, `VirbSession::serial()` for retrieving device serial number.
- FIXED VIRB sessions should no longer be duplicated if multiple copies of the same FIT file are found.
