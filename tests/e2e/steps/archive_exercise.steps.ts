import { Given, When, Then, expect } from "./fixtures";

When(
  "the user taps the archive button on the detail view",
  async ({ page }) => {
    await page.getByTestId("archive-button").click();
  },
);

Then(
  "the archive dialog is shown for {string}",
  async ({ page }, name: string) => {
    const dialog = page.getByTestId("confirmation-dialog");
    await expect(dialog).toBeVisible();
    const title = page.getByTestId("confirmation-dialog-title");
    await expect(title).toContainText(`Archive ${name}`);
  },
);

Then("the archive dialog shows {string}", async ({ page }, text: string) => {
  const body = page.getByTestId("confirmation-dialog-body");
  await expect(body).toContainText(text);
});

When("the user confirms the archive dialog", async ({ page }) => {
  await page.getByTestId("confirmation-dialog-confirm").click();
  // Wait for dialog to disappear
  await expect(page.getByTestId("confirmation-dialog")).not.toBeVisible();
});

Then(
  "{string} is not in the active exercise list",
  async ({ page }, name: string) => {
    await expect(
      page.locator('[data-testid="library-view"] h3', {
        hasText: name.toUpperCase(),
      }),
    ).not.toBeVisible();
  },
);

Then(
  "{string} appears in the archived list with an ARCHIVED badge",
  async ({ page }, name: string) => {
    const card = page.locator('[data-testid="library-view"] div.card', {
      hasText: name.toUpperCase(),
    });
    await expect(card).toBeVisible();
    const badge = card.getByTestId("archived-badge");
    await expect(badge).toBeVisible();
    await expect(badge).toHaveText("ARCHIVED");
  },
);

Then(
  "the user sees the Unarchive button instead of START",
  async ({ page }) => {
    await expect(page.getByTestId("unarchive-button")).toBeVisible();
    await expect(page.getByTestId("start-button")).not.toBeVisible();
  },
);

When("the user taps the Unarchive button", async ({ page }) => {
  await page.getByTestId("unarchive-button").click();
});

Then(
  "{string} is in the active exercise list",
  async ({ page }, name: string) => {
    // Toggle back to active view first (toggle is currently on archived)
    const toggle = page.getByTestId("show-archived-toggle");
    if (await toggle.isChecked()) {
      await toggle.click();
    }
    await expect(
      page.locator('[data-testid="library-view"] h3', {
        hasText: name.toUpperCase(),
      }),
    ).toBeVisible();
  },
);

// ── Plan cascade steps (Issue #193) ──────────────────────────────────────────

// Globals to hold plan IDs created by the steps below so assertions can
// reference them across step boundaries.
let _soloPlanId: string | null = null;
let _sharedPlanId: string | null = null;

Given(
  "a future plan exists with only {string}",
  async ({ page }, exerciseName: string) => {
    const planId = await page.evaluate(async (name: string) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const exec = (window as any).__dbExecuteQuery as (
        sql: string,
        params: unknown[],
      ) => Promise<unknown>;
      if (!exec) throw new Error("__dbExecuteQuery not available");

      // Look up exercise id by name.
      const exRows = (await exec(
        "SELECT uuid FROM exercises WHERE name = ? AND deleted_at IS NULL",
        [name],
      )) as Array<{ uuid: string }>;
      if (!exRows.length) throw new Error(`Exercise '${name}' not found`);
      const exerciseId = exRows[0].uuid;

      // Create a future plan (no started_at).
      const planId = "solo-" + Math.random().toString(36).slice(2) + Date.now();
      const now = Date.now();
      await exec("INSERT INTO workout_plans (id, updated_at) VALUES (?, ?)", [
        planId,
        now,
      ]);
      const slotId = "slot-" + Math.random().toString(36).slice(2);
      await exec(
        "INSERT INTO workout_plan_exercises (id, plan_id, exercise_id, planned_sets, position, updated_at) VALUES (?, ?, ?, 3, 0, ?)",
        [slotId, planId, exerciseId, now],
      );
      return planId;
    }, exerciseName);
    _soloPlanId = planId as string;
  },
);

Given(
  "a future plan exists with {string} and {string}",
  async ({ page }, nameA: string, nameB: string) => {
    const planId = await page.evaluate(
      async ([a, b]: string[]) => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const exec = (window as any).__dbExecuteQuery as (
          sql: string,
          params: unknown[],
        ) => Promise<unknown>;
        if (!exec) throw new Error("__dbExecuteQuery not available");

        const getEx = async (n: string) => {
          const rows = (await exec(
            "SELECT uuid FROM exercises WHERE name = ? AND deleted_at IS NULL",
            [n],
          )) as Array<{ uuid: string }>;
          if (!rows.length) throw new Error(`Exercise '${n}' not found`);
          return rows[0].uuid;
        };

        const eidA = await getEx(a);
        const eidB = await getEx(b);

        const planId =
          "shared-" + Math.random().toString(36).slice(2) + Date.now();
        const now = Date.now();
        await exec("INSERT INTO workout_plans (id, updated_at) VALUES (?, ?)", [
          planId,
          now,
        ]);
        await exec(
          "INSERT INTO workout_plan_exercises (id, plan_id, exercise_id, planned_sets, position, updated_at) VALUES (?, ?, ?, 3, 0, ?)",
          ["slot-a-" + Math.random().toString(36).slice(2), planId, eidA, now],
        );
        await exec(
          "INSERT INTO workout_plan_exercises (id, plan_id, exercise_id, planned_sets, position, updated_at) VALUES (?, ?, ?, 3, 1, ?)",
          ["slot-b-" + Math.random().toString(36).slice(2), planId, eidB, now],
        );
        return planId;
      },
      [nameA, nameB],
    );
    _sharedPlanId = planId as string;
  },
);

Then(
  "the solo future plan for {string} is deleted",
  async ({ page }, _exerciseName: string) => {
    const planId = _soloPlanId;
    if (!planId) throw new Error("No solo plan ID recorded");
    const deleted = await page.evaluate(async (id: string) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const exec = (window as any).__dbExecuteQuery as (
        sql: string,
        params: unknown[],
      ) => Promise<unknown>;
      const rows = (await exec(
        "SELECT deleted_at FROM workout_plans WHERE id = ?",
        [id],
      )) as Array<{ deleted_at: number | null }>;
      return rows.length > 0 && rows[0].deleted_at != null;
    }, planId);
    expect(deleted).toBe(true);
  },
);

Then("the shared future plan still exists", async ({ page }) => {
  const planId = _sharedPlanId;
  if (!planId) throw new Error("No shared plan ID recorded");
  const alive = await page.evaluate(async (id: string) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const exec = (window as any).__dbExecuteQuery as (
      sql: string,
      params: unknown[],
    ) => Promise<unknown>;
    const rows = (await exec(
      "SELECT deleted_at FROM workout_plans WHERE id = ?",
      [id],
    )) as Array<{ deleted_at: number | null }>;
    return rows.length > 0 && rows[0].deleted_at == null;
  }, planId);
  expect(alive).toBe(true);
});
