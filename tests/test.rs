use {
    tokio::test,
    wthor::{
        Downloader,
        Jou,
        Trn,
        Wtb,
    },
};

#[test]
async fn test() {
    let downloader = Downloader::new();
    Jou::download(&downloader).await.unwrap();
    Trn::download(&downloader).await.unwrap();

    for year in 1977..=2023 {
        assert_eq!(Wtb::download(&downloader, year).await.unwrap().year, year);
    }
}
