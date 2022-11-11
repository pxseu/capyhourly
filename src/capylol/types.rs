use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BaseResponse<T> {
    pub success: bool,
    pub data: T,
}
