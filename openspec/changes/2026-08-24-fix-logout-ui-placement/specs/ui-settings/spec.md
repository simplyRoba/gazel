## REMOVED Requirements

### Requirement: Settings exposes local logout when authentication is enabled
**Reason**: Logout is an application-session action and belongs in persistent navigation chrome rather than in a Settings Authentication section.

**Migration**: Remove the Authentication section and logout control from Settings. Provide the conditional translated Sign out action under the `ui-app-layout` capability while preserving the existing logout request and redirect behavior.
