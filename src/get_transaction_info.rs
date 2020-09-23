use crate::response::ResponseBody;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use log::{info, warn};
use common_structure::digital_currency::DigitalCurrencyWrapper;
//数据库相关
use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};


/*
 *function: 获取单个数字货币详情
 * param：
 * data: 数据库连接句柄
 * req：数据请求结构
 * req_head::http请求头，包含用户认证
 *
 * return :响应数据code=0成功，其他值参考错误列表
 */
//获取金额
#[derive(Deserialize, Debug)]
pub struct GetCurrencyRequest {
    currency_id: String,
}
//响应体
#[derive(Serialize, Debug)]
pub struct GetCurrencyResponse {
    currency_id: String, //转移之后的数字货币
    transaction_id: String,
    status: String,
    owner: String,
    amount: i64,
    create_time: i64,
    late_time: i64,  
}

#[get("/api/public/currency/info")]
pub async fn get_currency_info(
    data: web::Data<Pool>,
    req: web::Query<GetCurrencyRequest>,
    req_head: HttpRequest
) -> impl Responder {
    //获取数据库句柄
    let conn = data.get().await.unwrap();
    //获取用户
    let http_head = req_head.headers();
    let head_value = http_head.get("X-USERID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //获取货币信息表数据
    let select_currency = match conn.query(
        "SELECT transaction_id, status, owner, amount, create_time, update_time from 
        transactions where currency_id = $1 and cloud_user_id = $2",&[&req.currency_id,&head_str]
    ).await {
        Ok(row) => {
            info!("electe success: {:?}", row);
            row
        }
        Err(error) => {
            warn!("1、digistal currency transactions select failed :{:?}!!", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(error.to_string()));
        }
    };
    if select_currency.is_empty(){
        warn!("this user and currency id mismatching");
        return HttpResponse::Ok().json(ResponseBody::<()>::currency_is_none());
    }
    let transaction_id: String = select_currency[0].get(0);
    let status: String = select_currency[0].get(1);
    let owner: String = select_currency[0].get(2);
    let amount: i64 = select_currency[0].get(3);
    let create: NaiveDateTime = select_currency[0].get(4);
    let create_time = create.timestamp();
    let last: NaiveDateTime = select_currency[0].get(5);
    let late_time = last.timestamp();

    return HttpResponse::Ok().json(ResponseBody::<GetCurrencyResponse>::new_success(Some(GetCurrencyResponse{
        currency_id: req.currency_id.clone(),
        transaction_id,
        status,
        owner,
        amount,
        create_time,
        late_time,
    })));
}

/*
 *function: 获取单次交易输入输出货币信息
 * param：
 * data: 数据库连接句柄
 * req：数据请求结构
 * req_head::http请求头，包含用户认证
 *
 * return :响应数据code=0成功，其他值参考错误列表
 */

 #[derive(Deserialize, Debug)]
pub struct GetExchangeRequest {
    transaction_id: String,
}
 //响应体
#[derive(Serialize, Debug)]
pub struct GetExchangeResponse {
    transaction_id: String, //本次交易编号
    trans_amount: u64,  //交易金额
    transaction_time: i64,   //交易时间
    input: Vec<(String, u64)>,
    output:Vec<(String, u64)>
}

#[get("/api/public/transaction/info")]
pub async fn get_exchange_info(
    data: web::Data<Pool>,
    req: web::Query<GetExchangeRequest>,
    req_head: HttpRequest
) -> impl Responder {
    //获取数据库句柄
    let conn = data.get().await.unwrap();
    //获取用户
    let http_head = req_head.headers();
    let head_value = http_head.get("X-USERID").unwrap();
    let head_str = head_value.to_str().unwrap();

    //获取交易id信息
    let select_exchang = match conn.query(
        "SELECT (trans_info->'inner'->'inputs'), (trans_info->'inner'->'outputs'),create_time from exchanges where transaction_id = $1 and cloud_user_id = $2",
        &[&req.transaction_id, &head_str]
    ).await{
        Ok(row) => {
            info!("electe success: {:?}", row);
            row
        }
        Err(error) => {
            warn!("1、digistal currency exchange select failed :{:?}!!", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(error.to_string()));
        }
    };
    if select_exchang.is_empty(){
        warn!("This transaction does not exist");
        return HttpResponse::Ok().json(ResponseBody::<()>::currency_is_none());
    }
    let input_info : Vec<DigitalCurrencyWrapper> = serde_json::from_value(select_exchang[0].get(0)).unwrap();
    let output: Vec<(String, u64)> = serde_json::from_value(select_exchang[0].get(1)).unwrap();
    let create: NaiveDateTime = select_exchang[0].get(2);
    let transaction_time = create.timestamp();
    let mut input: Vec<(String, u64)> = Vec::new();
    let mut trans_amount:u64 = 0;
    for currency in input_info.iter(){
        //货币id
        let currency_id = currency.get_body().get_id_str();
        //金额
        let amount:u64 = currency.get_body().get_amount() as u64;
        trans_amount += amount;
        input.push((currency_id,amount))
    }
    return HttpResponse::Ok().json(ResponseBody::<GetExchangeResponse>::new_success(Some(GetExchangeResponse{
        transaction_id: req.transaction_id.clone(), 
        trans_amount,  
        transaction_time,   
        input,
        output
    })));
}

/*
 *function: 获取数字货币列表
 * param：
 * data: 数据库连接句柄
 * req：数据请求结构
 * req_head::http请求头，包含用户认证
 *
 * return :响应数据code=0成功，其他值参考错误列表
 */
 #[derive(Deserialize, Debug)]
 pub struct CurrencyListRequest {
     page: i64,
     count: i64,
     currency_id: Option<String>,
     status: Option<String>,
     amount: Option<i64>,
     create_time: Option<NaiveDateTime>,
     create_end_time: Option<NaiveDateTime>,
     destroy_time: Option<NaiveDateTime>,
     destroy_end_time: Option<NaiveDateTime>,
 }

 #[derive(Serialize, Debug)]
struct CurrencyListResponseInner {
    currency_id: String,
    amount: i64,
    create_time: i64,
    status: String,
    late_time: i64,  
}
#[derive(Serialize, Debug)]
struct CurrencyListResponse {
    total: i64,
    inner: Vec<CurrencyListResponseInner>,
}
#[get("/api/public/currency/list")]
pub async fn get_currency_list(
    data: web::Data<Pool>,
    req: web::Query<CurrencyListRequest>,
    req_head: HttpRequest
) -> impl Responder {
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-USERID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //连接数据库
    let conn = data.get().await.unwrap();
    //计算显示页数与条数
    let offset: i64 = (req.page - 1) * req.count;
    let mut sql_sum = "SELECT COUNT(*) FROM transactions where cloud_user_id = $1".to_string();
    let mut sql = "SELECT currency_id, status, amount, create_time, update_time from transactions 
    where cloud_user_id = $1".to_string();
    let mut sql_params: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec![&head_str];
    if req.currency_id.is_some(){
        let currency_sql = format!(" and currency_id like \'{}%\'",req.currency_id.as_ref().unwrap());
        sql_sum.push_str(&currency_sql);
        sql.push_str(&currency_sql);
    }
    if req.status.is_some(){
        sql_sum.push_str(" and status = $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and status = $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.status.as_ref().unwrap());
    }
    if req.amount.is_some(){
        sql_sum.push_str(" and amount = $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and amount = $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.amount.as_ref().unwrap());
    }
    if req.create_time.is_some(){
        sql_sum.push_str(" and create_time > $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and create_time > $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.create_time.as_ref().unwrap());
    }
    if req.create_end_time.is_some(){
        sql_sum.push_str(" and create_time < $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and create_time < $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.create_end_time.as_ref().unwrap());
    }
    if req.destroy_time.is_some(){
        sql_sum.push_str(" and create_time > $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and create_time > $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.destroy_time.as_ref().unwrap());
    }
    if req.destroy_end_time.is_some(){
        sql_sum.push_str(" and update_time < $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and update_time < $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.destroy_end_time.as_ref().unwrap());
    }
    let total_state = match conn.query(sql_sum.as_str(), &sql_params[..]).await{
        Ok(value) => value,
        Err(error) => {
            warn!("1.get currency list select data failed:{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(error.to_string()));
        }
    };
    let total: i64 = total_state[0].get(0);
    if total <= 0 {
        warn!("The user has not recharged any digital currency");
        return HttpResponse::Ok().json(ResponseBody::<i32>::new_success(Some(0)));
    }
    sql.push_str(" ORDER BY create_time DESC LIMIT $");
    sql.push_str(&(sql_params.len() + 1).to_string());
    sql.push_str(" OFFSET $");
    sql.push_str(&(sql_params.len() + 2).to_string());
    sql_params.push(&req.count);
    sql_params.push(&offset);
    match conn.query(sql.as_str(), &sql_params[..]).await{
        Ok(row) => {
            let mut v = Vec::new();
            for r in row {
                let currency_id = r.get(0);
                let status = r.get(1);
                let amount = r.get(2);
                let create: NaiveDateTime = r.get(3);
                let create_time = create.timestamp();
                let update: NaiveDateTime = r.get(4);
                let late_time = update.timestamp();
                let inner = CurrencyListResponseInner {currency_id, amount, create_time, status, late_time};
                v.push(inner);
            }
            let resp = CurrencyListResponse {total, inner: v};
            return HttpResponse::Ok().json(ResponseBody::<CurrencyListResponse>::new_success(Some(resp)));
        }
        Err(error) => {
            warn!("2.get currency list select data failed:{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(error.to_string()));
        }
    };
}

 /*
 *function: 获取交易列表
 * param：
 * data: 数据库连接句柄
 * req：数据请求结构
 * req_head::http请求头，包含用户认证
 *
 * return :响应数据code=0成功，其他值参考错误列表
 */
 #[derive(Deserialize, Debug)]
 pub struct TransactionListRequest {
     page: i64,
     count: i64,
     transaction_id: Option<String>,
     amount: Option<i64>,
     begin_time: Option<NaiveDateTime>,
     end_time: Option<NaiveDateTime>,
 }

 #[derive(Serialize, Debug)]
struct TransactionListResponseInner {
    transaction_id: String,
    amount: i64,
    create_time: i64,
}
#[derive(Serialize, Debug)]
struct TransactionListResponse {
    total: i64,
    inner: Vec<TransactionListResponseInner>,
}

#[get("/api/public/transaction/list")]
pub async fn get_transaction_list(
    data: web::Data<Pool>,
    req: web::Query<TransactionListRequest>,
    req_head: HttpRequest,
) -> impl Responder {
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-USERID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //连接数据库
    let conn = data.get().await.unwrap();

    let offset: i64 = (req.page - 1) * req.count;
    let mut sql_sum = "SELECT COUNT(*) from exchanges where cloud_user_id = $1".to_string();
    let mut sql = "SELECT transaction_id, amount, create_time from exchanges where cloud_user_id = $1".to_string();
    let mut sql_params: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec![&head_str];

    //插入筛选条件
    if req.transaction_id.is_some(){
        let transaction_sql = format!(" and transaction_id like \'{}%\'",req.transaction_id.as_ref().unwrap());
        sql_sum.push_str(&transaction_sql);
        sql.push_str(&transaction_sql);
    }
    if req.amount.is_some(){
        sql_sum.push_str(" and amount = $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and amount = $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.amount.as_ref().unwrap());
    }
    if req.begin_time.is_some(){
        sql_sum.push_str(" and create_time > $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and create_time > $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.begin_time.as_ref().unwrap());
    }
    if req.end_time.is_some(){
        sql_sum.push_str(" and create_time < $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and create_time < $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.begin_time.as_ref().unwrap());
    }
    let total_state = match conn.query(sql_sum.as_str(), &sql_params[..]).await{
        Ok(value) => value,
        Err(error) => {
            warn!("1.get transaction list select data failed:{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(error.to_string()));
        }
    };
    let total: i64 = total_state[0].get(0);
    if total <= 0{
        warn!("The user has not recharged any transaction");
        return HttpResponse::Ok().json(ResponseBody::<i32>::new_success(Some(0)));
    }
    sql.push_str(" ORDER BY create_time DESC LIMIT $");
    sql.push_str(&(sql_params.len() + 1).to_string());
    sql.push_str(" OFFSET $");
    sql.push_str(&(sql_params.len() + 2).to_string());
    sql_params.push(&req.count);
    sql_params.push(&offset);
    match conn.query(sql.as_str(), &sql_params[..]).await{
        Ok(row) => {
            let mut v = Vec::new();
            for r in row {
                let transaction_id = r.get(0);
                let amount = r.get(1);
                let get_time: NaiveDateTime = r.get(2);
                let create_time = get_time.timestamp();
                let inner = TransactionListResponseInner {transaction_id, amount, create_time};
                v.push(inner);
            }
            let resp = TransactionListResponse {total, inner: v};
            return HttpResponse::Ok().json(ResponseBody::<TransactionListResponse>::new_success(Some(resp)));
        }
        Err(error) => {
            warn!("2.get transaction list select data failed:{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(error.to_string()));
        }
    };
}