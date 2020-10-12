use crate::response::ResponseBody;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use common_structure::digital_currency::DigitalCurrencyWrapper;
use log::{info, warn};
//数据库相关
use chrono::prelude::*;
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
    destroytrans_id: Option<String>,
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
    req_head: HttpRequest,
) -> impl Responder {
    //获取数据库句柄
    let conn = data.get().await.unwrap();
    //获取用户
    let http_head = req_head.headers();
    let head_value = http_head.get("X-USERID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //获取货币信息表数据
    let select_currency = match conn
        .query(
            "SELECT transaction_id, status, owner, amount, create_time, update_time from 
        transactions where currency_id = $1 and cloud_user_id = $2",
            &[&req.currency_id, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("electe success: {:?}", row);
            row
        }
        Err(error) => {
            warn!(
                "1、digistal currency transactions select failed :{:?}!!",
                error
            );
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                error.to_string(),
            ));
        }
    };
    if select_currency.is_empty() {
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
    //获取销毁交易编号
    let destroy_id = match conn.query(
        "SELECT output_id from currencys where input_id = $1 and cloud_user_id = $2",
        &[&req.currency_id, &head_str]
    ).await{
        Ok(row) => {
            info!("electe success: {:?}", row);
            row
        }
        Err(error) => {
            warn!(
                "2、digistal currency currencys select failed :{:?}!!",
                error
            );
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                error.to_string(),
            ));
        }
    };
    let mut destroytrans_id: Option<String>= None;
    if !(destroy_id.is_empty()){
        destroytrans_id = match destroy_id[0].get(0){
            Some(value)=> {
                let des_str: String = value;
                let id = match conn
                .query(
                    "SELECT transaction_id from transactions where currency_id = $1 and cloud_user_id = $2 and status = $3",
                    &[&des_str, &head_str, &"destroy"],
                ).await {
                    Ok(row) => {
                        info!("electe success: {:?}", row);
                        row
                    }
                    Err(error) => {
                        warn!(
                            "3、digistal currency transactions select failed :{:?}!!",
                            error
                        );
                        return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                            error.to_string(),
                        ));
                    }
                };
                if id.is_empty(){
                    None
                }else{
                    let trans_id: Option<String> = id[0].get(0);
                    trans_id
                }
            },
            None => None,
        };
    }
    
    return HttpResponse::Ok().json(ResponseBody::<GetCurrencyResponse>::new_success(Some(
        GetCurrencyResponse {
            currency_id: req.currency_id.clone(),
            transaction_id,
            destroytrans_id,
            status,
            owner,
            amount,
            create_time,
            late_time,
        },
    )));
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
    trans_amount: u64,      //交易金额
    transaction_time: i64,  //交易时间
    input: Vec<(String, u64)>,
    output: Vec<(String, i64)>,
}

#[get("/api/public/transaction/info")]
pub async fn get_exchange_info(
    data: web::Data<Pool>,
    req: web::Query<GetExchangeRequest>,
    req_head: HttpRequest,
) -> impl Responder {
    //获取数据库句柄
    let conn = data.get().await.unwrap();
    //获取用户
    let http_head = req_head.headers();
    let head_value = http_head.get("X-USERID").unwrap();
    let head_str = head_value.to_str().unwrap();

    //获取交易id信息
    let select_exchang = match conn.query(
        "SELECT (trans_info->'inner'->'inputs'), create_time from exchanges where transaction_id = $1 and cloud_user_id = $2",
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
    if select_exchang.is_empty() {
        warn!("This transaction does not exist");
        return HttpResponse::Ok().json(ResponseBody::<()>::currency_is_none());
    }
    let input_info: Vec<DigitalCurrencyWrapper> =
        serde_json::from_value(select_exchang[0].get(0)).unwrap();
    let create: NaiveDateTime = select_exchang[0].get(1);
    let transaction_time = create.timestamp();
    let mut input: Vec<(String, u64)> = Vec::new();
    let mut trans_amount: u64 = 0;
    for currency in input_info.iter() {
        //货币id
        let currency_id = currency.get_body().get_id_str();
        //金额
        let amount: u64 = currency.get_body().get_amount() as u64;
        trans_amount += amount;
        input.push((currency_id, amount))
    }
    let mut output: Vec<(String, i64)> = Vec::new();
    //获取本次交易产生的所有订单
    let select_transid = match conn.query(
        "SELECT currency_id,amount from transactions where transaction_id = $1 and cloud_user_id = $2",
        &[&req.transaction_id, &head_str]
    ).await{
        Ok(row) => {
            info!("electe success: {:?}", row);
            row
        }
        Err(error) => {
            warn!("2、digistal currency transactions select failed :{:?}!!", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(error.to_string()));
        }
    };
    if select_transid.is_empty() {
        warn!("2、This transaction does not exist");
        return HttpResponse::Ok().json(ResponseBody::<()>::currency_is_none());
    }
    for value in select_transid.iter(){
        let currency_id:String = value.get(0);
        let amount: i64 = value.get(1);
        let mut flag:i32 = 0;
        for (id, _) in input.iter(){
            if currency_id == id.clone(){
                flag += 1;
            }
        }
        if flag == 0 {
            output.push((currency_id,amount));
        }
    }

    return HttpResponse::Ok().json(ResponseBody::<GetExchangeResponse>::new_success(Some(
        GetExchangeResponse {
            transaction_id: req.transaction_id.clone(),
            trans_amount,
            transaction_time,
            input,
            output,
        },
    )));
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
    req_head: HttpRequest,
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
    where cloud_user_id = $1"
        .to_string();
    let mut sql_params: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> =
        vec![&head_str];
    if req.currency_id.is_some() {
        let currency_sql = format!(
            " and currency_id like \'{}%\'",
            req.currency_id.as_ref().unwrap()
        );
        sql_sum.push_str(&currency_sql);
        sql.push_str(&currency_sql);
    }
    if req.status.is_some() {
        sql_sum.push_str(" and status = $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and status = $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.status.as_ref().unwrap());
    }
    if req.amount.is_some() {
        sql_sum.push_str(" and amount = $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and amount = $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.amount.as_ref().unwrap());
    }
    if req.create_time.is_some() {
        sql_sum.push_str(" and create_time > $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and create_time > $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.create_time.as_ref().unwrap());
    }
    if req.create_end_time.is_some() {
        sql_sum.push_str(" and create_time < $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and create_time < $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.create_end_time.as_ref().unwrap());
    }
    if req.destroy_time.is_some() {
        sql_sum.push_str(" and create_time > $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and create_time > $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.destroy_time.as_ref().unwrap());
    }
    if req.destroy_end_time.is_some() {
        sql_sum.push_str(" and update_time < $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and update_time < $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.destroy_end_time.as_ref().unwrap());
    }
    let total_state = match conn.query(sql_sum.as_str(), &sql_params[..]).await {
        Ok(value) => value,
        Err(error) => {
            warn!("1.get currency list select data failed:{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                error.to_string(),
            ));
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
    match conn.query(sql.as_str(), &sql_params[..]).await {
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
                let inner = CurrencyListResponseInner {
                    currency_id,
                    amount,
                    create_time,
                    status,
                    late_time,
                };
                v.push(inner);
            }
            let resp = CurrencyListResponse { total, inner: v };
            return HttpResponse::Ok().json(ResponseBody::<CurrencyListResponse>::new_success(
                Some(resp),
            ));
        }
        Err(error) => {
            warn!("2.get currency list select data failed:{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                error.to_string(),
            ));
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
    let mut sql =
        "SELECT transaction_id, amount, create_time from exchanges where cloud_user_id = $1"
            .to_string();
    let mut sql_params: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> =
        vec![&head_str];

    //插入筛选条件
    if req.transaction_id.is_some() {
        let transaction_sql = format!(
            " and transaction_id like \'{}%\'",
            req.transaction_id.as_ref().unwrap()
        );
        sql_sum.push_str(&transaction_sql);
        sql.push_str(&transaction_sql);
    }
    if req.amount.is_some() {
        sql_sum.push_str(" and amount = $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and amount = $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.amount.as_ref().unwrap());
    }
    if req.begin_time.is_some() {
        sql_sum.push_str(" and create_time > $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and create_time > $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.begin_time.as_ref().unwrap());
    }
    if req.end_time.is_some() {
        sql_sum.push_str(" and create_time < $");
        sql_sum.push_str(&(sql_params.len() + 1).to_string());
        sql.push_str(" and create_time < $");
        sql.push_str(&(sql_params.len() + 1).to_string());
        sql_params.push(req.end_time.as_ref().unwrap());
    }
    let total_state = match conn.query(sql_sum.as_str(), &sql_params[..]).await {
        Ok(value) => value,
        Err(error) => {
            warn!("1.get transaction list select data failed:{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                error.to_string(),
            ));
        }
    };
    let total: i64 = total_state[0].get(0);
    if total <= 0 {
        warn!("The user has not recharged any transaction");
        return HttpResponse::Ok().json(ResponseBody::<i32>::new_success(Some(0)));
    }
    sql.push_str(" ORDER BY create_time DESC LIMIT $");
    sql.push_str(&(sql_params.len() + 1).to_string());
    sql.push_str(" OFFSET $");
    sql.push_str(&(sql_params.len() + 2).to_string());
    sql_params.push(&req.count);
    sql_params.push(&offset);
    match conn.query(sql.as_str(), &sql_params[..]).await {
        Ok(row) => {
            let mut v = Vec::new();
            for r in row {
                let transaction_id = r.get(0);
                let amount = r.get(1);
                let get_time: NaiveDateTime = r.get(2);
                let create_time = get_time.timestamp();
                let inner = TransactionListResponseInner {
                    transaction_id,
                    amount,
                    create_time,
                };
                v.push(inner);
            }
            let resp = TransactionListResponse { total, inner: v };
            return HttpResponse::Ok().json(ResponseBody::<TransactionListResponse>::new_success(
                Some(resp),
            ));
        }
        Err(error) => {
            warn!("2.get transaction list select data failed:{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                error.to_string(),
            ));
        }
    };
}

/*
 *function: 交易系统流通货币统计
 * param：
 * data: 数据库连接句柄
 * req：数据请求结构
 * req_head::http请求头，包含用户认证
 *
 * return :响应数据code=0成功，其他值参考错误列表
 */
#[derive(Deserialize, Debug)]
pub struct AllCurrencyRequest {
    begin_time: Option<NaiveDateTime>,
    end_time: Option<NaiveDateTime>,
}
//获取金额
#[derive(Serialize, Debug)]
struct GetNumberResponse {
    circulation_quota: i64, //流通中
    destroy_quota: i64,     //已销毁
    transaction_number: i64,  //交易次数
    circulation_day:Vec<i64>
}

#[get("/api/public/currency/statis")]
pub async fn get_currency_statis(
    data: web::Data<Pool>,
    req: web::Query<AllCurrencyRequest>,
    req_head: HttpRequest
) -> impl Responder {
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-USERID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //连接数据库
    let conn = data.get().await.unwrap();

    //货币状态
    let circulation = String::from("circulate");
    let destroy = String::from("destroy");
    //不同状态金额
    let mut circulation_quota: i64 = 0; //已发行
    let mut destroy_quota: i64 = 0; //已销毁
    //构建数据库命令
    let mut sql_trans = "select count(*)::bigint from exchanges where cloud_user_id = $1".to_string();
    let mut sql_currcy = "select sum(amount)::BIGINT from transactions where cloud_user_id = $1".to_string();
    let mut circulation_params: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> =
        vec![&head_str];
    let mut destroy_params: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> =
        vec![&head_str];

    if req.begin_time.is_some() {
        sql_trans.push_str(" and create_time > $");
        sql_trans.push_str(&(circulation_params.len() + 1).to_string());
        sql_currcy.push_str(" and create_time > $");
        sql_currcy.push_str(&(circulation_params.len() + 1).to_string());
        circulation_params.push(req.begin_time.as_ref().unwrap());
        destroy_params.push(req.begin_time.as_ref().unwrap());
    }
    if req.end_time.is_some() {
        sql_trans.push_str(" and create_time < $");
        sql_trans.push_str(&(circulation_params.len() + 1).to_string());
        sql_currcy.push_str(" and create_time < $");
        sql_currcy.push_str(&(circulation_params.len() + 1).to_string());
        circulation_params.push(req.end_time.as_ref().unwrap());
        destroy_params.push(req.end_time.as_ref().unwrap());
    }
    //交易次数
    let select_count = match conn
    .query(sql_trans.as_str(), &circulation_params[..]).await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("1、get transaction count select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                error.to_string()));
        }
    };
    let transaction_number:i64 = select_count[0].get(0);
    //总金额近7天数据
    let mut circulation_day:Vec<i64> = vec![0,0,0,0,0,0,0];
    let select_amount_day = match conn.query("select time_internal,sum(amount)::bigint from 
    (select to_char(create_time, 'YYYY-MM-DD') as time_internal, * FROM transactions)as foo 
    where (now()-create_time) <= interval '7days' and status = 'circulate' and cloud_user_id = $1 
    group by time_internal ORDER BY time_internal ASC",&[&head_str]).await
    {
        Ok(row) => {
            row
        }
        Err(error) => {
            warn!("2、get 7 days data select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                error.to_string()));
        }
    };
    //获取当前时间天数
    let utc: DateTime<Utc> = Utc::now();
    let days:u32 = utc.day();
    //更新数组
    if !(select_amount_day.is_empty()){
        for value in select_amount_day.iter(){
            let date: String = value.get(0);
            let int_date: u32 = (&date[(date.len()-2)..]).parse::<u32>().unwrap();
            let amount: i64 = value.get(1);
            let i: usize=(days-int_date) as usize;
            if i < 7{
                circulation_day[i] = amount;
            }
        }
    }
    //计算不同状态金额
    sql_currcy.push_str(" and status = $");
    sql_currcy.push_str(&(circulation_params.len() + 1).to_string());
    circulation_params.push(&circulation);
    destroy_params.push(&destroy);
    //获取流通状态金额
    let circulation_state = match conn
    .query(sql_currcy.as_str(), &circulation_params[..]).await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("3、get circulation amount select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                error.to_string()));
        }
    };
    match circulation_state[0].get(0) {
        Some(value) => Some(circulation_quota =value),
        None => None,
    };
    //计算销毁金额
    let destroy_state = match conn
    .query(sql_currcy.as_str(), &destroy_params[..]).await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("4、get destroy amount select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(
                error.to_string()));
        }
    };
    match destroy_state[0].get(0) {
        Some(value) => Some(destroy_quota =value),
        None => None,
    };
    return HttpResponse::Ok().json(ResponseBody::<GetNumberResponse>::new_success(Some(GetNumberResponse{
        circulation_quota, 
        destroy_quota,   
        transaction_number, 
        circulation_day
    })));
}