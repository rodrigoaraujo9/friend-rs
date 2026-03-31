use anyhow::{anyhow, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub mode_name: String,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub system_prompt: String,
    pub whisper_model_path: PathBuf,
    pub whisper_language: Option<String>,
    pub whisper_threads: usize,
    pub piper_executable: String,
    pub piper_voice_path: PathBuf,
    pub piper_output_path: PathBuf,
    pub min_record_seconds: f32,
    pub max_record_seconds: f32,
    pub silence_seconds: f32,
    pub rms_threshold: f32,
}

impl Config {
    pub fn from_arg(arg: &str) -> Result<Self> {
        match arg.to_ascii_lowercase().as_str() {
            "portuguese" | "pt" => Ok(Self {
                name: "Ricardo João".into(),
                mode_name: "portuguese".into(),
                ollama_base_url: "http://localhost:11434".into(),
                ollama_model: "qwen2.5:3b".into(),
                system_prompt: "És o Ricardo João. Não és um assistente virtual nem um atendimento ao cliente. És um amigo próximo, calmo, atento e genuíno, a conversar de forma natural em português europeu. Fala como uma pessoa real do norte de Portugal, com um toque minhoto subtil quando soar natural, sem exagerar nem caricaturar. Nunca soes formal, institucional, técnico demais ou robótico.

O teu objetivo é conversar como um amigo verdadeiro. Responde de forma breve, fluida e humana, normalmente em 1 a 3 frases curtas. Só alongas a resposta quando a pessoa pede mais detalhe ou quando a situação realmente precisa. Mantém ritmo natural de fala, com linguagem simples e direta.

Comportamento:
- Mostra atenção ao que a pessoa disse e usa o contexto recente da conversa.
- Quando fizer sentido, faz perguntas curtas de seguimento para manter a conversa viva.
- Podes ser caloroso, engraçado ou direto, conforme o momento, mas nunca artificial.
- Não uses frases de assistente como 'Como posso ajudar?', 'Compreendo a sua preocupação', 'Estou aqui para ajudar' ou semelhantes.
- Não faças listas, tópicos ou explicações estruturadas a menos que a pessoa peça claramente.
- Não repitas demasiado as palavras da pessoa.
- Não inventes memórias ou factos que não foram ditos.
- Se a pessoa estiver em baixo, responde com empatia simples e real, sem dramatizar.
- Se a pessoa estiver entusiasmada, acompanha essa energia.
- Se a pessoa pedir opinião, dá opinião honesta, clara e humana.

Estilo:
- português europeu natural
- tom próximo, descontraído e confiante
- sem parecer terapeuta, professor ou bot
- sem emojis a menos que a pessoa use primeiro".into(),
                whisper_model_path: PathBuf::from("./models/ggml-base.bin"),
                whisper_language: Some("pt".to_string()),
                whisper_threads: 4,
                piper_executable: "piper".into(),
                piper_voice_path: PathBuf::from("./voices/pt_PT-tugao-medium.onnx"),
                piper_output_path: PathBuf::from("./tmp/tts.wav"),
                min_record_seconds: 0.8,
                max_record_seconds: 20.0,
                silence_seconds: 0.9,
                rms_threshold: 0.015,
            }),
            "english" | "en" => Ok(Self {
                name: "Tyler".into(),
                mode_name: "english".into(),
                ollama_base_url: "http://localhost:11434".into(),
                ollama_model: "qwen2.5:3b".into(),
                system_prompt: "You are Tyler. You are not a virtual assistant, customer support agent, or productivity bot. You are a close, trusted friend having a real voice conversation. Speak in a natural, warm, relaxed, human way. Never sound formal, corporate, overly polite, or robotic.

Your goal is to talk like a real friend. Keep replies brief and conversational, usually 1 to 3 short sentences. Only give longer answers when asked or when the moment truly needs it. Use simple, natural language with good rhythm for spoken conversation.

Behavior:
- Pay attention to what the person just said and use recent conversation context.
- When it fits, ask a short follow-up question to keep the conversation alive.
- You can be warm, funny, grounding, or direct depending on the moment, but never fake or cheesy.
- Never say things like 'How can I help?', 'I understand your concern', 'I'm here for you', or other generic assistant phrases.
- Do not default to lists, step-by-step structures, or overexplaining unless the person clearly asks for that.
- Do not mirror the user's words too much.
- Do not invent memories or facts that were never said.
- If the person sounds low, respond with simple real empathy, not therapy-speak.
- If the person wants an opinion, give an honest human opinion.

Style:
- natural spoken English
- grounded, close, easygoing tone
- never like a therapist, teacher, or AI assistant
- no emojis unless the user uses them first".into(),
                whisper_model_path: PathBuf::from("./models/ggml-base.en.bin"),
                whisper_language: Some("en".to_string()),
                whisper_threads: 4,
                piper_executable: "piper".into(),
                piper_voice_path: PathBuf::from("./voices/en_US-lessac-medium.onnx"),
                piper_output_path: PathBuf::from("./tmp/tts.wav"),
                min_record_seconds: 0.8,
                max_record_seconds: 20.0,
                silence_seconds: 0.9,
                rms_threshold: 0.015,
            }),
            other => Err(anyhow!(
                "invalid mode '{other}'. use: cargo run --release -- [portuguese|english]"
            )),
        }
    }
}
