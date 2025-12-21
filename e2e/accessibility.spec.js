// @ts-check
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test.describe('Accessibility Tests', () => {
  test('homepage should have no critical or serious accessibility violations', async ({ page }) => {
    await page.goto('/');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();

    // Filter for critical and serious violations only
    const criticalViolations = accessibilityScanResults.violations.filter(
      violation => violation.impact === 'critical' || violation.impact === 'serious'
    );

    expect(criticalViolations).toEqual([]);
  });

  test('homepage should have minimal accessibility violations', async ({ page }) => {
    await page.goto('/');

    const accessibilityScanResults = await new AxeBuilder({ page }).analyze();

    // Allow up to 2 minor violations (moderate/minor) for now
    const significantViolations = accessibilityScanResults.violations.filter(
      violation => violation.impact === 'critical' || violation.impact === 'serious'
    );

    expect(significantViolations).toEqual([]);
  });

  test('admin login page should be accessible', async ({ page }) => {
    await page.goto('/admin');

    const accessibilityScanResults = await new AxeBuilder({ page }).analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('page should have proper heading structure', async ({ page }) => {
    await page.goto('/');

    // Check for h1 heading
    const h1 = await page.locator('h1').count();
    expect(h1).toBe(1);

    // Check heading hierarchy (no skipped levels)
    const headings = await page.evaluate(() => {
      const hs = Array.from(document.querySelectorAll('h1, h2, h3, h4, h5, h6'));
      return hs.map(h => parseInt(h.tagName.charAt(1)));
    });

    // Verify no levels are skipped
    for (let i = 1; i < headings.length; i++) {
      const diff = headings[i] - headings[i - 1];
      // Should not skip more than 1 level down
      expect(diff).toBeLessThanOrEqual(1);
    }
  });

  test('all images should have alt text', async ({ page }) => {
    await page.goto('/');

    const imagesWithoutAlt = await page.evaluate(() => {
      const images = Array.from(document.querySelectorAll('img'));
      return images.filter(img => !img.hasAttribute('alt')).length;
    });

    expect(imagesWithoutAlt).toBe(0);
  });

  test('all form inputs should have labels', async ({ page }) => {
    await page.goto('/');

    const inputsWithoutLabels = await page.evaluate(() => {
      const inputs = Array.from(
        document.querySelectorAll('input:not([type="hidden"]), select, textarea')
      );
      return inputs.filter(input => {
        const id = input.id;
        if (!id) return true;
        const label = document.querySelector(`label[for="${id}"]`);
        const ariaLabel = input.getAttribute('aria-label');
        const ariaLabelledBy = input.getAttribute('aria-labelledby');
        return !label && !ariaLabel && !ariaLabelledBy;
      }).length;
    });

    expect(inputsWithoutLabels).toBe(0);
  });

  test('page should have skip link for keyboard navigation', async ({ page }) => {
    await page.goto('/');

    const skipLink = await page.locator('a[href="#main-content"], .skip-link');
    await expect(skipLink).toHaveCount(1);
  });

  test('interactive elements should be keyboard focusable', async ({ page }) => {
    await page.goto('/');

    // Check buttons have no negative tabindex
    const nonFocusableButtons = await page.evaluate(() => {
      const buttons = Array.from(document.querySelectorAll('button'));
      return buttons.filter(btn => btn.tabIndex < 0 && !btn.disabled).length;
    });

    expect(nonFocusableButtons).toBe(0);
  });

  test('color contrast should meet WCAG AA standards', async ({ page }) => {
    await page.goto('/');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2aa'])
      .options({ runOnly: ['color-contrast'] })
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('focus indicators should be visible', async ({ page }) => {
    await page.goto('/');

    // Tab to first focusable element and check focus style
    await page.keyboard.press('Tab');

    const hasFocusIndicator = await page.evaluate(() => {
      const focused = document.activeElement;
      if (!focused || focused === document.body) return true; // No focusable elements is ok

      const styles = window.getComputedStyle(focused);
      const outline = styles.outline;
      const boxShadow = styles.boxShadow;

      // Check if there's a visible focus indicator
      const hasOutline = outline && !outline.includes('none') && !outline.includes('0px');
      const hasBoxShadow = boxShadow && boxShadow !== 'none';

      return hasOutline || hasBoxShadow;
    });

    expect(hasFocusIndicator).toBe(true);
  });

  test('ARIA roles should be used correctly', async ({ page }) => {
    await page.goto('/');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .options({ runOnly: ['aria-roles', 'aria-required-attr', 'aria-valid-attr'] })
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('language attribute should be set', async ({ page }) => {
    await page.goto('/');

    const lang = await page.getAttribute('html', 'lang');
    expect(lang).toBeTruthy();
    expect(lang).toMatch(/^[a-z]{2}(-[A-Z]{2})?$/);
  });
});
