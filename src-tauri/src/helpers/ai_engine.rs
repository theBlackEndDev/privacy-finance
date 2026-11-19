//! In-process LLM inference via llama.cpp (CPU only).
//!
//! Sampling rules follow Locked decision D9: the model's own chat template,
//! greedy sampling, at most 256 new tokens, stop at end-of-turn or `;`.

use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

use crate::error::{AppError, AppResult};

const MAX_NEW_TOKENS: usize = 256;
const QWEN3_EMPTY_THINK: &str = "<think>\n\n</think>\n\n";

/// llama.cpp allows one backend init per process; share it between engines.
fn backend() -> AppResult<&'static LlamaBackend> {
    static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();
    static INIT: Mutex<()> = Mutex::new(());
    let _guard = INIT.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(b) = BACKEND.get() {
        return Ok(b);
    }
    let mut b = LlamaBackend::init().map_err(err("failed to init llama backend"))?;
    b.void_logs();
    Ok(BACKEND.get_or_init(|| b))
}

fn err<E: std::fmt::Display>(what: &'static str) -> impl Fn(E) -> AppError {
    move |e| AppError::Validation(format!("{what}: {e}"))
}

pub struct Engine {
    backend: &'static LlamaBackend,
    model: LlamaModel,
    is_qwen3: bool,
}

impl Engine {
    pub fn load(path: &Path) -> AppResult<Engine> {
        if !path.exists() {
            return Err(AppError::NotFound(format!("model file {}", path.display())));
        }
        let backend = backend()?;
        // CPU only (D7): llama-cpp-sys-2 builds Metal on macOS, so keep every layer off the GPU.
        let params = LlamaModelParams::default().with_n_gpu_layers(0);
        let model = LlamaModel::load_from_file(backend, path, &params)
            .map_err(err("failed to load model"))?;
        let is_qwen3 = model
            .meta_val_str("general.architecture")
            .map(|a| a.starts_with("qwen3"))
            .unwrap_or(false);
        Ok(Engine { backend, model, is_qwen3 })
    }

    /// Run one system + user turn and return the generated text, cut at the
    /// first `;` (exclusive) and trimmed.
    pub fn complete(&self, system: &str, user: &str) -> AppResult<String> {
        let template = self
            .model
            .chat_template(None)
            .map_err(err("model has no chat template"))?;
        let msgs = vec![
            LlamaChatMessage::new("system".into(), system.into()).map_err(err("bad system prompt"))?,
            LlamaChatMessage::new("user".into(), user.into()).map_err(err("bad user prompt"))?,
        ];
        let mut prompt = self
            .model
            .apply_chat_template(&template, &msgs, true)
            .map_err(err("failed to apply chat template"))?;
        if self.is_qwen3 {
            prompt.push_str(QWEN3_EMPTY_THINK);
        }

        // The template already contains special tokens such as BOS, so do not add another.
        let vocab = self.model.vocab();
        let tokens = vocab.tokenize(prompt.as_bytes(), true, true);
        let n_prompt = tokens.len();
        let n_ctx = (n_prompt + MAX_NEW_TOKENS).next_multiple_of(256) as u32;
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(n_ctx))
            .with_n_batch(n_ctx)
            // CPU only: also keep KV cache and ops off the GPU.
            .with_offload_kqv(false)
            .with_op_offload(false);
        let mut ctx = self
            .model
            .new_context(self.backend, ctx_params)
            .map_err(err("failed to create context"))?;

        let mut batch = LlamaBatch::new(n_prompt.max(1), 1);
        let last = n_prompt as i32 - 1;
        for (i, tok) in tokens.iter().enumerate() {
            batch.add(*tok, i as i32, &[0], i as i32 == last).map_err(err("batch error"))?;
        }
        ctx.decode(&mut batch).map_err(err("decode failed"))?;

        let mut sampler = LlamaSampler::greedy();
        let mut out: Vec<u8> = Vec::new();
        let mut pos = n_prompt as i32;
        for _ in 0..MAX_NEW_TOKENS {
            let tok = sampler.sample(&ctx, batch.n_tokens() - 1);
            if vocab.is_eog(tok) {
                break;
            }
            out.extend(vocab.token_to_piece(tok, false, None));
            if out.contains(&b';') {
                break;
            }
            batch.clear();
            batch.add(tok, pos, &[0], true).map_err(err("batch error"))?;
            pos += 1;
            ctx.decode(&mut batch).map_err(err("decode failed"))?;
        }
        if let Some(i) = out.iter().position(|&b| b == b';') {
            out.truncate(i);
        }
        Ok(String::from_utf8_lossy(&out).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn engine_completes() {
        let path = std::env::var("MODEL_PATH").expect("set MODEL_PATH to a .gguf file");
        let engine = Engine::load(Path::new(&path)).unwrap();
        let out = engine
            .complete(
                "Answer with one SQLite SELECT statement and nothing else.",
                "How many rows are in the table transactions?",
            )
            .unwrap();
        println!("model output: {out}");
        assert!(!out.is_empty());
    }
}
