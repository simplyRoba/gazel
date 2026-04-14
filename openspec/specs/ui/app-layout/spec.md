## ADDED Requirements

### Requirement: Root layout structure

The root layout SHALL render an app shell consisting of a navigation region, a content region, and a pull-to-refresh indicator. The content region SHALL render the active route's page content. The pull-to-refresh indicator SHALL be a fixed-position element at the top of the viewport, hidden by default, that becomes visible during pull-to-refresh gestures.

#### Scenario: Layout renders navigation and content
- **WHEN** any route is loaded
- **THEN** the layout SHALL render a navigation region, a `<main>` content region, and a pull-to-refresh indicator element
- **AND** the active route's page content SHALL appear inside the content region

#### Scenario: Pull indicator present but hidden
- **WHEN** the layout renders and no pull gesture is active
- **THEN** the pull-to-refresh indicator element SHALL exist in the DOM but not be visible

### Requirement: Responsive sidebar navigation

The navigation SHALL adapt to three viewport breakpoints, switching between a bottom tab bar and a fixed left sidebar.

#### Scenario: Mobile bottom bar (viewport ≤ 768px)
- **WHEN** the viewport width is 768px or less
- **THEN** a fixed bottom tab bar SHALL be displayed
- **AND** the left sidebar SHALL be hidden
- **AND** the content region SHALL have no left margin
- **AND** the content region SHALL have bottom padding to clear the tab bar height plus the safe-area inset

#### Scenario: Tablet icon sidebar (viewport 769px–1279px)
- **WHEN** the viewport width is between 769px and 1279px
- **THEN** a fixed left sidebar of 64px width SHALL be displayed with icon-only nav items
- **AND** the bottom tab bar SHALL be hidden
- **AND** the content region SHALL have a left margin of 64px

#### Scenario: Widescreen expanded sidebar (viewport ≥ 1280px)
- **WHEN** the viewport width is 1280px or greater
- **THEN** the fixed left sidebar SHALL expand to 200px with icons and text labels
- **AND** the content region SHALL have a left margin of 200px

### Requirement: Navigation items

The navigation SHALL contain exactly two route items: Dashboard and Settings. The Dashboard item SHALL appear first. The Settings item SHALL be positioned at the bottom of the sidebar on tablet and desktop viewports.

#### Scenario: Dashboard nav item
- **WHEN** the user is on the root route `/` or any route starting with `/vehicles`
- **THEN** the Dashboard nav item SHALL be displayed as active

#### Scenario: Settings nav item
- **WHEN** the user is on any route starting with `/settings`
- **THEN** the Settings nav item SHALL be displayed as active

### Requirement: Fill-up CTA in navigation

The navigation SHALL include a prominent call-to-action button for adding fill-ups, integrated directly into the navigation bar.

#### Scenario: Mobile CTA button
- **WHEN** the viewport is 768px or less
- **THEN** a raised circular accent-colored button SHALL appear in the center of the bottom tab bar between the Dashboard and Settings items
- **AND** tapping it SHALL open the add fill-up flow

#### Scenario: Tablet CTA button
- **WHEN** the viewport is between 769px and 1279px
- **THEN** an icon-only accent-colored button SHALL appear in the sidebar below the logo
- **AND** clicking it SHALL open the add fill-up flow

#### Scenario: Widescreen CTA button
- **WHEN** the viewport is 1280px or greater
- **THEN** an accent-colored button with a label SHALL appear in the sidebar below the logo
- **AND** clicking it SHALL open the add fill-up flow

### Requirement: Logo display

The app logo SHALL be displayed in the sidebar on tablet and desktop viewports, and as a small mark in the page header on mobile.

#### Scenario: Logo in sidebar
- **WHEN** the viewport is 769px or greater
- **THEN** the logo SHALL appear at the top of the sidebar above the CTA button

#### Scenario: Logo on mobile
- **WHEN** the viewport is 768px or less
- **THEN** a small logo mark SHALL appear in the page header area

### Requirement: Content width tokens

The layout SHALL define three content width CSS custom properties that constrain page content to readable widths.

#### Scenario: Default content widths
- **WHEN** the viewport is less than 1280px
- **THEN** `--content-width-narrow` SHALL be 640px
- **AND** `--content-width-default` SHALL be 800px
- **AND** `--content-width-wide` SHALL be 1200px

#### Scenario: Widescreen content widths
- **WHEN** the viewport is 1280px or greater
- **THEN** `--content-width-narrow` SHALL be 720px
- **AND** `--content-width-default` SHALL be 960px
- **AND** `--content-width-wide` SHALL be 1400px

### Requirement: Safe area handling

The layout SHALL account for device safe areas (notch, home indicator) using `env(safe-area-inset-*)` values.

#### Scenario: Bottom safe area on mobile
- **WHEN** the device has a bottom safe area inset (e.g., iPhone home indicator)
- **THEN** the bottom tab bar height SHALL include the safe area inset
- **AND** the content bottom padding SHALL account for the total tab bar height including the safe area

### Requirement: Global CSS reset and base styles

The layout SHALL import a CSS reset and apply base typographic styles using the design system tokens.

#### Scenario: Base styles applied
- **WHEN** the app loads
- **THEN** `box-sizing: border-box` SHALL be applied to all elements
- **AND** the body SHALL use `--font-family` for font, `--color-text` for color, and `--color-bg` for background
- **AND** default margins and padding SHALL be reset to zero
