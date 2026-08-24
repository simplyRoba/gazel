## MODIFIED Requirements

### Requirement: Root layout structure
The root layout SHALL render the protected application shell for application routes and a standalone public surface for `/login`. Protected application content SHALL include navigation, a main content region, and a pull-to-refresh indicator; the login route SHALL render without protected shell chrome or hydration.

#### Scenario: Layout renders navigation and content
- **WHEN** an authenticated application route other than `/login` is loaded
- **THEN** the layout SHALL render a navigation region, a `<main>` content region, and a pull-to-refresh indicator element
- **AND** the active route's page content SHALL appear inside the content region

#### Scenario: Pull indicator present but hidden
- **WHEN** the protected application layout renders and no pull gesture is active
- **THEN** the pull-to-refresh indicator element SHALL exist in the DOM but not be visible

#### Scenario: Public login layout
- **WHEN** `/login` is loaded
- **THEN** the layout SHALL render a standalone public login gate while auth config is resolved
- **AND** SHALL NOT render application navigation, fill-up controls, or the pull-to-refresh indicator
- **AND** SHALL NOT initialize settings, vehicles, fill-ups, stats, or any other protected store
- **AND** SHALL render the login experience only when auth config reports enabled, otherwise replacing navigation with `/`
