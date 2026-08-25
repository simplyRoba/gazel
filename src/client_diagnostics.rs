use axum::http::StatusCode;
use serde::Deserialize;

use crate::api::error::{ApiError, JsonBody};

const MAX_PATHNAME_BYTES: usize = 1_024;
const MAX_EXCEPTION_TYPE_BYTES: usize = 80;
const MAX_EXCEPTION_MESSAGE_BYTES: usize = 512;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientDiagnosticStage {
    WindowError,
    UnhandledRejection,
    SettingsInitialization,
    LayoutVehicleInitialization,
    DashboardVehicleInitialization,
    VehicleLoading,
    ActiveVehicleSelection,
    FillupLoading,
    StatsLoading,
    StatsHistoryLoading,
    FleetStatsLoading,
    DashboardLoadingSnapshot,
}

impl ClientDiagnosticStage {
    const fn as_str(self) -> &'static str {
        match self {
            Self::WindowError => "window_error",
            Self::UnhandledRejection => "unhandled_rejection",
            Self::SettingsInitialization => "settings_initialization",
            Self::LayoutVehicleInitialization => "layout_vehicle_initialization",
            Self::DashboardVehicleInitialization => "dashboard_vehicle_initialization",
            Self::VehicleLoading => "vehicle_loading",
            Self::ActiveVehicleSelection => "active_vehicle_selection",
            Self::FillupLoading => "fillup_loading",
            Self::StatsLoading => "stats_loading",
            Self::StatsHistoryLoading => "stats_history_loading",
            Self::FleetStatsLoading => "fleet_stats_loading",
            Self::DashboardLoadingSnapshot => "dashboard_loading_snapshot",
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientDiagnosticOutcome {
    Started,
    Succeeded,
    Failed,
    Snapshot,
}

impl ClientDiagnosticOutcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Snapshot => "snapshot",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientDiagnostic {
    stage: ClientDiagnosticStage,
    outcome: ClientDiagnosticOutcome,
    pathname: String,
    exception_type: Option<String>,
    exception_message: Option<String>,
    settings_loading: Option<bool>,
    vehicles_loading: Option<bool>,
    active_vehicle_selected: Option<bool>,
    fillups_loading: Option<bool>,
    stats_loading: Option<bool>,
}

/// Record an allowlisted, content-free frontend runtime diagnostic.
///
/// # Errors
///
/// Returns a validation error when a text field is too long or contains
/// characters that are unsafe for production logs.
pub async fn record(
    JsonBody(diagnostic): JsonBody<ClientDiagnostic>,
) -> Result<StatusCode, ApiError> {
    if !valid_pathname(&diagnostic.pathname)
        || !valid_optional_text(
            diagnostic.exception_type.as_deref(),
            MAX_EXCEPTION_TYPE_BYTES,
        )
        || !valid_optional_text(
            diagnostic.exception_message.as_deref(),
            MAX_EXCEPTION_MESSAGE_BYTES,
        )
    {
        return Err(ApiError::Validation("CLIENT_DIAGNOSTIC_INVALID"));
    }

    let exception_type = diagnostic
        .exception_type
        .as_deref()
        .map_or("none", safe_log_text);
    let exception_message = diagnostic
        .exception_message
        .as_deref()
        .map_or("none", safe_log_text);
    tracing::debug!(
        stage = diagnostic.stage.as_str(),
        outcome = diagnostic.outcome.as_str(),
        pathname = diagnostic.pathname,
        exception_type,
        exception_message,
        settings_loading = ?diagnostic.settings_loading,
        vehicles_loading = ?diagnostic.vehicles_loading,
        active_vehicle_selected = ?diagnostic.active_vehicle_selected,
        fillups_loading = ?diagnostic.fillups_loading,
        stats_loading = ?diagnostic.stats_loading,
        "Frontend runtime diagnostics"
    );

    Ok(StatusCode::NO_CONTENT)
}

fn valid_pathname(value: &str) -> bool {
    value.starts_with('/')
        && value.len() <= MAX_PATHNAME_BYTES
        && !value.chars().any(char::is_control)
        && !value.contains(['?', '#'])
}

fn valid_optional_text(value: Option<&str>, max_bytes: usize) -> bool {
    value.is_none_or(|value| value.len() <= max_bytes && !value.chars().any(char::is_control))
}

fn safe_log_text(value: &str) -> &str {
    const REDACTED: &str = "Sensitive exception message redacted";
    const SENSITIVE_WORDS: [&str; 9] = [
        "client_secret",
        "code",
        "cookie",
        "nonce",
        "secret",
        "session_id",
        "state",
        "subject",
        "token",
    ];

    if value
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|word| {
            SENSITIVE_WORDS
                .iter()
                .any(|sensitive| word.eq_ignore_ascii_case(sensitive))
        })
    {
        REDACTED
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_rejects_query_fragments_controls_and_oversized_errors() {
        assert!(valid_pathname("/"));
        assert!(valid_pathname("/settings/vehicles"));
        assert!(!valid_pathname("/settings?tab=data"));
        assert!(!valid_pathname("/settings#data"));
        assert!(!valid_pathname("/settings\nforged"));
        assert!(!valid_pathname("relative"));
        assert!(!valid_optional_text(Some("error\nforged"), 80));
        assert!(!valid_optional_text(Some(&"x".repeat(81)), 80));
    }

    #[test]
    fn log_text_redacts_sensitive_words_without_hiding_normal_errors() {
        assert_eq!(
            safe_log_text("TypeError: load failed"),
            "TypeError: load failed"
        );
        assert_eq!(
            safe_log_text("token=do-not-log"),
            "Sensitive exception message redacted"
        );
        assert_eq!(
            safe_log_text("session_id:do-not-log"),
            "Sensitive exception message redacted"
        );
    }
}
