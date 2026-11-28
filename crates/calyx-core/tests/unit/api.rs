use super::*;

#[test]
fn test_storefront_validation() {
    assert!(AppleMusicClient::new("cl").is_ok() && AppleMusicClient::new("US").is_ok());
    assert!(AppleMusicClient::new("12").is_err() && AppleMusicClient::new("chile").is_err());
}

#[tokio::test]
async fn test_rejects_empty_search_term() {
    assert!(
        AppleMusicClient::new("cl")
            .expect("client")
            .search_catalog("   ", 10)
            .await
            .is_err()
    );
}
