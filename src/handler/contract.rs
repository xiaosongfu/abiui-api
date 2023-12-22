use spin_sdk::http::{IntoResponse, Json};
use spin_sdk::pg::{Connection, Decode, Row};

use crate::handler;

#[derive(serde::Deserialize)]
pub struct UploadReq {
    pub address: String,
    pub chain_id: String,
    pub abi: String,
    pub abi_pretty: String,
    pub html: String,
}

#[derive(serde::Serialize)]
pub struct UploadResp {
    pub url: String,
}

// `POST /contract/upload`
pub fn upload(
    req: http::Request<Json<UploadReq>>,
    _: spin_sdk::http::Params,
) -> anyhow::Result<impl IntoResponse> {
    let update_at = 0i64;
    let address = req.body().address.to_string();
    let network = chain_id_to_name(req.body().chain_id.as_str());

    // check if network is supported
    if network.is_empty() {
        return Err(anyhow::anyhow!("network not supported"));
    }

    // check if user is login
    let wallet = if let Ok(wallet) = handler::whoami(&req) {
        wallet
    } else {
        "".to_string()
    };
    // let Ok(wallet) = handler::whoami(&req) else {
    //     "".to_string()
    // };

    // connect to database
    let db_url = std::env::var("DB_URL")?;
    let conn = Connection::open(&db_url)?;

    // query if contract exists by address and network
    let rowset = conn.query(
        format!(
            r##"SELECT id,owner FROM public."contracts" WHERE address = '{}' AND network = '{}'"##,
            address.as_str(),
            network.as_str()
        )
        .as_str(),
        &[],
    )?;
    if rowset.rows.is_empty() {
        // executing sql statement
        // conn.execute(
        //     r##"INSERT INTO public."contracts" (address, network, abi, abi_pretty, html, owner, upload_at) VALUES (?, ?, ?, ?, ?, ?)"##,
        //     &vec![
        //         ParameterValue::Str(address.clone()),
        //         ParameterValue::Str(network.clone()),
        //         ParameterValue::Str(req.body().abi.to_string()),
        //         ParameterValue::Str(req.body().abi_pretty.to_string()),
        //         ParameterValue::Str(req.body().html.to_string()),
        //         ParameterValue::Str(wallet),
        //         ParameterValue::Int64(update_at),
        //     ],
        // )?;
        conn.execute(format!(r##"INSERT INTO public."contracts" (address, network, abi, abi_pretty, html, owner, upload_at) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {})"##, address.as_str(), network.as_str(), req.body().abi.as_str(), req.body().abi_pretty.as_str(), req.body().html.as_str(), wallet, update_at).as_str(), &[])?;
    } else {
        let row = &rowset.rows[0];
        let id = i64::decode(&row[0])?;
        let owner = Option::<String>::decode(&row[1])?;
        // contract's owner is empty or current user, update it
        if owner.is_none() || owner.unwrap() == wallet {
            conn.execute(
                format!(
                    r##"UPDATE public."contracts" SET abi = '{}', abi_pretty = '{}', html = '{}', upload_at = {} WHERE id = {}"##,
                    req.body().abi.as_str(),
                    req.body().abi_pretty.as_str(),
                    req.body().html.as_str(),
                    update_at,
                    id
                )
                .as_str(),
                &[],
            )?;
        } else {
            return Err(anyhow::anyhow!("has been uploaded by other user"));
        }
    }

    handler::json(&UploadResp {
        url: format!("https://{}.{}.abiui.dev", address, network),
    })
}

// `DELETE /contract/delete/:id`
pub fn delete(
    req: http::Request<()>,
    params: spin_sdk::http::Params,
) -> anyhow::Result<impl IntoResponse> {
    let id = params.get("id").unwrap();
    let wallet = handler::whoami(&req)?;

    // connect to database
    let db_url = std::env::var("DB_URL")?;
    let conn = Connection::open(&db_url)?;

    conn.execute(
        format!(
            r##"DELETE FROM public."contracts" WHERE id = {} AND owner = '{}'"##,
            id,
            wallet.as_str()
        )
        .as_str(),
        &[],
    )?;

    handler::ok()
}

// `PUT /contract/update_alias/:id/:alias
pub fn update_alias(
    req: http::Request<()>,
    params: spin_sdk::http::Params,
) -> anyhow::Result<impl IntoResponse> {
    let id = params.get("id").unwrap();
    let alias = params.get("alias").unwrap();
    let wallet = handler::whoami(&req)?;

    // connect to database
    let db_url = std::env::var("DB_URL")?;
    let conn = Connection::open(&db_url)?;

    conn.execute(
        format!(
            r##"UPDATE public."contracts" SET alias = '{}' WHERE id = {} AND owner = '{}'"##,
            alias,
            id,
            wallet.as_str()
        )
        .as_str(),
        &[],
    )?;

    handler::ok()
}

#[derive(Debug, Clone, serde::Serialize)]
struct Contract {
    id: i64,
    alias: Option<String>,
    address: String,
    network: String,
    upload_at: i64,
}

impl TryFrom<&Row> for Contract {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Contract {
            id: i64::decode(&row[0])?,
            alias: Option::<String>::decode(&row[1])?,
            address: String::decode(&row[2])?,
            network: String::decode(&row[3])?,
            upload_at: i64::decode(&row[4])?,
        })
    }
}

// `GET /contract/my_contracts`
pub fn my_contracts(
    req: http::Request<()>,
    _: spin_sdk::http::Params,
) -> anyhow::Result<impl IntoResponse> {
    let wallet = handler::whoami(&req)?;

    // connect to database
    let db_url = std::env::var("DB_URL")?;
    let conn = Connection::open(&db_url)?;

    let rowset = conn.query(format!(r##"SELECT id,alias,address,network,upload_at FROM public."contracts" WHERE owner = '{}'"##, wallet.as_str()).as_str(), &[])?;

    let mut contracts = Vec::new();
    for row in rowset.rows {
        contracts.push(Contract::try_from(&row)?);
    }
    handler::json(&contracts)
}

fn chain_id_to_name(chain_id: &str) -> String {
    match chain_id {
        "1" => "mainnet",
        "3" => "ropsten",
        "4" => "rinkeby",
        "5" => "goerli",
        "42" => "kovan",
        _ => "",
    }
    .to_string()
}
