use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::BufReader;

use crate::jup::TokenSymbol;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Token {
    pub address: String,
    pub symbol: TokenSymbol,
    pub name: String,
    pub decimals: u8,
}

#[derive(Debug, Default, Clone)]
pub struct TokenRegistry {
    pub tokens: Vec<Token>,
    pub stable_tokens: Vec<Token>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        let file_path = "./tokens/default.json";
        let stable_file_path = "./tokens/stable.json";
        let tokens = Self::load_tokens(file_path).expect("Missing json");
        let stable_tokens = Self::load_stable_tokens(stable_file_path).expect("Missing json");

        TokenRegistry {
            tokens,
            stable_tokens,
        }
    }

    fn load_tokens(file_path: &str) -> anyhow::Result<Vec<Token>> {
        let file = File::open(file_path).context("Failed to open token file")?;
        let reader = BufReader::new(file);
        let tokens = serde_json::from_reader(reader)?;

        Ok(tokens)
    }

    fn load_stable_tokens(file_path: &str) -> anyhow::Result<Vec<Token>> {
        let file = File::open(file_path).context("Failed to open token file")?;
        let reader = BufReader::new(file);
        let stable_tokens = serde_json::from_reader(reader)?;

        Ok(stable_tokens)
    }

    pub fn get_by_address(&self, address: &str) -> Option<&Token> {
        self.tokens.iter().find(|token| token.address == address)
    }

    pub fn get_by_symbol(&self, symbol: &TokenSymbol) -> Option<&Token> {
        self.tokens.iter().find(|token| token.symbol == *symbol)
    }

    pub fn get_by_pair_address(&self, address: &str) -> anyhow::Result<Vec<Token>> {
        if !address.contains("_") {
            bail!("Not pair address")
        }

        let pairs = address.split("_").collect::<Vec<_>>();
        let tokens = vec![
            self.get_by_address(pairs[0]).expect("Not exist").clone(),
            self.get_by_address(pairs[1]).expect("Not exist").clone(),
        ];

        Ok(tokens)
    }

    pub fn get_tokens_from_pair_address(&self, address: &str) -> anyhow::Result<Vec<Token>> {
        let tokens = if address.contains("_") {
            self.get_by_pair_address(address).expect("Invalid address")
        } else if let Some(token) = self.get_by_address(address) {
            vec![token.clone()]
        } else {
            vec![]
        };

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_registry_load_and_parse() {
        // Make test return Result
        let token_registry = TokenRegistry::new();
        let sol_token = token_registry
            .get_by_address("So11111111111111111111111111111111111111112")
            .unwrap();
        let jlp_token = token_registry.get_by_symbol(&TokenSymbol::JLP).unwrap();

        assert_eq!(sol_token.symbol, TokenSymbol::SOL);
        assert_eq!(jlp_token.symbol, TokenSymbol::JLP);
    }

    #[test]
    fn test_tokens() {
        let registry = TokenRegistry::new();
        assert!(!registry.tokens.is_empty());
        assert!(!registry.stable_tokens.is_empty());
    }
}
