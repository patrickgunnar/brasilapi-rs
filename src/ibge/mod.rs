use crate::{error::Error, spec::BRASIL_API_URL};
use serde::{Deserialize, Serialize};
use std::ascii::AsciiExt;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Municipality {
    nome: String,
    codigo_ibge: String,
}

impl Municipality {
    pub fn get_name(&self) -> &str {
        &self.nome
    }

    pub fn get_ibge_code(&self) -> &str {
        &self.codigo_ibge
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct State {
    id: i32,
    sigla: String,
    nome: String,
    regiao: StateRegion,
}

impl State {
    pub fn get_sigla(&self) -> &str {
        &self.sigla
    }

    pub fn get_name(&self) -> &str {
        &self.nome
    }

    pub fn get_region(&self) -> &StateRegion {
        &self.regiao
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct StateRegion {
    id: i32,
    sigla: String,
    nome: String,
}

impl StateRegion {
    pub fn get_sigla(&self) -> &str {
        &self.sigla
    }

    pub fn get_name(&self) -> &str {
        &self.nome
    }
}

pub enum MunicipalitiesProvider {
    DadosAbertos,
    Gov,
    Wikipedia,
}

impl MunicipalitiesProvider {
    pub fn to_string(&self) -> &str {
        match self {
            MunicipalitiesProvider::DadosAbertos => "dados-abertos-br",
            MunicipalitiesProvider::Gov => "gov",
            MunicipalitiesProvider::Wikipedia => "wikipedia",
        }
    }
}

pub struct IbgeService {
    base_url: String,
}

impl IbgeService {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    async fn get_municipalities_request(
        &self,
        uf: &str,
        providers: Option<Vec<MunicipalitiesProvider>>,
    ) -> Result<reqwest::Response, Error> {
        let providers = match providers {
            Some(providers) => providers
                .iter()
                .map(|provider| provider.to_string())
                .collect::<Vec<&str>>()
                .join(","),
            None => "".to_string(),
        };

        let url = format!(
            "{}/api/ibge/municipios/v1/{}?providers={}",
            self.base_url, uf, providers
        );

        match reqwest::get(&url).await {
            Ok(response) => Error::from_response(response).await,
            Err(e) => Err(Error::from_error(e)),
        }
    }

    async fn get_all_states_request(&self) -> Result<reqwest::Response, Error> {
        let url = format!("{}/api/ibge/uf/v1", self.base_url);

        match reqwest::get(&url).await {
            Ok(response) => Error::from_response(response).await,
            Err(e) => Err(Error::from_error(e)),
        }
    }

    async fn get_state_request(&self, code: &str) -> Result<reqwest::Response, Error> {
        let url = format!("{}/api/ibge/uf/v1/{}", self.base_url, code);

        match reqwest::get(&url).await {
            Ok(response) => Error::from_response(response).await,
            Err(e) => Err(Error::from_error(e)),
        }
    }
}

/// #### `get_municipalities(uf: &str, providers: Option<Vec<MunicipalitiesProvider>>)`
/// Retorna uma lista de municípios de um estado.
///
/// ### Argumentos
/// * `uf:&str` => Sigla da unidade federativa, por exemplo SP, RJ, SC, etc.
/// * `providers:Option<Vec<MunicipalitiesProvider>>` => Provedores de dados para consulta.
///
/// ### Retorno
/// * `Result<Vec<Municipality>, Error>`
///
/// # Exemplo
/// ```
/// use brasilapi::ibge;
///
/// #[tokio::main]
/// async fn main() {
///    let municipalities = ibge::get_municipalities("SP", None).await.unwrap();
/// }
/// ```
pub async fn get_municipalities(
    uf: &str,
    providers: Option<Vec<MunicipalitiesProvider>>,
) -> Result<Vec<Municipality>, Error> {
    let ibge_service = IbgeService::new(BRASIL_API_URL);

    let response = ibge_service
        .get_municipalities_request(uf, providers)
        .await?;

    let body = response.text().await.unwrap();
    let municipalities: Vec<Municipality> = serde_json::from_str(&body).unwrap();

    Ok(municipalities)
}

pub async fn find_municipality_by_state_and_name(
    uf: &str,
    city_name: &str,
    providers: Option<Vec<MunicipalitiesProvider>>,
) -> Result<Option<Municipality>, Error> {
    let municipalities = get_municipalities(uf, providers).await?;

    let municipality = municipalities
        .into_iter()
        .find(|municipality| municipality.get_name().eq_ignore_ascii_case(city_name));

    Ok(municipality)
}

/// #### `get_all_states()`
/// Retorna informações de todos estados do Brasil
///
/// ### Retorno
/// * `Result<Vec<State>, Error>`
///
/// # Exemplo
/// ```
/// use brasilapi::ibge;
///
/// #[tokio::main]
/// async fn main() {
///    let states = ibge::get_all_states().await.unwrap();
/// }
/// ```
pub async fn get_all_states() -> Result<Vec<State>, Error> {
    let ibge_service = IbgeService::new(BRASIL_API_URL);

    let response = ibge_service.get_all_states_request().await?;

    let body = response.text().await.unwrap();
    let states: Vec<State> = serde_json::from_str(&body).unwrap();

    Ok(states)
}

/// #### `get_state(code: &str)`
/// Busca as informações de um estado a partir da sigla ou código
///
/// ### Argumento
/// * `code:&str` => Sigla ou código do estado
///
/// ### Retorno
/// * `Result<State, Error>`
///
/// # Exemplo
/// ```
/// use brasilapi::ibge;
///
/// #[tokio::main]
/// async fn main() {
///   let state = ibge::get_state("SP").await.unwrap();
/// }
/// ```
pub async fn get_state(code: &str) -> Result<State, Error> {
    let ibge_service = IbgeService::new(BRASIL_API_URL);

    let response = ibge_service.get_state_request(code).await?;

    let body = response.text().await.unwrap();
    let state: State = serde_json::from_str(&body).unwrap();

    Ok(state)
}

#[cfg(test)]
mod ibge_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_municipalities() {
        let municipalities = get_municipalities("SP", None).await.unwrap();

        assert_eq!(municipalities.len(), 645);
    }

    #[tokio::test]
    async fn test_get_municipalities_with_providers() {
        let providers = vec![
            MunicipalitiesProvider::DadosAbertos,
            MunicipalitiesProvider::Wikipedia,
        ];

        let municipalities = get_municipalities("SC", Some(providers)).await.unwrap();

        assert_eq!(municipalities.len(), 295);
    }

    #[tokio::test]
    async fn test_get_municipalities_with_invalid_uf() {
        let result = get_municipalities("XX", None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_states() {
        let states = get_all_states().await.unwrap();

        assert_eq!(states.len(), 27);
    }

    #[tokio::test]
    async fn test_get_state() {
        let state = get_state("SP").await.unwrap();

        assert_eq!(state.sigla, "SP");
    }

    #[tokio::test]
    async fn test_get_state_with_invalid_code() {
        let result = get_state("XX").await;

        assert!(result.is_err());
    }
}
