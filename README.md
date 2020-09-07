# DCDS

### 数字货币生成流程

数字货币的生成流程，区别在于需要通过内部系统生成额度控制位，而不是从数字货币中读取。

### 数据表设计

| Field               | Type         | Comment                             |
| ------------------- | ------------ | ----------------------------------- |
| id                  | varchar(255) | `QuotaControlField` 's key.         |
| currency_id         | varchar(255) | digister currency                   |
| transaction_id      | varchar(255) | transaction id                      |
| state               | varchar(255) | State of DCDS.(possess、transfer)   |
| owner               | varchar(255) | Owner of QCF.                       |
| cloud_user_id       | varchar(255) | user verify id                      |
| create_time         | timestamp    | Create time.                        |
| update_time         | timestamp    | Update time.                        |

### 邻接表
| Field               | Type         | Comment                             |
| ------------------- | ------------ | ----------------------------------- |
| id                  | varchar(255) | `QuotaControlField` 's key.         |
| state               | varchar(255) | State of DCDS.                      |
| owner               | varchar(255) | Owner of QCF.                       |
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
    "currency_id":,// 传入的数字货币
    "in_owner":,  //当前拥有者
    "out_owner":   //需要转移到的拥有者
}
```

响应示例：

```json
{
    "code": 0,
    "message": "success",
    "data": {
        "currency_id",   //转移之后的数字货币
        "out_owner",    //转移之后新的拥有者
        "amount"   //金额
    }
}
```
