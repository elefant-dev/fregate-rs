use tonic::Code::{self, *};

/// All gRPC response codes
pub const GRPC_CODES: [Code; 17] = [
    Ok,
    Cancelled,
    Unknown,
    InvalidArgument,
    DeadlineExceeded,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    ResourceExhausted,
    FailedPrecondition,
    Aborted,
    OutOfRange,
    Unimplemented,
    Internal,
    Unavailable,
    DataLoss,
    Unauthenticated,
];

/// Return name of enum Code
#[inline]
pub const fn grpc_code_to_str(code: Code) -> &'static str {
    match code {
        Ok => "Ok",
        Cancelled => "Cancelled",
        Unknown => "Unknown",
        InvalidArgument => "InvalidArgument",
        DeadlineExceeded => "DeadlineExceeded",
        NotFound => "NotFound",
        AlreadyExists => "AlreadyExists",
        PermissionDenied => "PermissionDenied",
        ResourceExhausted => "ResourceExhausted",
        FailedPrecondition => "FailedPrecondition",
        Aborted => "Aborted",
        OutOfRange => "OutOfRange",
        Unimplemented => "Unimplemented",
        Internal => "Internal",
        Unavailable => "Unavailable",
        DataLoss => "DataLoss",
        Unauthenticated => "Unauthenticated",
    }
}

/// Return gRPC response code as string number
#[inline]
pub const fn grpc_code_to_num(code: Code) -> &'static str {
    match code {
        Ok => "0",
        Cancelled => "1",
        Unknown => "2",
        InvalidArgument => "3",
        DeadlineExceeded => "4",
        NotFound => "5",
        AlreadyExists => "6",
        PermissionDenied => "7",
        ResourceExhausted => "8",
        FailedPrecondition => "9",
        Aborted => "10",
        OutOfRange => "11",
        Unimplemented => "12",
        Internal => "13",
        Unavailable => "14",
        DataLoss => "15",
        Unauthenticated => "16",
    }
}
