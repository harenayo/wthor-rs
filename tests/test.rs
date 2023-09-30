use {
    tokio::test,
    wthor::{
        Downloader,
        Jou,
        Trn,
        Wtb
    },
};

#[test]
async fn test() {
    let downloader = Downloader::new();
    Jou::read(&downloader.jou().await.unwrap()).unwrap();
    Trn::read(&downloader.trn().await.unwrap()).unwrap();

    for year in 1977..=2023 {
        assert_eq!(Wtb::read(&downloader.wtb(year).await.unwrap()).unwrap().year, year);
    }
}
