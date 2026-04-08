import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import EmptyState from "./EmptyState.svelte";

describe("EmptyState", () => {
  it("renders heading and description", () => {
    render(EmptyState, {
      props: {
        heading: "No vehicles yet",
        description: "Add your first vehicle to start tracking.",
      },
    });
    expect(screen.getByText("No vehicles yet")).toBeTruthy();
    expect(
      screen.getByText("Add your first vehicle to start tracking."),
    ).toBeTruthy();
  });

  it("does not render action area when action is not provided", () => {
    const { container } = render(EmptyState, {
      props: {
        heading: "Empty",
        description: "Nothing here.",
      },
    });
    expect(container.querySelector(".empty-action")).toBeNull();
  });
});
