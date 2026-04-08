import { describe, it, expect } from "vitest";
import { render } from "@testing-library/svelte";
import PageContainer from "./PageContainer.svelte";

describe("PageContainer", () => {
  it("renders with default width class", () => {
    const { container } = render(PageContainer);
    const el = container.querySelector(".page-container");
    expect(el).toBeTruthy();
    expect(el?.classList.contains("narrow")).toBe(false);
    expect(el?.classList.contains("wide")).toBe(false);
  });

  it("applies narrow class when width is narrow", () => {
    const { container } = render(PageContainer, { props: { width: "narrow" } });
    const el = container.querySelector(".page-container");
    expect(el?.classList.contains("narrow")).toBe(true);
  });

  it("applies wide class when width is wide", () => {
    const { container } = render(PageContainer, { props: { width: "wide" } });
    const el = container.querySelector(".page-container");
    expect(el?.classList.contains("wide")).toBe(true);
  });
});
