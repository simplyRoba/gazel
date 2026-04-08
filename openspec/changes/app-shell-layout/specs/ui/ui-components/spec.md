## ADDED Requirements

### Requirement: PageContainer component

A reusable `PageContainer` component SHALL constrain page content to a maximum width and apply consistent horizontal padding and centering.

#### Scenario: Default width
- **WHEN** `PageContainer` is used without a `width` prop
- **THEN** the content SHALL be constrained to `--content-width-default`
- **AND** the content SHALL be horizontally centered with `margin: 0 auto`

#### Scenario: Narrow width
- **WHEN** `PageContainer` is used with `width="narrow"`
- **THEN** the content SHALL be constrained to `--content-width-narrow`

#### Scenario: Wide width
- **WHEN** `PageContainer` is used with `width="wide"`
- **THEN** the content SHALL be constrained to `--content-width-wide`

#### Scenario: Content rendering
- **WHEN** child content is placed inside `PageContainer`
- **THEN** all children SHALL render inside the constrained container

### Requirement: EmptyState component

A reusable `EmptyState` component SHALL display a centered message with an icon, heading, description, and an optional action button, used when a list or page has no data.

#### Scenario: Full empty state
- **WHEN** `EmptyState` is rendered with an `icon`, `heading`, `description`, and an `action` snippet
- **THEN** the icon SHALL render above the heading
- **AND** the heading SHALL be displayed in `--font-lg` weight semibold
- **AND** the description SHALL be displayed below the heading in `--color-text-secondary`
- **AND** the action snippet SHALL render below the description

#### Scenario: Empty state without action
- **WHEN** `EmptyState` is rendered without an `action` snippet
- **THEN** the icon, heading, and description SHALL render
- **AND** no action area SHALL be displayed

#### Scenario: Visual centering
- **WHEN** `EmptyState` is rendered
- **THEN** all content SHALL be vertically and horizontally centered within its container
- **AND** there SHALL be consistent spacing between the icon, heading, description, and action
