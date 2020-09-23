use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseBody<T> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ResponseBody<T> {
    //所有响应成功
    pub fn new_success(data: Option<T>) -> Self {
        Self {
            code: 0,
            message: String::from("success"),
            data,
        }
    }
    ///返回部分unwrap错误信息
    pub fn return_unwrap_error(data: String) -> Self {
        ResponseBody {
            code: 90001,
            message: String::from(data),
            data: None,
        }
    }
    ///读取配置文件打开失败
    pub fn new_file_error() -> Self {
        ResponseBody {
            code: 90002,
            message: String::from("file open or write or read error."),
            data: None,
        }
    }
    ///字符转换相关产生的错误返回
    pub fn new_str_conver_error() -> Self {
        ResponseBody {
            code: 90003,
            message: String::from("char conversion error"),
            data: None,
        }
    }
    ///数字货币交易体检查错误
    pub fn transaction_error() -> Self {
        ResponseBody {
            code: 90004,
            message: String::from("数字货币交易体检查错误"),
            data: None,
        }
    }
    ///数字货币交易状态不满足交易
    pub fn currency_state_error() -> Self {
        ResponseBody {
            code: 90005,
            message: String::from("数字货币交易状态不满足交易"),
            data: None,
        }
    }
    ///用户信息错误，无此货币
    pub fn currency_is_none() -> Self {
        ResponseBody {
            code: 90006,
            message: String::from("用户信息错误，无此货币"),
            data: None,
        }
    }
}