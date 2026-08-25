export type ClientDiagnosticStage =
  | "window_error"
  | "unhandled_rejection"
  | "settings_initialization"
  | "layout_vehicle_initialization"
  | "dashboard_vehicle_initialization"
  | "vehicle_loading"
  | "active_vehicle_selection"
  | "fillup_loading"
  | "stats_loading"
  | "stats_history_loading"
  | "fleet_stats_loading"
  | "dashboard_loading_snapshot";

export type ClientDiagnosticOutcome =
  "started" | "succeeded" | "failed" | "snapshot";

export interface ClientLoadingState {
  settings_loading?: boolean;
  vehicles_loading?: boolean;
  active_vehicle_selected?: boolean;
  fillups_loading?: boolean;
  stats_loading?: boolean;
}

export interface ClientDiagnostic extends ClientLoadingState {
  stage: ClientDiagnosticStage;
  outcome: ClientDiagnosticOutcome;
  exception_type?: string;
  exception_message?: string;
}

export interface ExceptionDetails {
  exception_type: string;
  exception_message: string;
}

export function reportClientDiagnostic(diagnostic: ClientDiagnostic): void {
  if (typeof window === "undefined") return;

  const payload = {
    ...diagnostic,
    pathname: window.location.pathname,
  };

  try {
    const request = fetch("/client-diagnostics", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
      keepalive: true,
    });
    void request.catch(() => {
      // Diagnostics must never affect application behavior.
    });
  } catch {
    // Diagnostics must never affect application behavior.
  }
}

export function exceptionDetails(error: unknown): ExceptionDetails {
  if (error instanceof Error) {
    return {
      exception_type: safeLogText(error.name || "Error", 20),
      exception_message: safeLogText(error.message, 128),
    };
  }

  if (typeof error === "string") {
    return {
      exception_type: "NonError",
      exception_message: safeLogText(error, 128),
    };
  }

  return {
    exception_type: "NonError",
    exception_message: "Non-Error exception",
  };
}

export function installGlobalClientDiagnostics(): () => void {
  const handleError = (event: ErrorEvent): void => {
    const details =
      event.error instanceof Error
        ? exceptionDetails(event.error)
        : {
            exception_type: "ErrorEvent",
            exception_message: safeLogText(event.message, 128),
          };
    reportClientDiagnostic({
      stage: "window_error",
      outcome: "failed",
      ...details,
    });
  };

  const handleUnhandledRejection = (event: PromiseRejectionEvent): void => {
    reportClientDiagnostic({
      stage: "unhandled_rejection",
      outcome: "failed",
      ...exceptionDetails(event.reason),
    });
  };

  window.addEventListener("error", handleError);
  window.addEventListener("unhandledrejection", handleUnhandledRejection);

  return () => {
    window.removeEventListener("error", handleError);
    window.removeEventListener("unhandledrejection", handleUnhandledRejection);
  };
}

function safeLogText(value: string, maxCharacters: number): string {
  let sanitized = "";
  for (const character of value) {
    const codePoint = character.codePointAt(0) ?? 0;
    sanitized += codePoint < 32 || codePoint === 127 ? " " : character;
    if (sanitized.length >= maxCharacters) break;
  }

  const sensitiveWords = new Set([
    "client_secret",
    "code",
    "cookie",
    "nonce",
    "secret",
    "session_id",
    "state",
    "subject",
    "token",
  ]);
  const containsSensitiveWord = sanitized
    .toLowerCase()
    .split(/[^a-z0-9_]+/)
    .some((word) => sensitiveWords.has(word));
  return containsSensitiveWord
    ? "Sensitive exception message redacted"
    : sanitized;
}
