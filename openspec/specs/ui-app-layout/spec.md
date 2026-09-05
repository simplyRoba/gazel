## Purpose

Application shell layout: root structure, responsive navigation with a fill-up CTA, logo display, readable content widths, responsive gutters, and safe-area handling.

## Requirements

### Requirement: Root layout structure
The root layout SHALL render the protected application shell for application routes and a standalone public surface for `/login`. Protected application content SHALL include navigation, a main content region, and a pull-to-refresh indicator; the login route SHALL render without protected shell chrome. For protected application routes, the pull-to-refresh indicator SHALL be fixed at the top of the viewport, hidden by default, and visible during pull-to-refresh gestures.

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
- **THEN** a raised diamond-shaped accent-colored button with an upright plus icon SHALL appear in the center of the bottom tab bar between the Dashboard and Settings items
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

The app logo SHALL be displayed in the sidebar on tablet and desktop viewports.

#### Scenario: Logo in sidebar
- **WHEN** the viewport is 769px or greater
- **THEN** the logo SHALL appear at the top of the sidebar above the CTA button

### Requirement: Readable page content widths

The layout SHALL support narrow, default, and wide page-content variants with responsive maximum widths.

#### Scenario: Default content widths
- **WHEN** the viewport is less than 1280px
- **THEN** narrow page content SHALL have a maximum width of 640px
- **AND** default page content SHALL have a maximum width of 800px
- **AND** wide page content SHALL have a maximum width of 1200px

#### Scenario: Widescreen content widths
- **WHEN** the viewport is 1280px or greater
- **THEN** narrow page content SHALL have a maximum width of 720px
- **AND** default page content SHALL have a maximum width of 960px
- **AND** wide page content SHALL have a maximum width of 1400px

### Requirement: Responsive content gutters

Page content SHALL have consistent responsive horizontal gutters.

#### Scenario: Mobile content gutter
- **WHEN** the viewport is below 769px
- **THEN** page content SHALL have a 16px gutter on each side

#### Scenario: Tablet content gutter
- **WHEN** the viewport is at least 769px and below 1280px
- **THEN** page content SHALL have a 24px gutter on each side

#### Scenario: Widescreen content gutter
- **WHEN** the viewport is at least 1280px
- **THEN** page content SHALL have a 32px gutter on each side

### Requirement: Safe area handling

The layout SHALL keep navigation and page content clear of device safe areas such as notches and home indicators.

#### Scenario: Bottom safe area on mobile
- **WHEN** the device has a bottom safe area inset (e.g., iPhone home indicator)
- **THEN** the bottom tab bar height SHALL include the safe area inset
- **AND** the content bottom padding SHALL account for the total tab bar height including the safe area
