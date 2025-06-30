//! Self-correction loop: Critic LLM reviews primary LLM output
use crate::ai::prompts::templates::CRITIC_TEMPLATE;

/// Run a self-correction loop: Critic LLM reviews primary LLM output and suggests corrections
pub async fn run_self_correction<L: crate::ai::api::llm_provider::LlmProvider>(
    llm: &L,
    prompt: &str,
    ai_answer: &str,
    context: &str,
) -> Result<String, String> {
    let critic_prompt = CRITIC_TEMPLATE
        .replace("{{prompt}}", prompt)
        .replace("{{ai_answer}}", ai_answer)
        .replace("{{context}}", context);
    llm.generate_explanation(&critic_prompt)
        .await
        .map_err(|e| e.to_string())
}
