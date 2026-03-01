import { test as base } from 'playwright-bdd';
import { expect } from '@playwright/test';
import { createBdd } from 'playwright-bdd';

export const test = base.extend<{}>({});
export const { Given, When, Then } = createBdd(test);
export { expect };
