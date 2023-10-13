use tonic::{metadata::MetadataMap, Request, Status};

use super::util::merge_metadata;

// extension key struct
#[derive(Clone)]
pub struct Context {
    metadata: MetadataMap,
}

// server interceptor
pub fn save_context(mut req: Request<()>) -> Result<Request<()>, Status> {
    // save metadata to extensions for retrieval by the client intercptor
    let metadata = req.metadata().clone();
    req.extensions_mut().insert(Context { metadata: metadata });

    Ok(req)
}

// client interceptor
pub fn apply_context(mut req: Request<()>) -> Result<Request<()>, Status> {
    // extract context from extension
    let context: Context = req.extensions().get::<Context>().unwrap().clone();

    // apply custom metadata from context
    merge_metadata(req.metadata_mut(), &context.metadata);

    Ok(req)
}
