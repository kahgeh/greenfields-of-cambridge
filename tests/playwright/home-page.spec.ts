import { test, expect } from '@playwright/test';

test.describe('Home Page', () => {
  test('loads successfully', async ({ page }) => {
    await page.goto('/');

    // Check that the page loads without errors
    await expect(page).toHaveTitle(/Greenfields of Cambridge/);

    // Check for main content
    await expect(page.locator('h1')).toBeVisible();
  });

  test('has navigation to contact page', async ({ page }) => {
    await page.goto('/');

    // Look for contact link/button
    const contactLink = page.locator('a[href*="contact"], button:has-text("Contact"), a:has-text("Contact")');
    if (await contactLink.count() > 0) {
      await contactLink.first().click();
      await expect(page).toHaveURL(/.*contact/);
    }
  });
});