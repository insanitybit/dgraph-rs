#[derive(Debug)]
pub enum DgraphError {
    Finished,
    EmptyTransaction,
    ReadOnly,
    StartTsMismatch,
    GrpcError(grpc::Error),
    Unknown,
}

impl std::fmt::Display for DgraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DgraphError::Finished => write!(f, "Dgraph client already finished"),
            DgraphError::EmptyTransaction => write!(f, "EmptyTransaction"),
            DgraphError::ReadOnly => write!(f, "Can not mutate, set to read only"),
            DgraphError::StartTsMismatch => write!(f, "StartTsMismatch"),
            DgraphError::GrpcError(_) => write!(f, "GrpcError"),
            DgraphError::Unknown => write!(f, "UnknownError"),
        }
    }
}

impl std::error::Error for DgraphError {

}

impl From<grpc::Error> for DgraphError {
    fn from(e: grpc::Error) -> DgraphError {
        DgraphError::GrpcError(e)
    }
}