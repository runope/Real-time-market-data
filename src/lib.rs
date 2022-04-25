pub mod realtime_data;
pub mod sina;
pub mod tencent;

use async_trait::async_trait;
use realtime_data::ItemData;

#[async_trait]
pub trait GainRTData {
    async fn stocks(&self, stocks_list: Vec<&str>, prefix: bool) -> Vec<ItemData>;
}

// pub struct RtData {}
#[derive(Clone, Copy, Debug)]
pub enum RtData {
    Tencent,
}

impl RtData {
    pub fn init(&self) -> Box<dyn GainRTData> {
        match self {
            RtData::Tencent => Box::new(tencent::Tencent::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn it_works() {
        println!("start");
        let source = RtData::Tencent.init();

        println!(
            "{:#?}",
            source
                .stocks(vec!["sz002271", "sh600036", "sz002791", "sz159928"], true)
                .await
        );
    }
}
