create database currencytransaction;

create table transactions(
    currency_id text NOT NULL,
    transaction_id varchar(255) NOT NULL,
    status varchar(255) NOT NULL,
    owner varchar(255) NOT NULL,
    amount BIGINT NOT NULL,
    cloud_user_id varchar(255) NOT NULL,
    create_time timestamp NOT NULL,
    update_time timestamp NOT NULL
);
create table currencys(
    output_id varchar(255)  NOT NULL,
    input_id varchar(255) NOT NULL,
    cloud_user_id varchar(255) NOT NULL,
    create_time timestamp NOT NULL,
    update_time timestamp NOT NULL
);

create table exchanges(
    transaction_id varchar(255) NOT NULL,
    cloud_user_id VARCHAR(255) NOT NULL,
    trans_info jsonb NOT NULL,
    trans_bin text NOT NULL,
    amount BIGINT NOT NULL,
    create_time timestamp NOT NULL,
    update_time timestamp NOT NULL
);