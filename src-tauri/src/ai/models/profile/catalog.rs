use std::{collections::BTreeMap, path::Path, sync::Arc, sync::OnceLock, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::Mutex;

use super::ModelProfile;
use crate::ai::contracts::ProviderKind;

const SOURCE: &str = "https://models.dev/api.json";
const CACHE_FILE: &str = "model-profiles.json";
const FRESH_FOR_MILLIS: i64 = 24 * 60 * 60 * 1000;
const RETRY_AFTER_MILLIS: i64 = 10 * 60 * 1000;
const MAX_SOURCE_BYTES: usize = 64 * 1024 * 1024;

/// Published limits by models.dev provider id, then by the provider's own model identifier.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Catalog {
    fetched_at: i64,
    providers: BTreeMap<String, BTreeMap<String, ModelProfile>>,
}

enum Sources {
    Providers(&'static [&'static str]),
    /// A generic OpenAI-compatible endpoint has no provider identity; the model identifier is
    /// all that names the model, so the limits every listing provider honours apply.
    Any,
}

const fn sources(provider: ProviderKind) -> Sources {
    Sources::Providers(match provider {
        ProviderKind::OpenAiCompatible => return Sources::Any,
        ProviderKind::OpenAi => &["openai"],
        ProviderKind::Anthropic => &["anthropic"],
        ProviderKind::Gemini => &["google"],
        ProviderKind::DeepSeek => &["deepseek"],
        ProviderKind::Groq => &["groq"],
        ProviderKind::OpenRouter => &["openrouter"],
        ProviderKind::DashScope => &["alibaba-cn", "alibaba"],
        ProviderKind::Zhipu => &["zhipuai", "zai"],
        ProviderKind::SiliconFlow => &["siliconflow-cn", "siliconflow"],
        ProviderKind::Grok => &["xai"],
        ProviderKind::Mistral => &["mistral"],
        ProviderKind::Together => &["togetherai"],
        ProviderKind::StepFun => &["stepfun"],
        ProviderKind::Moonshot => &["moonshotai-cn", "moonshotai"],
        ProviderKind::Doubao => &["volcengine"],
        ProviderKind::MiniMax => &["minimax-cn", "minimax"],
        // A local server's limits are its own runtime configuration, not a published fact.
        ProviderKind::Ollama | ProviderKind::Kling => &[],
    })
}

impl Catalog {
    pub(super) fn parse(source: &[u8], fetched_at: i64) -> Result<Self, String> {
        let published: BTreeMap<String, Value> =
            serde_json::from_slice(source).map_err(|error| error.to_string())?;
        let providers = published
            .into_iter()
            .filter_map(|(provider, entry)| {
                let models = entry
                    .get("models")?
                    .as_object()?
                    .iter()
                    .filter_map(|(id, model)| Some((id.clone(), published(model.get("limit")?)?)))
                    .collect::<BTreeMap<_, _>>();
                (!models.is_empty()).then_some((provider, models))
            })
            .collect::<BTreeMap<_, _>>();
        if providers.is_empty() {
            return Err("models.dev catalog publishes no model limits".into());
        }
        Ok(Self {
            fetched_at,
            providers,
        })
    }

    pub(super) fn profile(&self, provider: ProviderKind, model: &str) -> ModelProfile {
        let published = match sources(provider) {
            Sources::Providers(ids) => ids
                .iter()
                .find_map(|id| self.providers.get(*id)?.get(model).copied()),
            Sources::Any => self
                .providers
                .values()
                .filter_map(|models| models.get(model))
                .copied()
                .reduce(|left, right| ModelProfile {
                    max_input_tokens: smallest(left.max_input_tokens, right.max_input_tokens),
                    max_output_tokens: smallest(left.max_output_tokens, right.max_output_tokens),
                }),
        };
        published.unwrap_or_default()
    }

    fn fresh(&self, now: i64) -> bool {
        now.saturating_sub(self.fetched_at) < FRESH_FOR_MILLIS
    }
}

/// models.dev writes `0` for a limit it does not know.
fn published(limit: &Value) -> Option<ModelProfile> {
    let tokens = |key: &str| {
        limit
            .get(key)
            .and_then(Value::as_u64)
            .filter(|tokens| *tokens > 0)
            .and_then(|tokens| u32::try_from(tokens).ok())
    };
    Some(ModelProfile {
        max_input_tokens: Some(tokens("input").or_else(|| tokens("context"))?),
        max_output_tokens: tokens("output"),
    })
}

fn smallest(left: Option<u32>, right: Option<u32>) -> Option<u32> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (known, None) | (None, known) => known,
    }
}

struct Loaded {
    catalog: Option<Arc<Catalog>>,
    next_attempt: i64,
}

static LOADED: Mutex<Loaded> = Mutex::const_new(Loaded {
    catalog: None,
    next_attempt: 0,
});

/// The catalog refreshes once a day; until a refresh succeeds the last copy keeps serving.
pub(super) async fn current<R: Runtime>(app: &AppHandle<R>) -> Result<Arc<Catalog>, String> {
    let mut loaded = LOADED.lock().await;
    let now = crate::foundation::clock::unix_time_millis()?;
    let path = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join(CACHE_FILE);
    if loaded.catalog.is_none() {
        loaded.catalog = read_cache(&path).await.map(Arc::new);
    }
    if let Some(catalog) = loaded.catalog.as_ref().filter(|catalog| catalog.fresh(now)) {
        return Ok(catalog.clone());
    }
    if now >= loaded.next_attempt {
        match fetch(now).await {
            Ok(catalog) => {
                if let Err(error) = write_cache(&path, &catalog).await {
                    log::warn!("Model limits catalog cache was not written: {error}");
                }
                let catalog = Arc::new(catalog);
                loaded.catalog = Some(catalog.clone());
                return Ok(catalog);
            }
            Err(error) => {
                loaded.next_attempt = now + RETRY_AFTER_MILLIS;
                log::warn!("models.dev catalog refresh failed: {error}");
            }
        }
    }
    loaded
        .catalog
        .clone()
        .ok_or_else(|| "models.dev catalog is unavailable".into())
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        crate::foundation::http::client_builder()
            .timeout(Duration::from_secs(20))
            .build()
            .expect("the static model catalog HTTP client configuration must be valid")
    })
}

async fn fetch(now: i64) -> Result<Catalog, String> {
    let response = client()
        .get(SOURCE)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| error.without_url().to_string())?;
    if response
        .content_length()
        .is_some_and(|length| length > MAX_SOURCE_BYTES as u64)
    {
        return Err("models.dev catalog exceeds the size limit".into());
    }
    let source = response
        .bytes()
        .await
        .map_err(|error| error.without_url().to_string())?;
    if source.len() > MAX_SOURCE_BYTES {
        return Err("models.dev catalog exceeds the size limit".into());
    }
    Catalog::parse(&source, now)
}

async fn read_cache(path: &Path) -> Option<Catalog> {
    let cached = tokio::fs::read(path).await.ok()?;
    serde_json::from_slice(&cached).ok()
}

async fn write_cache(path: &Path, catalog: &Catalog) -> Result<(), String> {
    let encoded = serde_json::to_vec(catalog).map_err(|error| error.to_string())?;
    if let Some(directory) = path.parent() {
        tokio::fs::create_dir_all(directory)
            .await
            .map_err(|error| error.to_string())?;
    }
    let part = path.with_extension("json.part");
    tokio::fs::write(&part, encoded)
        .await
        .map_err(|error| error.to_string())?;
    tokio::fs::rename(&part, path)
        .await
        .map_err(|error| error.to_string())
}
