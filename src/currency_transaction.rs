use crate::config::ConfigPath;
use crate::response::ResponseBody;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use asymmetric_crypto::hasher::sha3::Sha3;
use asymmetric_crypto::keypair;
use asymmetric_crypto::prelude::Keypair;
use common_structure::digital_currency::DigitalCurrencyWrapper;
use common_structure::transaction::TransactionWrapper;
use dislog_hal::Bytes;
use hex::{FromHex, ToHex};
use kv_object::sm2::{CertificateSm2, KeyPairSm2};
use serde::{Deserialize, Serialize};
use log::{info, warn};
use rand::thread_rng;
use tokio::fs::File;
use tokio::prelude::*;
use deadpool_postgres::Pool;

/*
 *function: 数字货币交易
 * param：
 * data: 数据库连接句柄
 * req：数据请求结构
 * req_head::http请求头，包含用户认证
 *
 * return :响应数据code=0成功，其他值参考错误列表
 */

//请求结构
#[derive(Deserialize, Serialize, Debug)]
pub struct DcdsRequestBody {
    curr_transaction:String,  //数字货币交易体Transaction
}
//响应数据
#[derive(Debug, Serialize)]
pub struct DcdsResponsetBody {
    currency:Vec<String>,
}

#[post("/api/public/transaction")]
pub async fn digistal_transaction(
    data: web::Data<Pool>,
    config: web::Data<ConfigPath>,
    req: web::Json<DcdsRequestBody>,
    req_head: HttpRequest,
) -> impl Responder {
    //获取用户
    let http_head = req_head.headers();
    let head_value = http_head.get("X-USERID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //获取数据库句柄
    let mut conn = data.get().await.unwrap();
    //随机数生成器
    let mut rng = thread_rng();
    //read file for get seed
    let mut file = match File::open(&config.meta_path).await{
        Ok(f) => {
            info!("{:?}", f);
            f
        }
        Err(e) => {
            warn!("file open failed:{:?}", e);
            return HttpResponse::Ok().json(ResponseBody::<()>::new_file_error());
        }
    };
    //read json file to string
    let mut contents = String::new();
    match file.read_to_string(&mut contents).await {
        Ok(s) => {
            info!("{:?}",s);
            s
        }
        Err(e) => {
            warn!("read file to string failed:{:?}", e);
            return HttpResponse::Ok().json(ResponseBody::<()>::new_str_conver_error());
        }
    };
    //deserialize to the specified data format
    let keypair_value: keypair::Keypair<
        [u8; 32],
        Sha3,
        dislog_hal_sm2::PointInner,
        dislog_hal_sm2::ScalarInner,
    > = match serde_json::from_str(&contents) {
        Ok(de) => {
            info!("{:?}", de);
            de
        }
        Err(e) => {
            warn!("Keypair generate failed:{:?}", e);
            return HttpResponse::Ok().json(ResponseBody::<()>::new_str_conver_error());
        }
    };
    //pass encode hex conversion get seed 
    let seed: [u8; 32] = keypair_value.get_seed();
    //get digital signature
    let keypair_sm2: KeyPairSm2 = KeyPairSm2::generate_from_seed(seed).unwrap();

    //解析出入的交易体
    let vec = Vec::<u8>::from_hex(req.curr_transaction.clone()).unwrap();
    let transaction_wrapper = TransactionWrapper::from_bytes(&vec).unwrap();
    let json_transaction = serde_json::to_value(&transaction_wrapper).unwrap();
    
    //let transaction_wrapper = TransactionWrapper::new(transaction_ok);
    //获取交易体中的输出数字货币
    let input: Vec<DigitalCurrencyWrapper> = transaction_wrapper.get_inputs().clone();
    //交易体检查
    let trans_bool = transaction_wrapper.check_validated();
    if trans_bool != true {
        warn!("transaction body error");
        return HttpResponse::Ok().json(ResponseBody::<()>::transaction_error());
    }
    //获取交易id
    let transaction_id: String = transaction_wrapper.get_id_str();
    //货币使用状态
    let possess = String::from("circulate");
    let transfer = String::from("destroy");
    //为存储输入输出分析存储货币id
    let mut input_currency_id:Vec<String> = Vec::new();
    let mut output_currency_id:Vec<String> = Vec::new();
    //事务开始
    let transaction_sql = conn.transaction().await.unwrap();
    //输入的数字货币
    for currency in input.iter() {
        //货币id
        let currency_id = currency.get_body().get_id_str();
        //金额
        let amount:i64 = currency.get_body().get_amount() as i64;
        //所有者
        let owner: CertificateSm2 = currency.get_body().get_owner().clone();
        let owner_str: String = owner.to_bytes().encode_hex::<String>();
        //数据表检查数字货币id
        let select_currency = match transaction_sql.query(
            "SELECT status from transactions where currency_id = $1 and cloud_user_id = $2",
            &[&currency_id, &head_str]
        ).await{
            Ok(row) => {
                info!("electe success: {:?}", row);
                row
            }
            Err(error) => {
                warn!("1、digistal currency select failed :{:?}!!", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(error.to_string()));
            }
        };
        if select_currency.is_empty() {
            transaction_sql.query("INSERT INTO transactions (currency_id, transaction_id, status, owner, amount, cloud_user_id,
                create_time, update_time) VALUES ($1, $2, $3, $4, $5 ,$6,now(), now())",
                &[&currency_id, &transaction_id, &transfer, &owner_str,&amount, &head_str]).await.unwrap();
        }else{
            let st: String = select_currency[0].get(0);
            if transfer ==st {
                warn!("this digistal currency Traded transfer");
                return HttpResponse::Ok().json(ResponseBody::<()>::currency_state_error());
            }else{
                transaction_sql.query("UPDATE transactions SET status = $1, owner=$2, update_time = now() WHERE currency_id = $3 and cloud_user_id = $4",
                &[&transfer, &owner_str, &currency_id, &head_str]).await.unwrap();
            }
        }
        input_currency_id.push(currency_id);
    }
    //本次交易总金额
    let mut total_amount: i64 = 0;
    //输出的数字货币
    let output_vec: Vec<DigitalCurrencyWrapper> = transaction_wrapper.gen_new_currency(&keypair_sm2,&mut rng);
    for currency in output_vec.iter(){
        //货币id
        let currency_id = currency.get_body().get_id_str();
        //金额
        let amount:i64 = currency.get_body().get_amount() as i64;
        total_amount += amount;
        //所有者
        let owner: CertificateSm2 = currency.get_body().get_owner().clone();
        let owner_str: String = owner.to_bytes().encode_hex::<String>();
        
        transaction_sql.query("INSERT INTO transactions (currency_id, transaction_id, status, owner, amount,cloud_user_id,
            create_time, update_time) VALUES ($1, $2, $3, $4, $5 ,$6,now(), now())",
            &[&currency_id, &transaction_id, &possess, &owner_str,&amount, &head_str]).await.unwrap();
        
        output_currency_id.push(currency_id);
    }
    //更新交易结构表
    for output_id in output_currency_id.iter() {
        for input_id in input_currency_id.iter() {
            match transaction_sql.query("INSERT INTO currencys (output_id, input_id, cloud_user_id,
            create_time, update_time) VALUES ($1, $2, $3, now(), now())",
            &[&output_id, &input_id, &head_str]).await{
                Ok(_) => {
                    info!("currencys table transaction struct success");
                }
                Err(error) => {
                    warn!("2、currencys table transaction struct :{:?}!!", error);
                    return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(error.to_string()));
                }
            };
        }
    }
    //记录本次交易
    match transaction_sql.query("INSERT INTO exchanges (transaction_id, cloud_user_id, trans_info, trans_bin, amount, 
        create_time, update_time) VALUES ($1, $2, $3, $4, $5, now(), now())",
    &[&transaction_id, &head_str, &json_transaction, &req.curr_transaction, &total_amount]).await{
        Ok(_) => {
            //事务结束
            transaction_sql.commit().await.unwrap();
            info!("exchanges table transaction struct success");
            return HttpResponse::Ok().json(ResponseBody::<DcdsResponsetBody>::new_success(Some(DcdsResponsetBody{
                currency: output_currency_id.clone()
            })));
        }
        Err(error) => {
            warn!("3、exchanges table transaction struct :{:?}!!", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::return_unwrap_error(error.to_string()));
        }
    };
}