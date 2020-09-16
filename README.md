# DCDS

### 数字货币生成流程

数字货币的生成流程，区别在于需要通过内部系统生成额度控制位，而不是从数字货币中读取。

### 数据表设计

| Field               | Type         | Comment                             |
| ------------------- | ------------ | ----------------------------------- |
| currency_id         | text         | digister currency                   |
| transaction_id      | varchar(255) | transaction id                      |
| state               | varchar(255) | State of DCDS.(possess、transfer)   |
| owner               | varchar(255) | Owner of QCF.                       |
| amount              | BIGINT       | currency amount.                    |
| cloud_user_id       | varchar(255) | user verify id                      |
| create_time         | timestamp    | Create time.                        |
| update_time         | timestamp    | Update time.                        |

### 邻接表
| Field               | Type         | Comment                             |
| ------------------- | ------------ | ----------------------------------- |
| output_id           | varchar(255) | at present digister currency id     |
| input_id            | varchar(255) | last digister currency  id          |
| cloud_user_id       | varchar(255) | user verify id                      |
| create_time         | timestamp    | Create time.                        |
| update_time         | timestamp    | Update time.                        |

### 交易表
| Field               | Type         | Comment                             |
| ------------------- | ------------ | ----------------------------------- |
| transaction_id      | varchar(255) | transaction id                      |
| trans_info          | jsonb        | transaction info.                   |
| trans_bin           | text         | transaction info bin.               |
| create_time         | timestamp    | Create time.                        |
| update_time         | timestamp    | Update time.                        |


### API设计

### 错误样例表

| code  | message                             | api                              |
| ----- | ----------------------------------- | -------------------------------- |
|     0 | 成功返回                             | all                              |
| 90001 | 返回部分unwrap错误信息               | post /api/public/transaction     |
| 90002 | 读取配置文件打开失败                  | post /api/public/transaction     |
| 90003 | 字符转换相关产生的错误返回            | post /api/public/transaction      |
| 90004 | 数字货币交易体检查错误                | post /api/public/transaction     |
| 90005 | 数字货币交易状态不满足交易            | post /api/public/transaction     |

#### 自有证书管理


#### 进行交易

HTTP 请求：`POST /api/public/transaction`

请求头：

```
X-USERID: uuid
```

请求示例：

```json
{
    "transaction":[
        "",
        "",   //数字货币交易体
    ]
}
```

响应示例：

```json
{
    "code": 0,
    "message": "success",
    "data": [
        "currency",//转移之后的数字货币
        "currency"
    ]
}
```

#### 获取货币信息

HTTP 请求：`GET /api/public/currency/info?currency_id=`

请求头：

```
X-USERID: uuid
```

响应示例：

```json
{
    "code": 0,
    "message": "success",
    "data": {
        "currency_id": "", //转移之后的数字货币
        "state": "",
        "owner": "",
        "create_time": "timestamp",
        "late_time": "Option<timestamp>"  //state为transfer则存在
    }
}
```
