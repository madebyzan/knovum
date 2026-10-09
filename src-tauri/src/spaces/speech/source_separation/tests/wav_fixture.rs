use std::{io::Write, path::Path};

use super::SAMPLE_RATE;

pub(super) fn write_sparse_wav(path: &Path, sample_count: u64) {
    let data_bytes = sample_count * 4;
    let mut header = [0_u8; 44];
    header[0..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&u32::try_from(data_bytes + 36).unwrap().to_le_bytes());
    header[8..12].copy_from_slice(b"WAVE");
    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&16_u32.to_le_bytes());
    header[20..22].copy_from_slice(&1_u16.to_le_bytes());
    header[22..24].copy_from_slice(&2_u16.to_le_bytes());
    header[24..28].copy_from_slice(&SAMPLE_RATE.to_le_bytes());
    header[28..32].copy_from_slice(&(SAMPLE_RATE * 4).to_le_bytes());
    header[32..34].copy_from_slice(&4_u16.to_le_bytes());
    header[34..36].copy_from_slice(&16_u16.to_le_bytes());
    header[36..40].copy_from_slice(b"data");
    header[40..44].copy_from_slice(&u32::try_from(data_bytes).unwrap().to_le_bytes());
    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(&header).unwrap();
    file.set_len(data_bytes + 44).unwrap();
}
