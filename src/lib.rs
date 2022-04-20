pub mod sina;
pub mod tencent;
pub mod realtime_data;
use async_trait::async_trait;
use realtime_data::ItemData;


#[async_trait]
pub trait GainRTData {
    async fn stocks(&self, stocks_list: Vec<&str>, prefix: bool) -> Vec<ItemData>;
}

// pub struct RtData {}
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

        let source = RtData::Tencent.init();
        println!("{:#?}", source.stocks(vec!["sz000001", "sz000002"], true).await);

        
    }
}