use reqwest::{header::HeaderMap};
use anyhow::{Result};
use nom::{self, bytes::complete::{tag, take_while, is_not}, IResult};
use chrono::{DateTime};

use crate::realtime_data::RtData;


struct Tencent<'a> {
    headers: reqwest::header::HeaderMap,
    stock_api: &'a str,
}

impl<'a> Tencent<'a> {
    pub fn new() -> Tencent<'a> {
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
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.88 Safari/537.36".parse().unwrap()
        );
        headers.insert("Referer", "https://gu.qq.com/".parse().unwrap());
        // headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
        // headers.insert("Accept-Language", "zh-CN,zh;q=0.9".parse().unwrap());

        Self {
            headers,
            stock_api: "https://qt.gtimg.cn/q=",
        }
    }

    pub async fn get_response(&self, url: &str) -> Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let resp = client.get(url)
            .headers(self.headers.clone())
            .send()
            .await?;

        Ok(resp)
    }

    pub fn parse(input: &str) -> IResult<&str, RtData> {
        let mut rt_data = RtData::default();
        let (input, _) = tag("v_")(input)?;
        
        let (input, resp) = take_while( |c| c != '=')(input)?;
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
        match DateTime::parse_from_str(resp[30], "%Y%m%d%H%M%S") {
            Ok(dt) => rt_data.datatime = Some(dt),
            Err(e) => {rt_data.datatime = None; println!("{}       {}", e, resp[30])}
        };
        rt_data.gain_amout = resp[31].parse::<f32>().unwrap();
        rt_data.gain_percentage = resp[32].parse::<f32>().unwrap();
        rt_data.high = resp[33].parse::<f32>().unwrap();
        rt_data.low = resp[34].parse::<f32>().unwrap();
        rt_data.total_value = resp[37].parse::<f64>().unwrap()*10000.0;
        rt_data.turnover = match resp[38].parse::<f32>() {
            Ok(v) => Some(v),
            Err(_) => None,
        };
        rt_data.pe = match resp[39].parse::<f32>() {
            Ok(v) => Some(v),
            Err(_) => None,
        };
        rt_data.pb = match resp[46].parse::<f32>() {
            Ok(v) => Some(v),
            Err(_) => None,
        };
        rt_data.amplitude = resp[43].parse::<f32>().unwrap();
        rt_data.traded_market_value = match resp[44].parse::<f32>() {
            Ok(v) => Some(v),
            Err(_) => None,
        };
        rt_data.market_value = match resp[45].parse::<f32>() {
            Ok(v) => Some(v),
            Err(_) => None,
        };
        rt_data.high_limit = resp[47].parse::<f32>().unwrap();
        rt_data.low_limit = resp[48].parse::<f32>().unwrap();
        rt_data.quantity_relative_ratio = match resp[49].parse::<f32>() {
            Ok(v) => Some(v),
            Err(_) => None,
        };
        rt_data.entrust_different = match resp[50].parse::<f32>() {
            Ok(v) => Some(v),
            Err(_) => None,
        };
        rt_data.average_price = match resp[51].parse::<f32>() {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        Ok(("", rt_data))
    }

    pub async fn format_response_data(&self, response: reqwest::Response) -> Result<()>{
        let mut data = response.text().await?;
        let data_details = data.split(";");
        for i in data_details {
            println!("{}", i.trim());
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_tencent() -> Result<()> {
        let url = "http://qt.gtimg.cn/q=sz000001,sz000002";
        let tencent = Tencent::new();
        let response = tencent.get_response(url).await?;
        tencent.format_response_data(response).await?;
        // println!("{:?}", resp.text().await?);
        Ok(())
    }

    #[actix_web::test]
    async fn test_parse_data() -> Result<()> {
        
        let parse1 = "v_sz000001=\"51~平安银行~000001~15.81~15.90~15.90~821772~381881~439892~15.81~506~15.80~1439~15.79~2145~15.78~3932~15.77~687~15.82~343~15.83~2665~15.84~1449~15.85~2681~15.86~1157~~20220419161403~-0.09~-0.57~15.97~15.62~15.81/821772/1294226951~821772~129423~0.42~8.44~~15.97~15.62~2.20~3068.01~3068.08~0.94~17.49~14.31~0.79~414~15.75~8.44~8.44~~~1.33~129422.6951~0.0000~0~ ~GP-A~-4.07~-0.69~1.14~9.19~0.74~25.16~13.22~0.38~7.55~-3.18~19405522500~19405918750~2.43~-23.25~19405522500~\"";
        let parse2 = "v_sz000002=\"51~万  科Ａ~000002~20.66~20.53~20.54~1199798~620368~579430~20.66~2554~20.65~474~20.64~194~20.63~476~20.62~75~20.67~333~20.68~1136~20.69~668~20.70~6720~20.71~504~~20220419161403~0.13~0.63~20.70~19.67~20.66/1199798/2423548170~1199798~242355~1.23~10.66~~20.70~19.67~5.02~2007.65~2401.80~1.02~22.58~18.48~0.78~-5588~20.20~10.66~10.66~~~1.28~242354.8170~0.0000~0~ ~GP-A~4.55~2.43~6.05~9.55~1.96~27.83~14.43~0.00~18.40~-0.19~9717553125~11625383750~-42.55~-5.40~9717553125~\"";

        println!("{:?}", Tencent::parse(parse1));
        Ok(())
    }
}

