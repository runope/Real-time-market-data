use anyhow::Result;
use async_trait::async_trait;
use chrono::naive::NaiveDateTime;
use log::{info, warn};
use nom::{
    self,
    bytes::complete::{is_not, tag, take_while},
    IResult,
};
use reqwest::{header::HeaderMap, Response};

use crate::{realtime_data::ItemData, GainRTData};

pub struct Tencent {
    headers:   reqwest::header::HeaderMap,
    stock_api: &'static str,
}

#[async_trait]
impl GainRTData for Tencent {
    async fn stocks(&self, stocks_list: Vec<&str>, prefix: bool) -> Vec<ItemData> {
        let resp = self.get_stocks(stocks_list).await.unwrap();

        self.format_response_data(resp).await.unwrap()
    }
}

impl Tencent {
    pub fn new() -> Tencent {
        // GET /?q=marketStat,sh000001,usDJI,r_hkHSI HTTP/1.1
        // Accept-Encoding: gzip, deflate, br
        // Accept-Language: zh-CN,zh;q=0.9
        // Connection: keep-alive
        // Host: qt.gtimg.cn
        // Referer: https://gu.qq.com/
        // Sec-Fetch-Dest: script
        // Sec-Fetch-Mode: no-cors
        // Sec-Fetch-Site: cross-site
        // User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.88 Safari/537.36
        // sec-ch-ua: " Not A;Brand";v="99", "Chromium";v="100", "Google Chrome";v="100"
        // sec-ch-ua-mobile: ?0
        // sec-ch-ua-platform: "Windows"

        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/100.0.4896.88 Safari/537.36"
                .parse()
                .unwrap(),
        );
        headers.insert("Referer", "https://gu.qq.com/".parse().unwrap());
        // headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
        // headers.insert("Accept-Language", "zh-CN,zh;q=0.9".parse().unwrap());

        Self {
            headers,
            stock_api: "https://qt.gtimg.cn/",
        }
    }

    pub async fn get_stocks(&self, stocks_list: Vec<&str>) -> Result<Response> {
        let mut params = String::from("q=");
        for i in stocks_list {
            params = params + i + ",";
        }

        let mut url = format!("{}?{}", self.stock_api, params);
        url.pop();

        let client = reqwest::Client::new();
        let resp = client.get(url).headers(self.headers.clone()).send().await?;

        Ok(resp)
    }

    pub fn parse(input: &str) -> IResult<&str, ItemData> {
        let mut rt_data = ItemData::default();
        let (input, _) = tag("v_")(input)?;

        let (input, resp) = take_while(|c| c != '=')(input)?;
        rt_data.code = resp.to_string();
        let (input, _) = is_not("~")(input)?;
        let resp = input.split('~').collect::<Vec<&str>>();
        rt_data.name = resp[1].to_string();
        rt_data.now = resp[3].parse::<f32>().unwrap();
        rt_data.close = resp[4].parse::<f32>().unwrap();
        rt_data.open = resp[5].parse::<f32>().unwrap();
        rt_data.volume = resp[6].parse::<i64>().unwrap() * 100;
        rt_data.bid_volume = resp[7].parse::<i64>().unwrap() * 100;
        rt_data.ask_volume = resp[8].parse::<i64>().unwrap() * 100;
        rt_data.bid1 = resp[9].parse::<f32>().unwrap();
        rt_data.bid1_volume = resp[10].parse::<i64>().unwrap() * 100;
        rt_data.bid2 = resp[11].parse::<f32>().unwrap();
        rt_data.bid2_volume = resp[12].parse::<i64>().unwrap() * 100;
        rt_data.bid3 = resp[13].parse::<f32>().unwrap();
        rt_data.bid3_volume = resp[14].parse::<i64>().unwrap() * 100;
        rt_data.bid4 = resp[15].parse::<f32>().unwrap();
        rt_data.bid4_volume = resp[16].parse::<i64>().unwrap() * 100;
        rt_data.bid5 = resp[17].parse::<f32>().unwrap();
        rt_data.bid5_volume = resp[18].parse::<i64>().unwrap() * 100;
        rt_data.ask1 = resp[19].parse::<f32>().unwrap();
        rt_data.ask1_volume = resp[20].parse::<i64>().unwrap() * 100;
        rt_data.ask2 = resp[21].parse::<f32>().unwrap();
        rt_data.ask2_volume = resp[22].parse::<i64>().unwrap() * 100;
        rt_data.ask3 = resp[23].parse::<f32>().unwrap();
        rt_data.ask3_volume = resp[24].parse::<i64>().unwrap() * 100;
        rt_data.ask4 = resp[25].parse::<f32>().unwrap();
        rt_data.ask4_volume = resp[26].parse::<i64>().unwrap() * 100;
        rt_data.ask5 = resp[27].parse::<f32>().unwrap();
        rt_data.ask5_volume = resp[28].parse::<i64>().unwrap() * 100;

        match NaiveDateTime::parse_from_str(resp[30], "%Y%m%d%H%M%S") {
            Ok(dt) => rt_data.datatime = Some(dt),
            Err(e) => {
                rt_data.datatime = None;
                warn!(
                    "parse datatime error: {}. code: {}, name: {}",
                    e, rt_data.code, rt_data.name
                );
            }
        };

        rt_data.gain_amout = resp[31].parse::<f32>().unwrap();
        rt_data.gain_percentage = resp[32].parse::<f32>().unwrap();
        rt_data.high = resp[33].parse::<f32>().unwrap();
        rt_data.low = resp[34].parse::<f32>().unwrap();
        rt_data.total_value = resp[37].parse::<f64>().unwrap() * 10000.0;
        rt_data.turnover = match resp[38].parse::<f32>() {
            Ok(v) => Some(v),
            Err(e) => {
                warn!(
                    "parse turnover error: {}. code: {}, name: {}",
                    e, rt_data.code, rt_data.name
                );
                None
            }
        };

        rt_data.pe = match resp[39].parse::<f32>() {
            Ok(v) => {
                if v == 0.0 {
                    None
                } else {
                    Some(v)
                }
            }
            Err(e) => {
                warn!(
                    "parse pe error: {}. code: {}, name: {}",
                    e, rt_data.code, rt_data.name
                );
                None
            }
        };

        rt_data.pb = match resp[46].parse::<f32>() {
            Ok(v) => {
                if v == 0.0 {
                    None
                } else {
                    Some(v)
                }
            }
            Err(e) => {
                warn!(
                    "parse pb error: {}. code: {}, name: {}",
                    e, rt_data.code, rt_data.name
                );
                None
            }
        };

        rt_data.amplitude = resp[43].parse::<f32>().unwrap();
        rt_data.traded_market_value = match resp[44].parse::<f32>() {
            Ok(v) => {
                if v == 0.0 {
                    None
                } else {
                    Some(v)
                }
            }
            Err(e) => {
                warn!(
                    "parse traded_market_value error: {}. code: {}, name: {}",
                    e, rt_data.code, rt_data.name
                );
                None
            }
        };

        rt_data.market_value = match resp[45].parse::<f32>() {
            Ok(v) => {
                if v == 0.0 {
                    None
                } else {
                    Some(v)
                }
            }
            Err(e) => {
                warn!(
                    "parse market_value error: {}. code: {}, name: {}",
                    e, rt_data.code, rt_data.name
                );
                None
            }
        };

        rt_data.high_limit = resp[47].parse::<f32>().unwrap();
        rt_data.low_limit = resp[48].parse::<f32>().unwrap();
        rt_data.quantity_relative_ratio = match resp[49].parse::<f32>() {
            Ok(v) => Some(v),
            Err(e) => {
                warn!(
                    "parse quantity_relative_ratio error: {}. code: {}, name: {}",
                    e, rt_data.code, rt_data.name
                );
                None
            }
        };

        rt_data.entrust_different = match resp.get(50) {
            Some(&v) => match v.parse::<f32>() {
                Ok(v) => Some(v),
                Err(e) => {
                    warn!(
                        "parse entrust_different error: {}. code: {}, name: {}",
                        e, rt_data.code, rt_data.name
                    );
                    None
                }
            },
            None => {
                info!(
                    "no entrust_different in data. code: {}, name: {}",
                    rt_data.code, rt_data.name
                );
                None
            }
        };

        rt_data.average_price = match resp.get(51) {
            Some(&v) => match v.parse::<f32>() {
                Ok(v) => Some(v),
                Err(e) => {
                    warn!(
                        "parse entrust_different error: {}. code: {}, name: {}",
                        e, rt_data.code, rt_data.name
                    );
                    None
                }
            },
            None => {
                info!(
                    "no average_price in data. code: {}, name: {}",
                    rt_data.code, rt_data.name
                );
                None
            }
        };

        Ok(("", rt_data))
    }

    pub async fn format_response_data(&self, response: reqwest::Response) -> Result<Vec<ItemData>> {
        let data = response.text().await?;
        let data_details = data.split(";");

        let mut res = vec![];

        for i in data_details {
            match Self::parse(i.trim()) {
                Ok((_, rt_data)) => {
                    res.push(rt_data);
                }
                Err(e) => {
                    warn!("parse error: {}", e);
                }
            };
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[actix_web::test]
    // async fn test_tencent() -> Result<()> {
    //     let url = "http://qt.gtimg.cn/q=sh518801";
    //     let tencent = Tencent::new();
    //     let response = tencent.get_stocks(url).await?;
    //     tencent.format_response_data(response).await?;
    //     // println!("{:?}", resp.text().await?);
    //     Ok(())
    // }

    #[test]
    fn test_parse_data() {
        let parse1 = "v_sz000001=\"51~平安银行~000001~15.81~15.90~15.90~821772~381881~439892~15.\
                      81~506~15.80~1439~15.79~2145~15.78~3932~15.77~687~15.82~343~15.83~2665~15.\
                      84~1449~15.85~2681~15.86~1157~~20220419161403~-0.09~-0.57~15.97~15.62~15.81/\
                      821772/1294226951~821772~129423~0.42~8.44~~15.97~15.62~2.20~3068.01~3068.\
                      08~0.94~17.49~14.31~0.79~414~15.75~8.44~8.44~~~1.33~129422.6951~0.0000~0~ \
                      ~GP-A~-4.07~-0.69~1.14~9.19~0.74~25.16~13.22~0.38~7.55~-3.\
                      18~19405522500~19405918750~2.43~-23.25~19405522500~\"";
        let parse2 = "v_sz000002=\"51~万  \
                      科Ａ~000002~20.66~20.53~20.54~1199798~620368~579430~20.66~2554~20.65~474~20.\
                      64~194~20.63~476~20.62~75~20.67~333~20.68~1136~20.69~668~20.70~6720~20.\
                      71~504~~20220419161403~0.13~0.63~20.70~19.67~20.66/1199798/\
                      2423548170~1199798~242355~1.23~10.66~~20.70~19.67~5.02~2007.65~2401.80~1.\
                      02~22.58~18.48~0.78~-5588~20.20~10.66~10.66~~~1.28~242354.8170~0.0000~0~ \
                      ~GP-A~4.55~2.43~6.05~9.55~1.96~27.83~14.43~0.00~18.40~-0.\
                      19~9717553125~11625383750~-42.55~-5.40~9717553125~\"";

        match Tencent::parse(parse1) {
            Ok((_, rt_data)) => {
                assert_eq!(rt_data.code, "sz000001");
                assert_eq!(rt_data.name, "平安银行");
                assert_eq!(rt_data.now, 15.81);
                assert_eq!(rt_data.close, 15.90);
                assert_eq!(rt_data.open, 15.90);
                assert_eq!(rt_data.volume, 82177200);
                assert_eq!(rt_data.bid_volume, 38188100);
                assert_eq!(rt_data.ask_volume, 43989200);
                assert_eq!(rt_data.bid1, 15.81);
                assert_eq!(rt_data.bid1_volume, 50600);
                assert_eq!(rt_data.bid2, 15.80);
                assert_eq!(rt_data.bid2_volume, 143900);
                assert_eq!(rt_data.bid3, 15.79);
                assert_eq!(rt_data.bid3_volume, 214500);
                assert_eq!(rt_data.bid4, 15.78);
                assert_eq!(rt_data.bid4_volume, 393200);
                assert_eq!(rt_data.bid5, 15.77);
                assert_eq!(rt_data.bid5_volume, 68700);
                assert_eq!(rt_data.ask1, 15.82);
                assert_eq!(rt_data.ask1_volume, 34300);
                assert_eq!(rt_data.ask2, 15.83);
                assert_eq!(rt_data.ask2_volume, 266500);
                assert_eq!(rt_data.ask3, 15.84);
                assert_eq!(rt_data.ask3_volume, 144900);
                assert_eq!(rt_data.ask4, 15.85);
                assert_eq!(rt_data.ask4_volume, 268100);
                assert_eq!(rt_data.ask5, 15.86);
                assert_eq!(rt_data.ask5_volume, 115700);
                assert_eq!(
                    rt_data.datatime,
                    Some(NaiveDateTime::parse_from_str("20220419161403", "%Y%m%d%H%M%S").unwrap())
                );
                assert_eq!(rt_data.gain_amout, -0.09);
                assert_eq!(rt_data.gain_percentage, -0.57);
                assert_eq!(rt_data.high, 15.97);
                assert_eq!(rt_data.low, 15.62);
                assert_eq!(rt_data.total_value, 1294230000.0);
                assert_eq!(rt_data.turnover, Some(0.42));
                assert_eq!(rt_data.pe, Some(8.44));
                assert_eq!(rt_data.pb, Some(0.94));
                assert_eq!(rt_data.amplitude, 2.2);
                assert_eq!(rt_data.traded_market_value, Some(3068.01));
                assert_eq!(rt_data.market_value, Some(3068.08));
                assert_eq!(rt_data.high_limit, 17.49);
                assert_eq!(rt_data.low_limit, 14.31);
                assert_eq!(rt_data.quantity_relative_ratio, Some(0.79));
                assert_eq!(rt_data.entrust_different, Some(414.0));
                assert_eq!(rt_data.average_price, Some(15.75));
            }
            Err(e) => {
                panic!("parse error: {}", e);
            }
        }

        match Tencent::parse(parse2) {
            Ok((_, rt_data)) => {
                assert_eq!(rt_data.code, "sz000002");
                assert_eq!(rt_data.name, "万  科Ａ");
                assert_eq!(rt_data.now, 20.66);
                assert_eq!(rt_data.close, 20.53);
                assert_eq!(rt_data.open, 20.54);
                assert_eq!(rt_data.volume, 119979800);
                assert_eq!(rt_data.bid_volume, 62036800);
                assert_eq!(rt_data.ask_volume, 57943000);
                assert_eq!(rt_data.bid1, 20.66);
                assert_eq!(rt_data.bid1_volume, 255400);
                assert_eq!(rt_data.bid2, 20.65);
                assert_eq!(rt_data.bid2_volume, 47400);
                assert_eq!(rt_data.bid3, 20.64);
                assert_eq!(rt_data.bid3_volume, 19400);
                assert_eq!(rt_data.bid4, 20.63);
                assert_eq!(rt_data.bid4_volume, 47600);
                assert_eq!(rt_data.bid5, 20.62);
                assert_eq!(rt_data.bid5_volume, 7500);
                assert_eq!(rt_data.ask1, 20.67);
                assert_eq!(rt_data.ask1_volume, 33300);
                assert_eq!(rt_data.ask2, 20.68);
                assert_eq!(rt_data.ask2_volume, 113600);
                assert_eq!(rt_data.ask3, 20.69);
                assert_eq!(rt_data.ask3_volume, 66800);
                assert_eq!(rt_data.ask4, 20.7);
                assert_eq!(rt_data.ask4_volume, 672000);
                assert_eq!(rt_data.ask5, 20.71);
                assert_eq!(rt_data.ask5_volume, 50400);
                assert_eq!(
                    rt_data.datatime,
                    Some(NaiveDateTime::parse_from_str("20220419161403", "%Y%m%d%H%M%S").unwrap())
                );
                assert_eq!(rt_data.gain_amout, 0.13);
                assert_eq!(rt_data.gain_percentage, 0.63);
                assert_eq!(rt_data.high, 20.7);
                assert_eq!(rt_data.low, 19.67);
                assert_eq!(rt_data.total_value, 2423550000.0);
                assert_eq!(rt_data.turnover, Some(1.23));
                assert_eq!(rt_data.pe, Some(10.66));
                assert_eq!(rt_data.pb, Some(1.02));
                assert_eq!(rt_data.amplitude, 5.02);
                assert_eq!(rt_data.traded_market_value, Some(2007.65));
                assert_eq!(rt_data.market_value, Some(2401.8));
                assert_eq!(rt_data.high_limit, 22.58);
                assert_eq!(rt_data.low_limit, 18.48);
                assert_eq!(rt_data.quantity_relative_ratio, Some(0.78));
                assert_eq!(rt_data.entrust_different, Some(-5588.0));
                assert_eq!(rt_data.average_price, Some(20.2));
            }
            Err(e) => {
                panic!("parse error: {}", e);
            }
        }
    }

    #[test]
    fn test_parse_data2() {
        let data = "v_sh518801=\"1~国泰申赎~518801~2.229~2.229~0.000~0~0~0~0.000~0~0.000~0~0.\
                    000~0~0.000~0~0.000~0~0.000~0~0.000~0~0.000~0~0.000~0~0.\
                    000~0~~20151224150221~0.000~0.00~0.000~0.000~2.230/0/0~0~0~~~~0.000~0.000~0.\
                    00~~~0.000~2.452~2.006~\"";
        match Tencent::parse(data) {
            Ok((_, rt_data)) => {
                assert_eq!(rt_data.code, "sh518801");
                assert_eq!(rt_data.name, "国泰申赎");
                assert_eq!(rt_data.now, 2.229);
                assert_eq!(rt_data.close, 2.229);
                assert_eq!(rt_data.open, 0.0);
                assert_eq!(rt_data.volume, 0);
                assert_eq!(rt_data.bid_volume, 0);
                assert_eq!(rt_data.ask_volume, 0);
                assert_eq!(rt_data.bid1, 0.0);
                assert_eq!(rt_data.bid1_volume, 0);
                assert_eq!(rt_data.bid2, 0.0);
                assert_eq!(rt_data.bid2_volume, 0);
                assert_eq!(rt_data.bid3, 0.0);
                assert_eq!(rt_data.bid3_volume, 0);
                assert_eq!(rt_data.bid4, 0.0);
                assert_eq!(rt_data.bid4_volume, 0);
                assert_eq!(rt_data.bid5, 0.0);
                assert_eq!(rt_data.bid5_volume, 0);
                assert_eq!(rt_data.ask1, 0.0);
                assert_eq!(rt_data.ask1_volume, 0);
                assert_eq!(rt_data.ask2, 0.0);
                assert_eq!(rt_data.ask2_volume, 0);
                assert_eq!(rt_data.ask3, 0.0);
                assert_eq!(rt_data.ask3_volume, 0);
                assert_eq!(rt_data.ask4, 0.0);
                assert_eq!(rt_data.ask4_volume, 0);
                assert_eq!(rt_data.ask5, 0.0);
                assert_eq!(rt_data.ask5_volume, 0);
                assert_eq!(
                    rt_data.datatime,
                    Some(NaiveDateTime::parse_from_str("20151224150221", "%Y%m%d%H%M%S").unwrap())
                );
                assert_eq!(rt_data.gain_amout, 0.000);
                assert_eq!(rt_data.gain_percentage, 0.000);
                assert_eq!(rt_data.high, 0.000);
                assert_eq!(rt_data.low, 0.000);
                assert_eq!(rt_data.total_value, 0.000);
                assert_eq!(rt_data.turnover, None);
                assert_eq!(rt_data.pe, None);
                assert_eq!(rt_data.pb, None);
                assert_eq!(rt_data.amplitude, 0.0);
                assert_eq!(rt_data.traded_market_value, None);
                assert_eq!(rt_data.market_value, None);
                assert_eq!(rt_data.high_limit, 2.452);
                assert_eq!(rt_data.low_limit, 2.006);
                assert_eq!(rt_data.quantity_relative_ratio, None);
                assert_eq!(rt_data.entrust_different, None);
                assert_eq!(rt_data.average_price, None);
            }
            Err(e) => {
                panic!("parse error: {}", e);
            }
        }
    }
}
