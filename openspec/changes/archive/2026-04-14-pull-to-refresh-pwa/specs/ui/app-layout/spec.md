## MODIFIED Requirements

### Requirement: Root layout structure

The root layout SHALL render an app shell consisting of a navigation region, a content region, and a pull-to-refresh indicator. The content region SHALL render the active route's page content. The pull-to-refresh indicator SHALL be a fixed-position element at the top of the viewport, hidden by default, that becomes visible during pull-to-refresh gestures.

#### Scenario: Layout renders navigation and content
- **WHEN** any route is loaded
- **THEN** the layout SHALL render a navigation region, a `<main>` content region, and a pull-to-refresh indicator element
- **AND** the active route's page content SHALL appear inside the content region

#### Scenario: Pull indicator present but hidden
- **WHEN** the layout renders and no pull gesture is active
- **THEN** the pull-to-refresh indicator element SHALL exist in the DOM but not be visible
