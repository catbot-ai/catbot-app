use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use crate::jup::TokenSymbol;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Token {
    pub address: String,
    pub symbol: TokenSymbol,
    pub name: String,
    pub decimals: u8,
}

#[derive(Debug, Default, Clone)]
pub struct TokenRegistry {
    pub by_address: HashMap<String, Token>,
    pub by_symbol: HashMap<TokenSymbol, Token>,
    pub tokens: Vec<Token>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        let file_path = "./tokens/default.json";
        let raw_tokens = Self::load_tokens(file_path).expect("Missing json");

        Self::parse(raw_tokens).expect("Invalid json")
    }

    fn load_tokens(file_path: &str) -> anyhow::Result<Vec<Token>> {
        let file = File::open(file_path).context("Failed to open token file")?;
        let reader = BufReader::new(file);
        let tokens = serde_json::from_reader(reader)?;

        Ok(tokens)
    }

    pub fn parse(tokens: Vec<Token>) -> anyhow::Result<Self> {
        let mut registry = TokenRegistry {
            tokens,
            by_address: HashMap::new(),
            by_symbol: HashMap::new(),
        };

        for token in &registry.tokens {
            let symbol = token.symbol;

            registry
                .by_address
                .insert(token.address.clone(), token.clone());
            registry.by_symbol.insert(symbol, token.clone());
        }

        Ok(registry)
    }

    pub fn get_by_address(&self, address: &str) -> Option<&Token> {
        self.by_address.get(address)
    }

    pub fn get_by_symbol(&self, symbol: &TokenSymbol) -> Option<&Token> {
        self.by_symbol.get(symbol)
    }

    pub fn get_tokens() -> Vec<Token> {
        TokenRegistry::new().tokens
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
    fn test_token_registry_load_and_parse() -> Result<()> {
        // Make test return Result
        let token_registry = TokenRegistry::new();
        let sol_token = token_registry
            .get_by_address("So11111111111111111111111111111111111111112")
            .unwrap();
        let jlp_token = token_registry.get_by_symbol(&TokenSymbol::JLP).unwrap();

        assert_eq!(sol_token.symbol, TokenSymbol::SOL);
        assert_eq!(jlp_token.symbol, TokenSymbol::JLP);
        Ok(())
    }

    #[test]
    fn test_get_tokens() -> Result<()> {
        let tokens = TokenRegistry::get_tokens();
        println!("{:#?}", tokens);
        assert!(!tokens.is_empty());
        Ok(())
    }

    #[test]
    fn test_tokens() -> Result<()> {
        let registry = TokenRegistry::new();
        let tokens = registry.tokens;
        assert!(!tokens.is_empty());
        Ok(())
    }
}
