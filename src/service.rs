pub const WS_BACKEND: &str = "https://172.172.194.77/";

pub mod Service {

    use http::HeaderValue;
    use reqwest::header::ACCEPT;
    use reqwest::header::CONTENT_TYPE;

    mod endpoints {
        // Private crate to hold all types that the user shouldn't have to interact with.
        use crate::types::{Health, Register};
        use serde::Deserialize;
        // Trait for API types. Has to be public due to trait bounds limitations on webex API, but hidden
        // in a private crate so users don't see it.
        pub trait Gettable {
            const API_ENDPOINT: &'static str; // Endpoint to query to perform an HTTP GET request with or without an Id.
        }

        impl Gettable for Message {
            const API_ENDPOINT: &'static str = "register";
        }

        impl Gettable for Health {
            const API_ENDPOINT: &'static str = "health";
        }

        #[derive(Deserialize)]
        pub struct ListResult<T> {
            pub items: Vec<T>,
        }
    }

    use crate::service::WEBEX_URI;
    use crate::types::{self, Message};
    use http::HeaderMap;
    use reqwest::Client;
    use std::{mem::MaybeUninit, sync::Once};

    use self::endpoints::Gettable;

    // Singleton class
    // ----------------------------------------------------------------------------
    pub struct Service {
        client: Client,
        headers: HeaderMap,
    }

    impl Service {
        fn get_instance() -> &'static Service {
            static mut instance: MaybeUninit<Service> = MaybeUninit::uninit();
            static once: Once = Once::new();

            unsafe {
                once.call_once(|| {
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").unwrap(),
                    );
                    headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());

                    instance.write(Service {
                        client: Client::new(),
                        headers,
                    });
                });

                instance.assume_init_ref()
            }
        }
    }

    // Webex client specific functionality.
    // ----------------------------------------------------------------------------
    pub async fn register(client_id: usize) -> Message {
        let client_service = Service::get_instance();
        let response = client_service
            .client
            .post(format!("{}{}", WS_BACKEND, Message::Register))
            .headers(client_service.headers.clone())
            .json(&message_out)
            .bearer_auth(token)
            .send()
            .await
            .unwrap();

        review_status(&response);

        let message = response
            .json::<Message>()
            .await
            .expect("failed to convert struct from json");

        return message;
    }
}
