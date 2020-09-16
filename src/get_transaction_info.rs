use crate::response::ResponseBody;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use asymmetric_crypto::prelude::Keypair;
use common_structure::digital_currency::{DigitalCurrency, DigitalCurrencyWrapper};
use log::{info, warn};
//数据库相关
use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

//获取金额
#[derive(Serialize, Debug)]
struct GetCurrencyRequest {
    currency_id: String,
}
//响应体
#[derive(Serialize, Debug)]
struct GetCurrencyResponse {
    currency_id: String, //转移之后的数字货币
    state: String,
    owner: String,
    create_time: i64,
    late_time: Option<i64>  //state为transfer则存在
}

#[get("/api/public/currency/info")]
pub async fn get_currency_info(
    data: web::Data<Pool>,
    req: Query<GetCurrencyResponse>,
    req_head: HttpRequest
) -> impl Responder {
    //获取数据库句柄
    let conn = data.get().await.unwrap();
    //获取用户
    let http_head = req_head.headers();
    let head_value = http_head.get("X-USERID").unwrap();
    let head_str = head_value.to_str().unwrap();

    return HttpResponse::Ok().json(ResponseBody::<()>::new_success(None));
}