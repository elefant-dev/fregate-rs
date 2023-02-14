use fregate::sugar::grpc_codes::{grpc_code_to_num, grpc_code_to_str, GRPC_CODES};

#[test]
fn test_grpc_code_to_num() {
    for code in GRPC_CODES {
        assert_eq!(grpc_code_to_num(code), &format!("{}", code as i32));
    }
}

#[test]
fn test_grpc_code_to_str() {
    for code in GRPC_CODES {
        assert_eq!(grpc_code_to_str(code), &format!("{code:?}"));
    }
}
