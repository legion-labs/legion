use std::collections::HashMap;

use anyhow::{bail, Context};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::authentication::jwt::signature_validation::RsaSignatureValidation;

use super::{
    SignatureValidation,
    ValidationResult::{self, Unsupported},
};

pub struct AwsCognitoSignatureValidation {
    keys: HashMap<String, Box<dyn SignatureValidation>>,
}

impl AwsCognitoSignatureValidation {
    /// Create a new AWS cognito signature validation by fetching the signing key from the given
    /// region and user pool id.
    pub async fn new(region: &str, aws_cognito_user_pool_id: &str) -> anyhow::Result<Self> {
        let url = format!(
            "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
            region, aws_cognito_user_pool_id,
        );

        debug!("Loading JWKS from {}...", url);

        let resp = reqwest::get(url).await.context("failed to fetch JWKS")?;
        let data = resp.text().await.context("failed to read response body")?;

        Self::new_from_jwks(&data)
    }

    fn new_from_jwks(data: &str) -> anyhow::Result<Self> {
        let jwks: Jwks = serde_json::from_str(data).context("failed to parse JWKS")?;
        let keys = jwks
            .keys
            .into_iter()
            .filter_map(|jwk| match jwk.to_rsa_signature_validation() {
                Ok(rsa_signature_validation) => Some((jwk.kid.clone(), rsa_signature_validation)),
                _ => None,
            })
            .collect();

        Self::new_from_keys(keys)
    }

    fn new_from_keys(keys: HashMap<String, Box<dyn SignatureValidation>>) -> anyhow::Result<Self> {
        if keys.is_empty() {
            bail!("no valid keys found in JWKS");
        }

        Ok(Self { keys })
    }
}

impl SignatureValidation for AwsCognitoSignatureValidation {
    fn validate_signature<'a>(
        &self,
        alg: &'a str,
        kid: Option<&'a str>,
        message: &'a str,
        signature: &'a [u8],
    ) -> ValidationResult<'a> {
        match kid {
            Some(kid) => self.keys.get(kid).map_or_else(
                || Unsupported(alg, Some(kid)),
                |key| key.validate_signature(alg, Some(kid), message, signature),
            ),
            None => Unsupported(alg, kid),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Jwk {
    kid: String,
    alg: String,
    kty: String,
    e: Option<String>,
    n: Option<String>,
    #[serde(rename = "use")]
    use_: String,
}

impl Jwk {
    fn to_rsa_signature_validation(&self) -> anyhow::Result<Box<dyn SignatureValidation>> {
        match self.kty.as_str() {
            "RSA" => match &self.n {
                None => {
                    bail!(
                        "Ignoring key {} as it does not contain the expected `n` value",
                        self.kid
                    );
                }
                Some(n) => match &self.e {
                    None => {
                        bail!(
                            "Ignoring key {} as it does not contain the expected `e` value",
                            self.kid
                        );
                    }
                    Some(e) => {
                        let rsa_signature_validation =
                            RsaSignatureValidation::new_from_components(n, e)?;

                        Ok(Box::new(rsa_signature_validation))
                    }
                },
            },
            _ => bail!("Unsupported key type {}", self.kty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_jwks() {
        let jwks = r#"{"keys":[{"alg":"RS256","e":"AQAB","kid":"31nW+dajw5ar3EySfbPel0xy2Yzw8sLiWiveaVnV39E=","kty":"RSA","n":"sDZrbHaGqPRpIawaLDA9cVySNL8QyAVWcCyLSYENBSJuvjk6ELyCbT3IYGbQv_lraZ3mYGeRp7JWVLMu_-ff3LMsjOcY4aViPIBhgilK5PGt2fOX7uInbleIrKOV21MLrdOq2Z_jh5TctjktwgxV1SN95r3CUe6U1lrL2SEOcN580Accdrl6yHmfq8Mrvv-2p0TG6eQaEbqoZBRr7_f297r-i-I-N8iUIxphdpB2DQGALnWOT_SFiIf-b4UTRyBCKJlUFBAUixS0wHm8HMpCiBZK_lGfZ94Y75mY1BKdqQ_d1lMc634McGFpX52iyZFkIca8tB4bhNBAgbIfTQyRJw","use":"sig"},{"alg":"RS256","e":"AQAB","kid":"zknIUtzDKlWQuUk9O0y9qNoopBlErqu8Sq9N8nIp0/o=","kty":"RSA","n":"xPpUroo8ljcBQW74xc3UmCDxsyecVl8VKrudUrp_VEtWxfseDR0d9rZLkaT8PDF27qNuxbgCAAA9XBhUsvyMrnWoJAiYI_HIvHe40IUfCLdq6mytRyOsgZy3Yxorp_E2h7zXroU0VqbFx1QXrf0vqruejxgpAyNDEpr1FRqM08c6JElMrEjGvjXsSsjEJ9awVIOwcGgTLbbLXAoHcXpz3_ekOObYA7yFrxxsBAB9jOOSs56YfTGJvjG6_v0rEp5R0QVOwV2SH96Zypji2BrFnE-lGejcAom5wMWdEnhLwjj0zl6ffL_nwLcwfVA-Cfi1LqBRwsjNwEJum6opeAjuDQ","use":"sig"}]}"#;

        let validation = AwsCognitoSignatureValidation::new_from_jwks(jwks).unwrap();
        assert_eq!(validation.keys.len(), 2);
    }
}
