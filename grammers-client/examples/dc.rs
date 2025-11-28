//! Example to print the ID and title of all the dialogs.
//!
//! The `TG_ID` and `TG_HASH` environment variables must be set (learn how to do it for
//! [Windows](https://ss64.com/nt/set.html) or [Linux](https://ss64.com/bash/export.html))
//! to Telegram's API ID and API hash respectively.
//!
//! Then, run it as:
//!
//! ```sh
//! cargo run --example dialogs
//! ```

#![allow(deprecated)]

use grammers_client::{Client};
use grammers_mtsender::{InvocationError, SenderPool};
use grammers_session::storages::MemorySession;
use grammers_session::Session;
use grammers_tl_types::functions::auth::SendCode;
use grammers_tl_types::types::CodeSettings;
use log::info;
use simple_logger::SimpleLogger;
use std::sync::Arc;
use tokio::runtime;
use grammers_tl_types::enums::help::CountriesList;
use grammers_tl_types::functions::help::GetCountriesList;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

async fn async_main() -> Result<()> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let api_id = 2040;
    let api_hash = "b18441a1ff607e10a989891a5462e627";
    let session = Arc::new(MemorySession::default());

    let pool = SenderPool::new(Arc::clone(&session), api_id);
    let client = Client::new(&pool);
    let SenderPool { runner, .. } = pool;
    let _ = tokio::spawn(runner.run());

    let phone = "6807793198"; // random phone
    let request = SendCode {
        phone_number: phone.to_string(),
        api_id,
        api_hash: api_hash.to_string(),
        settings: CodeSettings {
            allow_flashcall: false,
            current_number: false,
            allow_app_hash: false,
            allow_missed_call: false,
            allow_firebase: false,
            logout_tokens: None,
            token: None,
            app_sandbox: None,
            unknown_number: false,
        }
        .into(),
    };

    let dc = match client.invoke(&request).await {
        Ok(_) => session.home_dc_id(),
        Err(InvocationError::Rpc(err)) if err.code == 303 => err.value.unwrap() as i32,
        Err(e) => return Err(e.into()),
    };
    info!("DC: {}", dc);
    let req = GetCountriesList {
        lang_code: "en".parse().unwrap(),
        hash: 0,
    };
    let countries = match client.invoke(&req).await{
        Ok(resp)=> {resp.clone()},
        Err(e) => return Err(e.into()),
    };
    match countries {
        CountriesList::NotModified => {}
        CountriesList::List(list) => {
            list.countries.iter().for_each(|country| {
                info!("Country: {:?}", country);
            })
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}
