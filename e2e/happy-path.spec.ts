import { test, expect } from "@playwright/test"
import { setupTauriFromFixtures, FIXTURE_NOTES } from "./helpers"

test.describe("Happy Path E2E", () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriFromFixtures(page)
  })

  test("shows welcome screen before workspace is opened", async ({ page }) => {
    await page.goto("/")
    await expect(page.getByText("Welcome to Scrapbox Clone MD")).toBeVisible()
    await expect(page.getByText("Select a folder to open as workspace")).toBeVisible()
    await expect(page.getByRole("button", { name: "Open Folder" })).toBeVisible()
  })

  test("opens workspace and displays notes in sidebar", async ({ page }) => {
    await page.goto("/")

    // Click "Open Folder" — dialog returns fixture workspace path, then open_workspace + list_notes are called
    await page.getByRole("button", { name: "Open Folder" }).click()

    // Workspace name should appear in sidebar
    await expect(page.getByText("test-workspace")).toBeVisible()

    // Notes derived from fixture .md files should be listed in sidebar
    await expect(page.getByText("First Note")).toBeVisible()
    await expect(page.getByText("Second Note")).toBeVisible()

    // Main area should show instruction text
    await expect(page.getByText("Select a note from the sidebar to start editing.")).toBeVisible()
  })

  test("selects a note and displays it in the editor", async ({ page }) => {
    await page.goto("/")
    await page.getByRole("button", { name: "Open Folder" }).click()
    await expect(page.getByText("First Note")).toBeVisible()

    // Click on the first note in the sidebar
    await page.getByText("First Note").click()

    // Editor should show the note title
    await expect(page.locator("h2").filter({ hasText: "First Note" })).toBeVisible()

    // Editor should show "Saved" status
    await expect(page.getByText("Saved")).toBeVisible()

    // Editor content area should contain the note body (from first-note.md fixture)
    await expect(page.getByText("This is the first note content.")).toBeVisible()
  })

  test("switches between notes", async ({ page }) => {
    await page.goto("/")
    await page.getByRole("button", { name: "Open Folder" }).click()
    await expect(page.getByText("First Note")).toBeVisible()

    // Select first note
    await page.getByText("First Note").click()
    await expect(page.getByText("This is the first note content.")).toBeVisible()

    // Switch to second note
    await page.getByText("Second Note").click()
    await expect(page.locator("h2").filter({ hasText: "Second Note" })).toBeVisible()
    await expect(page.getByText("Another note here.")).toBeVisible()
  })

  test("edits a note and triggers auto-save", async ({ page }) => {
    await page.goto("/")
    await page.getByRole("button", { name: "Open Folder" }).click()
    await expect(page.getByText("First Note")).toBeVisible()

    // Select a note
    await page.getByText("First Note").click()
    await expect(page.getByText("Saved")).toBeVisible()

    // Type in the editor (Tiptap editor uses contenteditable)
    const editor = page.locator(".tiptap")
    await editor.click()
    await page.keyboard.press("End")
    await page.keyboard.type("New content added.")

    // Status should change to "Unsaved"
    await expect(page.getByText("Unsaved")).toBeVisible()

    // Wait for auto-save (2 seconds debounce + some buffer)
    await expect(page.getByText("Saved")).toBeVisible({ timeout: 5000 })
  })

  test("saves a note with Ctrl+S", async ({ page }) => {
    await page.goto("/")
    await page.getByRole("button", { name: "Open Folder" }).click()
    await expect(page.getByText("First Note")).toBeVisible()

    // Select a note
    await page.getByText("First Note").click()
    await expect(page.getByText("Saved")).toBeVisible()

    // Type in the editor
    const editor = page.locator(".tiptap")
    await editor.click()
    await page.keyboard.press("End")
    await page.keyboard.type("Manual save test.")

    await expect(page.getByText("Unsaved")).toBeVisible()

    // Trigger Ctrl+S (Meta+S on macOS)
    await page.keyboard.press("Meta+s")

    // Should save
    await expect(page.getByText("Saved")).toBeVisible({ timeout: 3000 })
  })

  test("searches notes", async ({ page }) => {
    // Set up with search results returning only the first fixture note
    await setupTauriFromFixtures(page, { searchResults: [FIXTURE_NOTES[0]] })
    await page.goto("/")
    await page.getByRole("button", { name: "Open Folder" }).click()
    await expect(page.getByText("First Note")).toBeVisible()
    await expect(page.getByText("Second Note")).toBeVisible()

    // Type in search box and press Enter
    const searchInput = page.getByPlaceholder("Search...")
    await searchInput.fill("first")
    await searchInput.press("Enter")

    // Only the matching note should be visible; second note should be hidden
    await expect(page.getByText("First Note")).toBeVisible()
    // The sidebar list should only show search results
    const sidebarItems = page.locator("aside > div:nth-child(2) > div")
    await expect(sidebarItems).toHaveCount(1)

    // Clear search
    await page.getByRole("button", { name: "✕" }).click()

    // Both notes should be visible again
    await expect(page.getByText("First Note")).toBeVisible()
    await expect(page.getByText("Second Note")).toBeVisible()
  })

  test("shows + New Note button when workspace is open", async ({ page }) => {
    await page.goto("/")
    await page.getByRole("button", { name: "Open Folder" }).click()
    await expect(page.getByText("test-workspace")).toBeVisible()

    // New Note button should be visible
    await expect(page.getByRole("button", { name: "+ New Note" })).toBeVisible()
  })
})
