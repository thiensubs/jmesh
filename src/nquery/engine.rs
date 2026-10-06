use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use tokenizers::Tokenizer;
use tract_onnx::prelude::*;

pub struct OnnxEngine;

/// Encoder file names, in preference order (smallest/fastest first).
const ENCODER_CANDIDATES: &[&str] = &[
    "encoder_model_q4.onnx",
    "encoder_model_quantized.onnx",
    "encoder_model_fp16.onnx",
    "encoder_model.onnx",
];

/// Decoder file names, in preference order. "Merged" and "with_past" decoder
/// variants need past-key-value plumbing that the greedy loop below does not
/// implement, so only plain decoders are accepted.
const DECODER_CANDIDATES: &[&str] = &[
    "decoder_model_q4.onnx",
    "decoder_model_quantized.onnx",
    "decoder_model_fp16.onnx",
    "decoder_model.onnx",
];

const MAX_GENERATED_TOKENS: usize = 128;

impl OnnxEngine {
    pub fn generate(model_dir: &Path, prompt: &str) -> Result<String> {
        let tokenizer = Tokenizer::from_file(model_dir.join("tokenizer.json"))
            .map_err(|e| anyhow::anyhow!("tokenizer load failed: {}", e))?;

        let (eos_id, dec_start_id) = generation_config(model_dir);

        // 1. Tokenize
        let encoding = tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow::anyhow!("tokenization failed: {}", e))?;
        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();
        let input_len = input_ids.len();

        // 2. Load and run encoder
        let encoder_path = find_model_file(model_dir, ENCODER_CANDIDATES, "encoder")?;
        let encoder = load_runnable(&encoder_path)?;

        let input_tensor = Tensor::from_shape(&[1, input_len], &input_ids)?;
        let mask_tensor = Tensor::from_shape(&[1, input_len], &attention_mask)?;

        let encoder_result = encoder.run(named_inputs(
            &encoder,
            [
                ("input_ids", input_tensor.into()),
                ("attention_mask", mask_tensor.into()),
            ],
        )?)?;
        let encoder_hidden = encoder_result[0].clone();
        let encoder_seq_len = encoder_hidden.shape()[1];

        // 3. Greedy decode
        let decoder_path = find_model_file(model_dir, DECODER_CANDIDATES, "decoder")?;
        let decoder = load_runnable(&decoder_path)?;

        let mut decoder_ids: Vec<i64> = vec![dec_start_id as i64];

        for _ in 0..MAX_GENERATED_TOKENS {
            let dec_tensor = Tensor::from_shape(&[1, decoder_ids.len()], &decoder_ids)?;
            let enc_mask_tensor =
                Tensor::from_shape(&[1, encoder_seq_len], &vec![1i64; encoder_seq_len])?;

            let dec_result = decoder.run(named_inputs(
                &decoder,
                [
                    ("input_ids", dec_tensor.into()),
                    ("encoder_hidden_states", encoder_hidden.clone()),
                    ("encoder_attention_mask", enc_mask_tensor.into()),
                ],
            )?)?;

            let logits = dec_result[0].to_plain_array_view::<f32>()?;
            let seq_pos = logits.shape()[1] - 1;
            let vocab_size = logits.shape()[2];

            let next_id = (0..vocab_size)
                .map(|v| (v, logits[[0, seq_pos, v]]))
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .context("empty logits")?
                .0;

            if next_id as u32 == eos_id {
                break;
            }

            decoder_ids.push(next_id as i64);
        }

        // 4. Detokenize (skip the decoder start token)
        let output_ids: Vec<u32> = decoder_ids.iter().skip(1).map(|&id| id as u32).collect();
        let sql = tokenizer
            .decode(&output_ids, true)
            .map_err(|e| anyhow::anyhow!("decode failed: {}", e))?;

        let sql = sql
            .trim()
            .trim_start_matches("```sql")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        Ok(sql.to_string())
    }
}

/// Locate the first existing model file from a preference-ordered list.
fn find_model_file(model_dir: &Path, candidates: &[&str], kind: &str) -> Result<PathBuf> {
    for name in candidates {
        let path = model_dir.join(name);
        if path.exists() {
            return Ok(path);
        }
    }
    bail!(
        "no {} ONNX model found in {}. Expected one of: {}. \
         Run script/download-nquery-model.py to fetch the model files.",
        kind,
        model_dir.display(),
        candidates.join(", ")
    )
}

/// Load an ONNX model as a runnable plan, preferring the optimized graph but
/// falling back to the plain typed one — tract's ChangeAxes pass fails on
/// some variants of this model ("required_rank 2 vs 0" on Unsqueeze nodes).
fn load_runnable(path: &Path) -> Result<std::sync::Arc<TypedRunnableModel>> {
    let inference = tract_onnx::onnx().model_for_path(path)?;
    match inference.into_optimized() {
        Ok(typed) => Ok(typed.into_runnable()?),
        Err(_) => Ok(tract_onnx::onnx()
            .model_for_path(path)?
            .into_typed()?
            .into_runnable()?),
    }
}

/// Build the model input list in the graph's declared order.
///
/// ONNX exports do not guarantee an input order (e.g. this decoder declares
/// `encoder_hidden_states` before `input_ids`), so tensors are matched by
/// input name rather than position.
fn named_inputs(
    model: &TypedRunnableModel,
    values: impl IntoIterator<Item = (&'static str, TValue)>,
) -> Result<TVec<TValue>> {
    let mut values: std::collections::HashMap<&str, TValue> = values.into_iter().collect();
    let mut inputs = tvec!();
    for outlet in model.model().input_outlets()? {
        let name = model.model().node(outlet.node).name.clone();
        let value = values
            .remove(name.as_str())
            .with_context(|| format!("no value provided for model input '{}'", name))?;
        inputs.push(value);
    }
    Ok(inputs)
}

/// Read EOS / decoder-start token ids from `config.json` (T5 defaults: 1 / 0).
fn generation_config(model_dir: &Path) -> (u32, u32) {
    let cfg: Option<serde_json::Value> = std::fs::read_to_string(model_dir.join("config.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    match cfg {
        Some(cfg) => (
            cfg["eos_token_id"].as_u64().unwrap_or(1) as u32,
            cfg["decoder_start_token_id"].as_u64().unwrap_or(0) as u32,
        ),
        None => (1, 0),
    }
}
