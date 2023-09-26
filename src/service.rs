pub mod service {

    use http::HeaderValue;
    use reqwest::header::ACCEPT;
    use reqwest::header::CONTENT_TYPE;

    mod endpoints {
        // Private crate to hold all types that the user shouldn't have to interact with.
        use crate::types::RegisterResponse;
        use serde::Deserialize;
        // Trait for API types. Has to be public due to trait bounds limitations on webex API, but hidden
        // in a private crate so users don't see it.
        pub trait Gettable {
            const API_ENDPOINT: &'static str; // Endpoint to query to perform an HTTP GET request with or without an Id.
        }

        impl Gettable for RegisterResponse {
            const API_ENDPOINT: &'static str = "register";
        }

        #[derive(Deserialize)]
        pub struct ListResult<T> {
            pub items: Vec<T>,
        }
    }

    use crate::types::{Register,RegisterResponse};
    use http::HeaderMap;
    use reqwest::Client;

    use self::endpoints::Gettable;

    // Singleton class
    // ----------------------------------------------------------------------------
    pub struct WebSocketClient {
        host: String,
        port: u16,
        _client: Client,
        _headers: HeaderMap,
    }

    impl WebSocketClient {
        pub fn new(host: String, port: u16) -> WebSocketClient {
            let mut headers = HeaderMap::new();
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_str("application/json").unwrap(),
            );
            headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());

            WebSocketClient { host, port, _client: Client::new(), _headers: headers }
        }
        
        // Webex client specific functionality.
        // ----------------------------------------------------------------------------
        pub async fn register(&self, user_id: u16) -> RegisterResponse {
            let response = self._client
                .post(format!("http://{}:{}/{}", self.host, self.port, RegisterResponse::API_ENDPOINT))
                .headers(self._headers.clone())
                .json(&Register{ user_id: user_id })
                .send()
                .await
                .unwrap();
        
                self.review_status(&response);
        
            let message = response
                .json::<RegisterResponse>()
                .await
                .expect("failed to convert struct from json");
        
            return message;
        }
    
        // Review the status for the response.
        // ----------------------------------------------------------------------------
        pub fn review_status(&self, response: &reqwest::Response) -> () {
            match response.status() {
                reqwest::StatusCode::OK => {
                    log::debug!("Succesful request: {:?}", response)
                }
                reqwest::StatusCode::NOT_FOUND => {
                    log::debug!("Got 404! Haven't found resource!: {:?}", response)
                }
                _ => {
                    log::error!("Got 404! Haven't found resource!: {:?}", response)
                }
            }
        }

    }
}
