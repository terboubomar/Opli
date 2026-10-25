import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  open: vi.fn(),
  onDragDropEvent: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: mocks.open }));
vi.mock("@tauri-apps/api/webview", () => ({ getCurrentWebview: () => ({ onDragDropEvent: mocks.onDragDropEvent }) }));

const preview = {
  plan_id: "opaque-approved-plan",
  output_root: "C:/output",
  recipe_id: "organize",
  total_bytes: 3,
  warnings: [],
  items: [{ source: "C:/input/a.txt", destination: "C:/output/Intent - Organized/Documents/a.txt", bytes: 3 }],
};

async function selectAndPreview() {
  const user = userEvent.setup();
  mocks.open.mockResolvedValueOnce(["C:/input/a.txt"]).mockResolvedValueOnce("C:/output");
  mocks.invoke.mockImplementation((command: string) => command === "preview_recipe" ? Promise.resolve(preview) : Promise.resolve({ completed: [], failed: [] }));
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Choose files or drop them here" }));
  await user.click(screen.getByRole("button", { name: /Browse/ }));
  await user.click(screen.getByRole("button", { name: /Preview workflow/ }));
  await screen.findByRole("button", { name: "Run recipe" });
  return user;
}

afterEach(cleanup);

beforeEach(() => {
  vi.clearAllMocks();
  mocks.onDragDropEvent.mockResolvedValue(() => {});
});

describe("approved plan UI contract", () => {
  it("executes only the approved plan ID", async () => {
    const user = await selectAndPreview();
    expect(mocks.invoke).toHaveBeenCalledWith("preview_recipe", {
      paths: ["C:/input/a.txt"], recipeId: "organize", outputDirectory: "C:/output",
    });
    await user.click(screen.getByRole("button", { name: "Run recipe" }));
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("execute_recipe", { planId: "opaque-approved-plan" }));
  });

  it("invalidates the preview when the recipe changes", async () => {
    const user = await selectAndPreview();
    await user.click(screen.getByRole("button", { name: /Rename cleanly/ }));
    expect(screen.queryByRole("button", { name: "Run recipe" })).toBeNull();
    expect(screen.getByRole("button", { name: /Preview workflow/ })).toBeTruthy();
    expect(mocks.invoke).toHaveBeenCalledTimes(1);
  });

  it("shows a backend retry instruction only once", async () => {
    const user = await selectAndPreview();
    mocks.invoke.mockImplementation((command: string) =>
      command === "execute_recipe"
        ? Promise.reject("Output folder changed. Preview again.")
        : Promise.resolve(preview)
    );
    await user.click(screen.getByRole("button", { name: "Run recipe" }));
    expect(await screen.findByText("Output folder changed. Preview again.")).toBeTruthy();
    expect(screen.queryByText(/Preview again\. Preview again\./)).toBeNull();
  });

  it("shows Windows extended paths without the internal device prefix", async () => {
    const user = userEvent.setup();
    mocks.open.mockResolvedValueOnce(["C:/input/a.txt"]).mockResolvedValueOnce("C:/output");
    mocks.invoke.mockResolvedValueOnce({
      ...preview,
      items: [{ ...preview.items[0], destination: "\\\\?\\C:\\output\\a.txt" }],
    });
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Choose files or drop them here" }));
    await user.click(screen.getByRole("button", { name: /Browse/ }));
    await user.click(screen.getByRole("button", { name: /Preview workflow/ }));
    expect((await screen.findAllByText("C:\\output\\a.txt")).length).toBeGreaterThan(0);
    expect(screen.queryByText("\\\\?\\C:\\output\\a.txt")).toBeNull();
  });

  it("marks a zero-copy run as failed instead of successful", async () => {
    const user = await selectAndPreview();
    mocks.invoke.mockImplementation((command: string) =>
      command === "execute_recipe"
        ? Promise.resolve({ completed: [], failed: ["a.txt: Access is denied."] })
        : Promise.resolve(preview)
    );
    await user.click(screen.getByRole("button", { name: "Run recipe" }));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("No files copied");
    expect(alert.textContent).toContain("1 failure");
    expect(screen.getByRole("list", { name: "Files that could not be copied" })).toBeTruthy();
  });
});

