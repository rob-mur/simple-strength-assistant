const { chromium } = require('playwright');
(async () => {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();
  page.on('console', msg => console.log('BROWSER CONSOLE:', msg.text()));
  page.on('pageerror', error => console.error('BROWSER ERROR:', error));
  
  await page.goto('http://localhost:8080');
  await page.evaluate(() => localStorage.clear());
  // Setup steps
  await page.click('text="Library"'); // rough guess
  await page.waitForTimeout(1000);
  
  await browser.close();
})();
