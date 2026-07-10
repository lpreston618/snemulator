use std::time::{Duration, Instant};

use crate::app::theme::AppTheme;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageKind {
    Info,
    Success,
    Warning,
    Error,
    Debug,
}

impl MessageKind {
    pub fn colors(self, app_theme: &AppTheme) -> (egui::Color32, egui::Color32) {
        match self {
            MessageKind::Error          => (app_theme.error,          app_theme.msg_error_bg),
            MessageKind::Warning        => (app_theme.warning,        app_theme.msg_warning_bg),
            MessageKind::Success        => (app_theme.success,        app_theme.msg_success_bg),
            MessageKind::Info           => (app_theme.info,           app_theme.msg_info_bg),
            MessageKind::Debug          => (app_theme.msg_debug,      app_theme.msg_debug_bg),
        }
    }
}

pub struct Message {
    pub kind: MessageKind,
    pub text: String,
    pub created: Instant,
    pub lifetime: Duration,
    pub count: u32,          // for dedup: how many times this fired
    pub id: u64,             // stable unique id for animation tracking
}

impl Message {
    pub fn alpha(&self, fade: Duration) -> Option<f32> {
        let elapsed = self.created.elapsed();
        if elapsed >= self.lifetime {
            return None;
        }
        let fade_s = fade.as_secs_f32();
        let e = elapsed.as_secs_f32();
        let total = self.lifetime.as_secs_f32();

        let fade_in = (e / fade_s).clamp(0.0, 1.0);
        let fade_out = ((total - e) / fade_s).clamp(0.0, 1.0);
        Some(fade_in.min(fade_out))
    }

    /// Returns the current fade phase: (alpha, slide_progress)
    /// slide_progress: 1.0 = fully in place, 0.0 = fully offset
    pub fn transition(&self, fade: Duration) -> Option<(f32, f32)> {
        let alpha = self.alpha(fade)?;
        // Use a slightly eased version for slide so it feels snappy
        let slide = ease_out_cubic(alpha);
        Some((alpha, slide))
    }

    pub fn display_text(&self) -> String {
        if self.count > 1 {
            format!("{} (x{})", self.text, self.count)
        } else {
            self.text.clone()
        }
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

pub struct MessageQueue {
    pub messages: Vec<Message>,
    next_id: u64,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            next_id: 0,
        }
    }

    pub fn push(
        &mut self,
        kind: MessageKind,
        text: impl Into<String>,
        lifetime: Duration,
        log_level: Option<log::Level>,
    ) {
        let text = text.into();

        if let Some(level) = log_level {
            log::log!(level, "{}", text);
        }
        
        // Check for duplicates
        if let Some(msg) = self.messages.iter_mut().find(|m| m.text == text && m.kind == kind) {
            msg.count += 1;
            msg.created = Instant::now(); // reset lifetime
            return;
        }

        self.messages.push(Message {
            kind,
            text,
            created: Instant::now(),
            lifetime,
            count: 1,
            id: self.next_id,
        });
        self.next_id += 1;
    }
}