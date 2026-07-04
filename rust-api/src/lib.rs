pub mod app_error;
pub mod postgres;
pub mod structs_and_enums;

pub fn generate_secret() -> Result<String, getrandom::Error> {
    use base64::Engine as _;
    let mut buf = [0u8; 32];
    getrandom::fill(&mut buf)?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf))
}
