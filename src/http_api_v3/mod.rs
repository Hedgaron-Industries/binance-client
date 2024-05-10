//!
//! The Binance API v3 HTTP client.
//!

pub mod data;
pub mod response;

use chrono::prelude::Utc;
use hmac::Hmac;
use hmac::Mac;
use reqwest::Method;
use reqwest::Url;
use sha2::Sha256;

use self::data::account::get::request::Query as AccountGetQuery;
use self::data::account::get::response::Response as AccountGetResponse;
use self::data::depth::get::request::Query as DepthGetQuery;
use self::data::depth::get::response::Response as DepthGetResponse;
use self::data::exchange_info::get::response::Response as ExchangeInfoGetResponse;
use self::data::klines::get::request::Query as KlinesGetQuery;
use self::data::klines::get::response::Response as KlinesGetResponse;
use self::data::open_orders::delete::request::Query as OpenOrdersDeleteQuery;
use self::data::open_orders::delete::response::Response as OpenOrdersDeleteResponse;
use self::data::open_orders::get::request::Query as OpenOrdersGetQuery;
use self::data::open_orders::get::response::Response as OpenOrdersGetResponse;
use self::data::order::delete::request::Query as OrderDeleteQuery;
use self::data::order::delete::response::Response as OrderDeleteResponse;
use self::data::order::get::request::Query as OrderGetQuery;
use self::data::order::get::response::Response as OrderGetResponse;
use self::data::order::post::request::Query as OrderPostQuery;
use self::data::order::post::response::Response as OrderPostResponse;
use self::data::time::get::response::Response as TimeGetResponse;

use crate::error::Error;

use self::response::Response;

///
/// The Binance API v3 HTTP client.
///
#[derive(Debug, Clone)]
pub struct Client {
    /// The inner HTTP client.
    inner: reqwest::Client,
    /// The Binance authorization API key.
    api_key: Option<String>,
    /// The Binance authorization secret key.
    secret_key: Option<String>,
    /// The request time offset.
    timestamp_offset: i64,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

type Result<T> = ::std::result::Result<T, Error>;

impl Client {
    /// The API base URL.
    const BASE_URL: &'static str = "https://api.binance.com";
    /// The request timestamp offset, which is substituted from the request time to prevent
    /// the `request window missed` error.
    const REQUEST_TIMESTAMP_OFFSET: i64 = 1000;

    ///
    /// Creates an unauthorized client instance.
    ///
    pub fn new() -> Self {
        let mut client = Self {
            inner: reqwest::Client::new(),
            api_key: None,
            secret_key: None,
            timestamp_offset: 0,
        };

        client.timestamp_offset = client.timestamp_offset();
        client
    }

    ///
    /// Creates an authorized client instance.
    ///
    pub fn new_with_auth(api_key: String, secret_key: String) -> Self {
        let mut client = Self {
            inner: reqwest::Client::new(),
            api_key: Some(api_key),
            secret_key: Some(secret_key),
            timestamp_offset: 0,
        };

        client.timestamp_offset = client.timestamp_offset();
        client
    }

    ///
    /// Test connectivity to the Rest API.
    ///
    pub fn ping(&self) -> Result<()> {
        self.execute::<()>(Method::GET, "/api/v3/ping".to_owned())
    }

    ///
    /// Test connectivity to the Rest API and get the current server time.
    ///
    pub fn time(&self) -> Result<TimeGetResponse> {
        self.execute::<TimeGetResponse>(Method::GET, "/api/v3/time".to_owned())
    }

    ///
    /// Current exchange trading rules and symbol information.
    ///
    pub fn exchange_info(&self) -> Result<ExchangeInfoGetResponse> {
        self.execute::<ExchangeInfoGetResponse>(Method::GET, "/api/v3/exchangeInfo".to_owned())
    }

    ///
    /// Kline/candlestick bars for a symbol.
    /// Klines are uniquely identified by their open time.
    ///
    pub fn klines(&self, request: KlinesGetQuery) -> Result<KlinesGetResponse> {
        self.execute::<KlinesGetResponse>(
            Method::GET,
            format!("/api/v3/klines?{}", request.to_string()),
        )
    }

    ///
    /// The real-time market depth.
    ///
    pub fn depth(&self, request: DepthGetQuery) -> Result<DepthGetResponse> {
        self.execute::<DepthGetResponse>(
            Method::GET,
            format!("/api/v3/depth?{}", request.to_string()),
        )
    }

    ///
    /// Get the account info and balances.
    ///
    pub fn account_get(&self, mut request: AccountGetQuery) -> Result<AccountGetResponse> {
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or(Error::AuthorizationKeysMissing)?;

        request.timestamp -= self.timestamp_offset;

        let mut params = request.to_string();
        params += &format!("&signature={}", Self::signature(&params, secret_key));

        self.execute_signed::<AccountGetResponse>(
            Method::GET,
            format!("/api/v3/account?{}", params),
        )
    }

    ///
    /// Get the account open orders.
    ///
    pub fn open_orders_get(
        &self,
        mut request: OpenOrdersGetQuery,
    ) -> Result<OpenOrdersGetResponse> {
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or(Error::AuthorizationKeysMissing)?;

        request.timestamp -= self.timestamp_offset;

        let mut params = request.to_string();
        params += &format!("&signature={}", Self::signature(&params, secret_key));

        self.execute_signed::<OpenOrdersGetResponse>(
            Method::GET,
            format!("/api/v3/openOrders?{}", params),
        )
    }

    ///
    /// Delete the account open orders.
    ///
    pub fn open_orders_delete(
        &self,
        mut request: OpenOrdersDeleteQuery,
    ) -> Result<OpenOrdersDeleteResponse> {
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or(Error::AuthorizationKeysMissing)?;

        request.timestamp -= self.timestamp_offset;

        let mut params = request.to_string();
        params += &format!("&signature={}", Self::signature(&params, secret_key));

        self.execute_signed::<OpenOrdersDeleteResponse>(
            Method::DELETE,
            format!("/api/v3/openOrders?{}", params),
        )
    }

    ///
    /// Check an order's status.
    ///
    pub fn order_get(&self, mut request: OrderGetQuery) -> Result<OrderGetResponse> {
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or(Error::AuthorizationKeysMissing)?;

        request.timestamp -= self.timestamp_offset;

        let mut params = request.to_string();
        params += &format!("&signature={}", Self::signature(&params, secret_key));

        self.execute_signed::<OrderGetResponse>(Method::GET, format!("/api/v3/order?{}", params))
    }

    ///
    /// Send in a new order.
    ///
    pub fn order_post(&self, mut request: OrderPostQuery) -> Result<OrderPostResponse> {
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or(Error::AuthorizationKeysMissing)?;

        request.timestamp -= self.timestamp_offset;

        let mut params = request.to_string();
        params += &format!("&signature={}", Self::signature(&params, secret_key));

        self.execute_signed::<OrderPostResponse>(Method::POST, format!("/api/v3/order?{}", params))
    }

    ///
    /// Cancel an active order.
    ///
    pub fn order_delete(&self, mut request: OrderDeleteQuery) -> Result<OrderDeleteResponse> {
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or(Error::AuthorizationKeysMissing)?;

        request.timestamp -= self.timestamp_offset;

        let mut params = request.to_string();
        params += &format!("&signature={}", Self::signature(&params, secret_key));

        self.execute_signed::<OrderDeleteResponse>(
            Method::DELETE,
            format!("/api/v3/order?{}", params),
        )
    }

    ///
    /// Test new order creation and signature/recvWindow long.
    /// Creates and validates a new order but does not send it into the matching engine.
    ///
    pub fn order_post_test(&self, mut request: OrderPostQuery) -> Result<OrderPostResponse> {
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or(Error::AuthorizationKeysMissing)?;

        request.timestamp -= self.timestamp_offset;

        let mut params = request.to_string();
        params += &format!("&signature={}", Self::signature(&params, secret_key));

        self.execute_signed::<OrderPostResponse>(
            Method::POST,
            format!("/api/v3/order/test?{}", params),
        )
    }

    ///
    /// Executes an unauthorized request.
    ///
    fn execute<T>(&self, method: Method, url: String) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = Self::BASE_URL.to_owned() + url.as_str();

        let response = self
            .inner
            .execute(
                self.inner
                    .request(
                        method,
                        Url::parse(&url).map_err(|error| Error::UrlParsing(error, url))?,
                    )
                    .build()
                    .map_err(Error::RequestBuilding)?,
            )
            .map_err(Error::RequestExecution)?
            .text()
            .map_err(Error::ResponseReading)?;
        let response: Response<T> = serde_json::from_str(response.as_str())
            .map_err(|error| Error::ResponseParsing(error, response))?;

        match response {
            Response::Ok(response) => Ok(response),
            Response::Error(error) => Err(Error::ResponseError(error)),
        }
    }

    ///
    /// Executes an authorized request.
    ///
    fn execute_signed<T>(&self, method: Method, url: String) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or(Error::AuthorizationKeysMissing)?;

        let url = Self::BASE_URL.to_owned() + url.as_str();

        let response = self
            .inner
            .execute(
                self.inner
                    .request(
                        method,
                        Url::parse(&url).map_err(|error| Error::UrlParsing(error, url))?,
                    )
                    .header("X-MBX-APIKEY", api_key.to_owned())
                    .build()
                    .map_err(Error::RequestBuilding)?,
            )
            .map_err(Error::RequestExecution)?
            .text()
            .map_err(Error::ResponseReading)?;
        let response: Response<T> = serde_json::from_str(response.as_str())
            .map_err(|error| Error::ResponseParsing(error, response))?;

        match response {
            Response::Ok(response) => Ok(response),
            Response::Error(error) => Err(Error::ResponseError(error)),
        }
    }

    ///
    /// Generates an HMAC signature for authorized requests.
    ///
    fn signature(params: &str, secret_key: &str) -> String {
        hex::encode({
            let mut hmac: Hmac<Sha256> =
                Hmac::new_from_slice(secret_key.as_bytes()).expect("HMAC is valid");
            hmac.update(params.as_bytes());
            hmac.finalize().into_bytes()
        })
    }

    ///
    /// Calculates the request timestamp offsets between the system time and Binance time.
    ///
    fn timestamp_offset(&self) -> i64 {
        let system_time = Utc::now().timestamp_millis();
        let request_time = std::time::Instant::now();
        let binance_time = self.time().expect("Time request").server_time
            - (request_time.elapsed().as_millis() as i64) / 2;

        (system_time - binance_time) + Self::REQUEST_TIMESTAMP_OFFSET
    }
}
