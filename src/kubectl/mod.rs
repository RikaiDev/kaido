// Kubectl module for natural language to kubectl command translation
//
// This module provides:
// - context.rs: Kubeconfig parsing and environment detection
// - translator.rs: Natural language to kubectl via OpenAI
// - risk_classifier.rs: Risk level classification (LOW/MEDIUM/HIGH)
// - executor.rs: kubectl command execution

pub mod context;
pub mod translator;
pub mod risk_classifier;
pub mod executor;
pub mod openai;

pub use context::{KubectlContext, EnvironmentType};
pub use translator::TranslationResult;
pub use risk_classifier::RiskLevel;
pub use executor::{ExecutionResult, execute_kubectl, format_output};

