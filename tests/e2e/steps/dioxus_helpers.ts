import type { Page } from "@playwright/test";

/**
 * Sets the value of a Dioxus WASM controlled input and triggers the oninput signal.
 *
 * Dioxus WASM controlled inputs cannot be filled with page.fill() or
 * pressSequentially() because those methods don't reliably trigger the Rust
 * oninput handler. The fix is to use the native HTMLInputElement value setter
 * and dispatch a bubbling InputEvent, which Dioxus's event delegation picks up.
 */
export async function setDioxusInput(
  page: Page,
  selector: string,
  value: string,
): Promise<void> {
  await page.locator(selector).click();
  await page.evaluate(
    ([sel, val]: [string, string]) => {
      const input = document.querySelector(sel) as HTMLInputElement;
      if (!input) throw new Error(`Input not found: ${sel}`);
      input.focus();
      const nativeSetter = Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set;
      nativeSetter?.call(input, val);
      input.dispatchEvent(
        new InputEvent("input", {
          bubbles: true,
          cancelable: true,
          data: val,
          inputType: "insertText",
        }),
      );
    },
    [selector, value],
  );
  // Give Dioxus time to process the event and update the signal
  await page.waitForTimeout(100);
}
