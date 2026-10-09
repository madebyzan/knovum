pub(crate) mod commands;
pub(crate) mod commit;
pub(super) mod copy;
pub(super) mod geometry;
mod markdown;
pub(crate) mod mcp_resource;
pub(crate) mod media;
pub(crate) mod prepare;
pub(crate) mod registry;
pub(crate) mod web;

pub(crate) fn reserve_maintenance(
) -> Result<[crate::job_registry::JobAdmissionReservation; 2], String> {
    let imports = registry::reserve_maintenance()?;
    let media = media::reserve_maintenance()?;
    Ok([imports, media])
}
