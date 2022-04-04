use anyhow::Result;
use rocket::serde::json::serde_json;

use crate::{
    types::kratos::{Identity, IdentityOidc},
    utils::http::get,
};

///structure containing the data of pusher
pub struct KratosData<'a> {
    pub provider_id: &'a str,
    pub user_id: i64,
}

///iter over the is of all identity and return the token of the pusher if it exist.
async fn get_kratos_token(
    identities: &[Identity],
    addr: &str,
    user_data: KratosData<'_>,
) -> Result<Option<(String, String)>> {
    info!("asking kratos for user token.");
    for identity in identities {
        let addr_id = addr.to_owned() + "/identities/" + &identity.id + "?include_credential=oidc";
        let resp = get(&addr_id).await?;
        let body = resp.text().await?;
        let data = serde_json::from_str::<IdentityOidc>(&body)?;
        let provider = data.credentials.oidc.config.providers.iter().find(|f| {
            f.subject == user_data.user_id.to_string() && f.provider == user_data.provider_id
        });
        if let Some(p) = provider {
            return Ok(Some((
                p.initial_access_token.to_owned(),
                identity.id.to_owned(),
            )));
        }
    }
    Ok(None)
}

///check if the pusher of the event is regitred in cratos and return it's token
pub async fn security_check(
    addr: String,
    user_data: KratosData<'_>,
) -> Result<Option<(String, String)>> {
    info!("asking kratos for user id.");
    let query = format!(
        "credentials.identifier={}:{}",
        user_data.provider_id, user_data.user_id
    );
    let query_addr = addr.clone() + "/identities" + "?" + &query;
    let resp = get(&query_addr).await?;
    let text = resp.text().await?;
    let identities = serde_json::from_str::<Vec<Identity>>(&text)?;
    if let Some((token, id)) = get_kratos_token(&identities, &addr, user_data).await? {
        info!("kratos Token optained!");
        return Ok(Some((token, id)));
    }
    Ok(None)
}

#[cfg(test)]
mod test_kratos {
    use httpmock::prelude::*;

    use super::*;

    #[tokio::test]
    async fn test_get_kratos_token() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/identities/lol")
                    .query_param("include_credential", "oidc");
                then.status(200).body(PAYLOAD_OIDC);
            })
            .await;
        let user_data = KratosData {
            provider_id: "github",
            user_id: 1,
        };
        let identity = serde_json::from_str::<Vec<Identity>>(PAYLOAD_IDENTITIES).unwrap();
        let res = get_kratos_token(&identity, &server.base_url(), user_data)
            .await
            .unwrap();
        mock.assert_async().await;
        let (token, id) = res.unwrap();
        assert_eq!(token, "gho_QRWWQERTQTR45655757745");
        assert_eq!(id, "lol");
    }

    #[tokio::test]
    async fn test_security_check() {
        let server = MockServer::start_async().await;
        let mock_token = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/identities/lol")
                    .query_param("include_credential", "oidc");
                then.status(200).body(PAYLOAD_OIDC);
            })
            .await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/identities")
                    .query_param("credentials.identifier", "github:1");
                then.status(200).body(PAYLOAD_IDENTITIES);
            })
            .await;
        let user_data = KratosData {
            provider_id: "github",
            user_id: 1,
        };
        let res = security_check(server.base_url(), user_data).await.unwrap();
        mock.assert_async().await;
        mock_token.assert_async().await;
        let (token, id) = res.unwrap();
        assert_eq!(token, "gho_QRWWQERTQTR45655757745");
        assert_eq!(id, "lol");
    }

    const PAYLOAD_OIDC: &str = r#"{
"id": "lol",
"credentials": {
  "oidc": {
    "type": "oidc",
    "identifiers": [
      "github:1"
    ],
    "config": {
      "providers": [
        {
          "initial_id_token": "",
          "subject": "1",
          "provider": "github",
          "initial_access_token": "gho_QRWWQERTQTR45655757745",
          "initial_refresh_token": ""
        }
      ]
    },
    "created_at": "2021-11-29T13:16:35.663741Z",
    "updated_at": "2021-11-29T13:16:35.663741Z"
  },
  "password": {
    "type": "password",
    "identifiers": [
      "lol.lol@lol.com"
    ],
    "created_at": "2021-11-29T13:16:35.655166Z",
    "updated_at": "2021-11-29T13:16:35.655166Z"
  }
},
"schema_id": "default",
"schema_url": "https://www2.dev.w6d.io/schemas/default",
"state": "active",
"state_changed_at": "2021-12-02T13:04:55.100327Z",
"traits": {
  "name": {},
  "email": "lol.lol@lol.com",
  "providers": [
    {
      "name": "github",
      "issuer_url": "https://github.com",
      "provider_type": "github"
    }
  ]
},
"verifiable_addresses": [
  {
    "id": "f23f32f-f23f32-f23f23f-f2f32-f23f23f2f23f",
    "value": "lol.lol@lol.com",
    "verified": false,
    "via": "email",
    "status": "sent",
    "created_at": "2021-12-02T13:04:55.106515Z",
    "updated_at": "2021-12-02T13:04:55.106515Z"
  }
],
"recovery_addresses": [
  {
    "id": "f23f23-f3223f-23f23f23-f23f32f-f32f232f2f3f",
    "value": "lol.lol@lol.com",
    "via": "email",
    "created_at": "2021-12-02T13:04:55.110475Z",
    "updated_at": "2021-12-02T13:04:55.110475Z"
  }
],
"created_at": "2021-12-02T13:04:55.103256Z",
"updated_at": "2021-12-02T13:04:55.103256Z"
}"#;

    const PAYLOAD_IDENTITIES: &str = r#"[{
"id": "lol",
"schema_id": "default",
"schema_url": "https://www2.dev.w6d.io/schemas/default",
"state": "active",
"state_changed_at": "2021-12-02T13:04:55.100327Z",
"traits": {
  "name": {},
  "email": "lol.lol@lol.com",
  "providers": [
    {
      "name": "github",
      "issuer_url": "https://github.com",
      "provider_type": "github"
    }
  ]
},
"verifiable_addresses": [
  {
    "id": "f23f32f-f23f32-f23f23f-f2f32-f23f23f2f23f",
    "value": "lol.lol@lol.com",
    "verified": false,
    "via": "email",
    "status": "sent",
    "created_at": "2021-12-02T13:04:55.106515Z",
    "updated_at": "2021-12-02T13:04:55.106515Z"
  }
],
"recovery_addresses": [
  {
    "id": "f23f23-f3223f-23f23f23-f23f32f-f32f232f2f3f",
    "value": "lol.lol@lol.com",
    "via": "email",
    "created_at": "2021-12-02T13:04:55.110475Z",
    "updated_at": "2021-12-02T13:04:55.110475Z"
  }
],
"created_at": "2021-12-02T13:04:55.103256Z",
"updated_at": "2021-12-02T13:04:55.103256Z"
}]"#;
}
