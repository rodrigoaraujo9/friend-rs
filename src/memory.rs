use crate::ollama::ChatMessage;

#[derive(Debug, Clone)]
pub struct ConversationMemory {
    system_prompt: String,
    recent: Vec<ChatMessage>,
    max_recent_messages: usize,
}

impl ConversationMemory {
    pub fn new(system_prompt: impl Into<String>, max_recent_messages: usize) -> Self {
        Self {
            system_prompt: system_prompt.into(),
            recent: Vec::new(),
            max_recent_messages,
        }
    }

    pub fn push_user(&mut self, content: impl Into<String>) {
        self.recent.push(ChatMessage {
            role: "user".into(),
            content: content.into(),
        });
        self.trim();
    }

    pub fn push_assistant(&mut self, content: impl Into<String>) {
        self.recent.push(ChatMessage {
            role: "assistant".into(),
            content: content.into(),
        });
        self.trim();
    }

    pub fn messages_for_model(&self) -> Vec<ChatMessage> {
        let mut messages = Vec::with_capacity(2 + self.recent.len());

        messages.push(ChatMessage {
            role: "system".into(),
            content: self.system_prompt.clone(),
        });

        // messages.push(ChatMessage {
        //     role: "system".into(),
        //     content: "Voice conversation mode: reply naturally and briefly. Usually answer in 1 to 3 short sentences. Use recent context. Sound like a real friend, not an assistant. Ask a short follow-up question sometimes, but not every turn.".into(),
        // });

        messages.extend(self.recent.clone());
        messages
    }

    fn trim(&mut self) {
        if self.recent.len() > self.max_recent_messages {
            let drop_count = self.recent.len() - self.max_recent_messages;
            self.recent.drain(0..drop_count);
        }
    }
}
