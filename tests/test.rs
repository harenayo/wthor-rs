use {
    tokio::test,
    wthor::Downloader,
};

#[test]
async fn test() {
    let downloader = Downloader::new();
    downloader.jou().await.unwrap();
    downloader.trn().await.unwrap();

    for year in 1977..=2023 {
        assert_eq!(downloader.wtb(year).await.unwrap().year, year);
    }
}
