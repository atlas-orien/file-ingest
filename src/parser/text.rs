use crate::error::Result;
use crate::model::{FileData, IngestOptions};
use crate::utils;

pub fn parse(result: &mut FileData, bytes: &[u8], options: &IngestOptions) -> Result<()> {
    if options.extract_text {
        let text = String::from_utf8_lossy(bytes).to_string();
        utils::attach_text(result, text, options);
    }
    Ok(())
}
