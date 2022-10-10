mod multiple_bootstrap_calls {
    use fregate::{bootstrap, Empty};

    #[tokio::test]
    #[should_panic]
    async fn multiple_bootstrap_calls() {
        let _config = bootstrap::<Empty, _>([]).unwrap();
        let _config = bootstrap::<Empty, _>([]).unwrap();
    }
}
