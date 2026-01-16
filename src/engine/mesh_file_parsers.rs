use std::io::ErrorKind;
use std::path::PathBuf;
use crate::core::error::EngineError;

use super::blas::{RawTriangle, RawTriangleParse};

pub fn read_file_to_string_except_engine_err(path: PathBuf) -> Result<String, EngineError> {
    // We use .map_err to run only if there is an error
    // It matches the kind of the io error to the engine error
    // We do this as thiserror cant let us convert an enum to an error
    // This code was a copy and paste snippet, be careful with unintended behavior

    let file_contents = std::fs::read_to_string(&path)
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => EngineError::ModelConfigNotFound(path),
            ErrorKind::InvalidData => EngineError::ModelConfigInvalidData(path),
            _ => EngineError::Io(e),
        })?;

    Ok(file_contents)
}

pub fn parse_tri_file(file_path: PathBuf) -> Result<Vec<RawTriangle>, EngineError> {
    let file_contents = read_file_to_string_except_engine_err(file_path)?;

    let floats: Vec<f32> = file_contents
        .split_whitespace()
        .map(str::parse::<f32>)
        .collect::<Result<Vec<f32>, std::num::ParseFloatError>>()?;

    let mut processed_floats: Vec<RawTriangle> = Vec::new();

    if floats.len() % 9 != 0 {
        return Err(EngineError::TriFileFloatNum);
    }

    processed_floats.reserve(floats.len() / 9);

    for i in 0..(floats.len() / 9) {
        let tri = RawTriangle::from_9_floats([
            floats[i * 9], floats[i * 9 + 1], floats[i * 9 + 2],
            floats[i * 9 + 3], floats[i * 9 + 4], floats[i * 9 + 5],
            floats[i * 9 + 6], floats[i * 9 + 7], floats[i * 9 + 8],
        ]);
        
        processed_floats.push(tri);
    }
    
    Ok(processed_floats)
}