use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use crate::jup::TokenSymbol;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Token {
    pub address: String,
    pub symbol: TokenSymbol,
    pub name: String,
    pub decimals: u8,
}

#[derive(Debug, Default, Clone)]
pub struct TokenRegistry {
    by_address: HashMap<String, Token>,
    by_symbol: HashMap<TokenSymbol, Token>,
}

impl TokenRegistry {
    pub fn load(file_path: &str) -> Result<Vec<Token>> {
        let file = File::open(file_path).context("Failed to open token file")?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).context("Invalid token JSON format")
    }

    pub fn parse(raw_tokens: Vec<Token>) -> Result<Self> {
        let mut registry = TokenRegistry::default();

        for raw in raw_tokens {
            let symbol = raw.symbol;

            let token = Token {
                address: raw.address.clone(),
                symbol,
                name: raw.name,
                decimals: raw.decimals,
            };

            registry
                .by_address
                .insert(raw.address.clone(), token.clone());
            registry.by_symbol.insert(symbol, token);
        }

        Ok(registry)
    }

    pub fn get_by_address(&self, address: &str) -> Option<&Token> {
        self.by_address.get(address)
    }

    pub fn get_by_symbol(&self, symbol: &TokenSymbol) -> Option<&Token> {
        self.by_symbol.get(symbol)
    }

    pub fn symbol_map(&self) -> HashMap<TokenSymbol, String> {
        self.by_symbol
            .iter()
            .map(|(sym, token)| (*sym, token.address.clone()))
            .collect()
    }

    pub fn get_by_token_symbol(&self, token_symbol: &TokenSymbol) -> Option<&Token> {
        let binding = self.symbol_map();
        let address = binding
            .get(token_symbol)
            .unwrap_or_else(|| panic!("Not found {:#?}", token_symbol));
        self.get_by_address(address)
    }

    pub fn all_tokens(&self) -> Vec<&Token> {
        self.by_address.values().collect()
    }

    pub fn get_tokens() -> Vec<Token> {
        let file_path = "./tokens/default.json";
        let json_value = TokenRegistry::load(file_path).unwrap();
        let token_registry = TokenRegistry::parse(json_value).unwrap();
        token_registry.by_address.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token() {
        let token = Token {
            address: "So111...11112".into(),
            symbol: TokenSymbol::SOL,
            name: "Wrapped SOL".into(),
            decimals: 9,
        };

        println!("{token:#?}");
    }

    #[test]
    fn test_token_registry_load_and_parse() {
        let file_path = "./tokens/default.json";
        let json_value = TokenRegistry::load(file_path).unwrap();
        let token_registry = TokenRegistry::parse(json_value).unwrap();
        let sol_token =
            token_registry.get_by_address("So11111111111111111111111111111111111111112");
        let jup_token = token_registry.get_by_token_symbol(&TokenSymbol::JLP);

        println!("{:#?}", sol_token);
        println!("{:#?}", jup_token);
    }

    #[test]
    fn test_get_tokens() {
        let tokens = TokenRegistry::get_tokens();

        println!("{:#?}", tokens);
    }
}
