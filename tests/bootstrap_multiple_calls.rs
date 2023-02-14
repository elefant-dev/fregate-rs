mod multiple_bootstrap_calls {
    use fregate::{bootstrap, AppConfig};

    #[tokio::test]
    #[should_panic]
    async fn multiple_bootstrap_calls() {
        let _config: AppConfig = bootstrap([]).unwrap();
        let _config: AppConfig = bootstrap([]).unwrap();
    }
}
