use crate::error::Result;
use crate::model::{FileData, IngestOptions};
use crate::utils;

pub fn parse(result: &mut FileData, _bytes: &[u8], options: &IngestOptions) -> Result<()> {
    if options.extract_text {
        let text = pdf_extract::extract_text(&result.path)?;
        let text = utils::normalize_cjk_spacing(&text);
        utils::attach_text(result, text, options);
    }
    Ok(())
}
