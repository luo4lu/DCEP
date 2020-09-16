use crate::config_command;
use clap::ArgMatches;
use deadpool_postgres::{Manager, Pool};
use tokio_postgres::{Config, NoTls};

#[derive(Clone)]
pub struct ConfigPath {
    pub meta_path: String,
}

impl Default for ConfigPath {
    fn default() -> Self{
        Self{
            meta_path: String::from("./digital_currency.json"),
        }
    }
}

pub struct DatabaseAddr {
    pub data_addr: String,
    pub user_name: String,
    pub base_name: String,
}

impl DatabaseAddr {
    pub fn new() -> Self {
        let mut _addr = String::new();
        let mut _name = String::new();
        let mut _base = String::new();
        let matches: ArgMatches = config_command::get_command();
        if let Some(d) = matches.value_of("database") {
            _addr = d.to_string();
        }else {
            _addr = String::from("localhost");
        }
        if let Some(n) = matches.value_of("username") {
            _name = n.to_string();
        }else{
            _name = String::from("postgres");
        }
        if let Some(b) = matches.value_of("basename") {
            _base = b.to_string();
        }else{
            _base = String::from("currencytransaction");
        }
        Self{
            data_addr: _addr,
            user_name: _name,
            base_name: _base,
        }
    }
}

//配置数据库文件
pub fn get_db() -> Pool {
    //配置数据库
    let data_value = DatabaseAddr::new();
    let mut cfg = Config::new();
    cfg.host(&data_value.data_addr); //数据库地址
    cfg.user(&data_value.user_name); //数据库用户名称
    cfg.password("postgres"); //数据库密码
    cfg.dbname(&data_value.base_name); //数据库名称
    let mgr = Manager::new(cfg,NoTls); //通过数据库管理池配置
    Pool::new(mgr, 8)  //设置最大连接池
}