pub mod gemini;
pub mod explainer;

pub use gemini::GeminiBackend;
pub use explainer::CommandExplainer;

use anyhow::Result;
use async_trait::async_trait;
use crate::config::Config;
use crate::kubectl::{KubectlContext, TranslationResult};
use crate::tools::{LLMBackend, LLMResponse};
use std::path::Path;

/// Real AI-powered Natural Language to Command Translator
/// Supports both local GGUF models (primary) and Gemini API (fallback)
pub struct AIManager {
    gemini: GeminiBackend,
}

impl AIManager {
    /// Create a new AI manager
    pub fn new(_config: Config) -> Self {
        Self {
            gemini: GeminiBackend::new(),
        }
    }
    


    /// Translate natural language to kubectl command
    /// Primary: Gemini API, Fallback: local GGUF model
    /// Returns TranslationResult with kubectl command, confidence score, and reasoning
    pub async fn translate_kubectl(&self, input: &str, context: &KubectlContext) -> crate::utils::KaidoResult<TranslationResult> {
        log::info!("Attempting kubectl translation with Gemini AI");
        
        // Build kubectl-specific prompt
        let namespace = context.namespace.as_deref().unwrap_or("default");
        let prompt = format!(
            "Translate this natural language request into a kubectl command.\n\
            Current Kubernetes context:\n\
            - Cluster: {}\n\
            - Namespace: {}\n\
            - Environment: {}\n\n\
            User request: {}\n\n\
            Respond ONLY with a JSON object in this exact format:\n\
            {{\n  \"command\": \"kubectl ...\",\n  \"confidence\": 85,\n  \"reasoning\": \"explanation\"\n}}",
            context.cluster,
            namespace,
            context.environment_type.as_str(),
            input
        );
        
        // Try Gemini API first
        let response_text = match self.gemini.infer(&prompt).await {
            Ok(response) => {
                log::info!("[OK] Gemini AI translation successful");
                // Gemini returns LLMResponse, extract the reasoning (which contains the JSON)
                response.reasoning
            }
            Err(e) => {
                log::warn!("Gemini API failed: {}, falling back to local GGUF model", e);
                // Fallback to local model
                self.interpret_with_local_model(&prompt).await?
            }
        };
        
        // Parse JSON response
        #[derive(serde::Deserialize)]
        struct KubectlResponse {
            command: String,
            confidence: u8,
            reasoning: String,
        }
        
        match serde_json::from_str::<KubectlResponse>(&response_text) {
            Ok(parsed) => {
                log::info!("Local GGUF model translation successful: {}", parsed.command);
                Ok(TranslationResult {
                    kubectl_command: parsed.command,
                    confidence_score: parsed.confidence,
                    reasoning: parsed.reasoning,
                })
            }
            Err(e) => {
                log::warn!("Failed to parse GGUF model output as JSON: {e}");
                Err(crate::utils::KaidoError::ModelError {
                    message: format!("Local model returned invalid JSON: {e}"),
                    model_name: "local-gguf".to_string(),
                })
            }
        }
    }


    /// Interpret natural language input using local GGUF model with llama-cpp-rs
    async fn interpret_with_local_model(&self, input: &str) -> crate::utils::KaidoResult<String> {
        let model_path = self.find_gguf_model()?;
        
        log::info!("Using GGUF model: {}", model_path.display());

        // Use llama-cpp-rs for real inference
        self.run_llama_cpp_inference(&model_path, input).await
    }

    /// Run actual GGUF inference using llama-cpp-2
    async fn run_llama_cpp_inference(&self, model_path: &Path, user_input: &str) -> crate::utils::KaidoResult<String> {
        log::info!("=== Starting LLM Inference ===");
        log::info!("Model path: {}", model_path.display());
        log::info!("User input: {user_input}");
        
        use llama_cpp_2::{
            context::params::LlamaContextParams,
            llama_backend::LlamaBackend,
            llama_batch::LlamaBatch,
            model::{LlamaModel, params::LlamaModelParams, AddBos, Special},
            sampling::LlamaSampler,
        };
        use std::num::NonZeroU32;
        use std::fs::OpenOptions;
        use std::os::unix::io::AsRawFd;
        
        log::debug!("Redirecting llama.cpp stderr to /dev/null...");
        // CRITICAL: Silence llama.cpp verbose logging by redirecting stderr to /dev/null
        let stderr_redirect = {
            let devnull = OpenOptions::new().write(true).open("/dev/null")
                .map_err(|e| {
                    log::error!("Failed to open /dev/null: {e}");
                    crate::utils::KaidoError::ApplicationError {
                        message: format!("Failed to open /dev/null: {e}"),
                        context: None,
                    }
                })?;
            let stderr_fd = std::io::stderr().as_raw_fd();
            unsafe {
                libc::dup2(devnull.as_raw_fd(), stderr_fd);
            }
            devnull  // RAII: File handle auto-closes on drop, restoring stderr
        };
        // Keep RAII guard alive throughout inference
        let _ = &stderr_redirect;
        
        log::info!("Initializing llama-cpp backend");
        
        // Initialize backend
        let backend = LlamaBackend::init().map_err(|e| crate::utils::KaidoError::ModelError {
            message: format!("Backend init failed: {e}"),
            model_name: "llama-cpp".to_string(),
        })?;
        
        log::info!("Loading model: {}", model_path.display());
        
        // Load model
        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
            .map_err(|e| {
                log::error!("Failed to load model: {e}");
                crate::utils::KaidoError::ModelError {
                    message: format!("Failed to load model: {e}"),
                    model_name: "llama-cpp".to_string(),
                }
            })?;
        
        log::info!("Model loaded successfully, size: {} bytes", std::fs::metadata(model_path).map(|m| m.len()).unwrap_or(0));
        
        // Detect OS for appropriate commands
        let (os_type, shell_type) = if cfg!(target_os = "windows") {
            ("Windows", "PowerShell")
        } else if cfg!(target_os = "macos") {
            ("macOS", "bash/zsh")
        } else {
            ("Linux", "bash")
        };
        
        // Get current directory for context
        let current_dir = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "/unknown".to_string());

        // Enhanced few-shot prompt with current directory context
        let system_prompt = format!(r#"You are an expert {os_type} shell command generator. Your ONLY job is to output EXECUTABLE shell commands.

CURRENT CONTEXT:
- Working Directory: {current_dir}
- Shell: {shell_type}

STRICT OUTPUT FORMAT:
{{"task":"brief task description","commands":[{{"cmd":"actual shell command","description":"what this command does"}}]}}

CRITICAL RULES:
1. Return ONLY valid JSON (no markdown, no code blocks, no explanation)
2. Use REAL executable {shell_type} commands (no placeholders like <file>, {{path}}, or TODO)
3. Commands must run immediately without modification in the current directory
4. For multi-step tasks, split into separate command objects
5. Each cmd must be a single executable command line
6. When uncertain about paths, use pwd to understand context first

COMMAND COMPOSITION RULES:
- Always use REAL commands that exist in {shell_type} shell
- For file operations: use actual file names or wildcards, not placeholders
- For directory navigation: be aware of current dir: {current_dir}
- For complex tasks: chain commands with && or ; appropriately
- Error handling: pipe stderr to /dev/null ONLY when appropriate (e.g. 2>/dev/null)

EXAMPLES WITH CONTEXT AWARENESS:

User: 列出檔案
{{"task":"list files","commands":[{{"cmd":"ls -la","description":"list all files with details"}}]}}

User: 查詢當前目錄
{{"task":"show current directory","commands":[{{"cmd":"pwd","description":"print working directory"}}]}}

User: 找出最大的 10 個檔案
{{"task":"find largest files","commands":[{{"cmd":"du -sh * 2>/dev/null | sort -h | tail -10","description":"find and sort 10 largest files"}}]}}

User: create test file with hello content
{{"task":"create test file","commands":[{{"cmd":"echo 'hello' > test.txt","description":"create test.txt with hello content"}}]}}

User: check disk usage
{{"task":"check disk usage","commands":[{{"cmd":"df -h","description":"show disk space usage in human-readable format"}}]}}

User: find python files
{{"task":"find python files","commands":[{{"cmd":"find . -name '*.py' -type f","description":"recursively find all .py files"}}]}}

User: show system info and location
{{"task":"show system info","commands":[{{"cmd":"pwd","description":"show current directory"}},{{"cmd":"uname -a","description":"display system information"}},{{"cmd":"uptime","description":"show system uptime"}}]}}

User: 回到上一層並顯示內容
{{"task":"go up and list","commands":[{{"cmd":"cd .. && pwd","description":"go to parent directory"}},{{"cmd":"ls -la","description":"list parent directory contents"}}]}}

CRITICAL: Your commands will be executed in directory: {current_dir}
Generate commands accordingly. Return ONLY the JSON object."#
        );
        
        let prompt = format!("{system_prompt}\n\nUser: {user_input}\n");
        
        log::info!("Tokenizing prompt");
        
        // Tokenize
        let tokens = model.str_to_token(&prompt, AddBos::Always)
            .map_err(|e| crate::utils::KaidoError::ModelError {
                message: format!("Tokenization failed: {e}"),
                model_name: "llama-cpp".to_string(),
            })?;
        
        log::info!("Prompt tokenized to {} tokens", tokens.len());
        
        // Create context with sufficient size for prompts
        log::debug!("Creating context with n_ctx=2048");
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(2048));
        let mut ctx = model.new_context(&backend, ctx_params)
            .map_err(|e| {
                log::error!("Context creation failed: {e}");
                crate::utils::KaidoError::ModelError {
                    message: format!("Context creation failed: {e}"),
                    model_name: "llama-cpp".to_string(),
                }
            })?;
        log::debug!("Context created successfully");
        
        // Create sampler for token generation
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::dist(1234),
            LlamaSampler::greedy(),
        ]);
        
        // Autoregressive generation: generate multiple tokens until complete JSON
        let mut generated_tokens = Vec::new();
        let mut generated_text = String::new();
        let max_new_tokens = 100;
        
        log::info!("Starting token generation (max {max_new_tokens} tokens)...");
        
        // First pass: process prompt
        log::debug!("Creating batch for {} prompt tokens", tokens.len());
        let mut batch = LlamaBatch::new(2048, 1);  // Increased from 512 to 2048 to accommodate longer prompts
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], i == tokens.len() - 1)
                .map_err(|e| {
                    log::error!("Batch add failed at token {i}: {e}");
                    crate::utils::KaidoError::ModelError {
                        message: format!("Batch add failed: {e}"),
                        model_name: "llama-cpp".to_string(),
                    }
                })?;
        }
        
        log::debug!("Decoding prompt batch...");
        ctx.decode(&mut batch).map_err(|e| {
            log::error!("Decode failed: {e}");
            crate::utils::KaidoError::ModelError {
                message: format!("Decode failed: {e}"),
                model_name: "llama-cpp".to_string(),
            }
        })?;
        log::debug!("Prompt batch decoded successfully");
        
        // Generate tokens autoregressively
        let mut n_cur = tokens.len() as i32;
        let eos_token = llama_cpp_2::token::LlamaToken(2); // Standard EOS token ID
        
        log::debug!("Starting autoregressive token generation loop...");
        for i in 0..max_new_tokens {
            // Sample next token
            let new_token = sampler.sample(&ctx, -1);
            
            if i % 10 == 0 {
                log::debug!("Generated {} tokens so far, current text length: {}", i, generated_text.len());
            }
            
            generated_tokens.push(new_token);
            
            // Decode token to string - skip invalid UTF-8 tokens
            match model.token_to_str(new_token, Special::Tokenize) {
                Ok(token_str) => {
                    generated_text.push_str(&token_str);
                }
                Err(e) => {
                    log::warn!("Skipping invalid token at iteration {i}: {e}");
                    continue;
                }
            }
            
            // Check for EOS or complete JSON
            if new_token == eos_token {
                break;
            }
            
            // Check if we have a complete JSON (must end with }]} for our schema)
            let trimmed = generated_text.trim();
            if trimmed.ends_with("}]}") && trimmed.starts_with('{') {
                break;
            }
            
            // Safety: stop if output is too long
            if generated_text.len() > 1000 {
                log::warn!("Output too long, stopping");
                break;
            }
            
            // Prepare next batch with the new token
            batch.clear();
            batch.add(new_token, n_cur, &[0], true)
                .map_err(|e| crate::utils::KaidoError::ModelError {
                    message: format!("Batch add failed: {e}"),
                    model_name: "llama-cpp".to_string(),
                })?;
            
            // Decode
            ctx.decode(&mut batch).map_err(|e| crate::utils::KaidoError::ModelError {
                message: format!("Decode failed: {e}"),
                model_name: "llama-cpp".to_string(),
            })?;
            
            n_cur += 1;
        }
        
        let output = generated_text.trim().to_string();
        
        log::info!("Generated {} tokens: {}", generated_tokens.len(), output);
        
        Ok(output)
    }

    /// Find available GGUF model (REMOVED: Local model support removed in kubectl-only MVP)
    fn find_gguf_model(&self) -> crate::utils::KaidoResult<std::path::PathBuf> {
        // Priority: local GGUF models for privacy (enterprise requirement)
        // Fallback to OpenAI only when local model unavailable
        let model_dir = "models";
        
        if !Path::new(model_dir).exists() {
            log::warn!("Model directory '{model_dir}' not found, will fallback to OpenAI API");
            return Err(crate::utils::KaidoError::ModelError {
                message: format!("Model directory '{model_dir}' not found. Create directory and place GGUF model, or configure OpenAI API as fallback."),
                model_name: "local".to_string(),
            });
        }

        let entries = std::fs::read_dir(model_dir)
            .map_err(|e| crate::utils::KaidoError::ModelError {
                message: format!("Cannot read model directory: {e}"),
                model_name: "local".to_string(),
            })?;

        let mut gguf_files = Vec::new();
        
        for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "gguf" {
                            gguf_files.push(path);
                    }
                }
            }
        }
        
        if gguf_files.is_empty() {
            log::warn!("No GGUF model files found in '{model_dir}', will fallback to OpenAI API");
            return Err(crate::utils::KaidoError::ModelError {
                message: format!("No GGUF model found in '{model_dir}'. Download a model or configure OpenAI API as fallback."),
                model_name: "local".to_string(),
            });
        }
        
        // Prioritize llama models for kubectl translation
        for path in &gguf_files {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.to_lowercase().contains("llama") {
                        log::info!("Found Llama GGUF model: {}", path.display());
                        return Ok(path.clone());
                    }
                }
            }
        }
        
        // Fall back to first available GGUF model
        let path = &gguf_files[0];
        log::info!("Using GGUF model: {}", path.display());
        Ok(path.clone())
    }
}

// Implement LLMBackend trait for AIManager
#[async_trait]
impl LLMBackend for AIManager {
    async fn infer(&self, prompt: &str) -> Result<LLMResponse> {
        // Try Gemini API first (faster, more capable for explanations)
        log::info!("AIManager: Attempting Gemini API inference");

        match self.gemini.infer(prompt).await {
            Ok(response) => {
                log::info!("[OK] Gemini API inference successful");
                Ok(response)
            }
            Err(e) => {
                log::warn!("Gemini API failed: {}, falling back to local GGUF model", e);

                // Fallback to local GGUF model
                match self.interpret_with_local_model(prompt).await {
                    Ok(output) => {
                        log::info!("[OK] Local GGUF model inference successful");
                        let command = output.lines().next().unwrap_or("").to_string();
                        Ok(LLMResponse {
                            command: command.clone(),
                            confidence: 75,
                            reasoning: output,
                        })
                    }
                    Err(local_e) => {
                        log::error!("Both Gemini and local model failed");
                        Err(anyhow::anyhow!("All AI backends failed: Gemini: {}, Local: {}", e, local_e))
                    }
                }
            }
        }
    }
}
