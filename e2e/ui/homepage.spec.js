// @ts-check
import { test, expect } from '@playwright/test';

test.describe('Homepage UI', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display the main title', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('Service de Devis Impression 3D');
  });

  test('should have skip link for accessibility', async ({ page }) => {
    const skipLink = page.locator('a.skip-link');
    await expect(skipLink).toHaveAttribute('href', '#main-content');
    await expect(skipLink).toContainText('Aller au contenu principal');
  });

  test('should have file uploader component', async ({ page }) => {
    const fileUploader = page.locator('file-uploader#file-uploader');
    await expect(fileUploader).toBeVisible();
  });

  test('should display upload instructions', async ({ page }) => {
    const instructions = page.locator('#upload-instructions');
    await expect(instructions).toBeVisible();
    await expect(instructions).toContainText('STL, 3MF');
    await expect(instructions).toContainText('50 MB');
  });

  test('should have upload section heading', async ({ page }) => {
    const heading = page.locator('#upload-heading');
    await expect(heading).toContainText('Télécharger vos fichiers');
  });

  test('should have hidden models section initially', async ({ page }) => {
    const modelsSection = page.locator('#models-section');
    await expect(modelsSection).toBeHidden();
  });

  test('should have hidden quote section initially', async ({ page }) => {
    const quoteSection = page.locator('#quote-section');
    await expect(quoteSection).toBeHidden();
  });

  test('should have proper ARIA roles for accessibility', async ({ page }) => {
    await expect(page.locator('header')).toHaveAttribute('role', 'banner');
    await expect(page.locator('main')).toHaveAttribute('role', 'main');
    await expect(page.locator('footer')).toHaveAttribute('role', 'contentinfo');
  });

  test('should initialize session on page load', async ({ page }) => {
    // Wait for session initialization by checking the session cookie
    // Use the Playwright cookie API so this works even if the cookie is httpOnly
    const timeout = 10000;
    const start = Date.now();
    // Search all cookies and accept either:
    // - a cookie whose value is JSON with { sessionId }
    // - or a cookie whose value is the sessionId directly (ULID)
    const ulidRe = /^[0-9A-Z]{26}$/;
    let found = null;
    while (Date.now() - start < timeout) {
      const cookies = await page.context().cookies();
      for (const c of cookies) {
        let val = c.value;
        try {
          val = decodeURIComponent(val);
        } catch {
          // ignore
        }

        // Try JSON parse
        try {
          const p = JSON.parse(val);
          if (p && typeof p === 'object' && p.sessionId && ulidRe.test(p.sessionId)) {
            found = { cookie: c, sessionId: p.sessionId };
            break;
          }
        } catch {
          // not JSON, fallthrough
        }

        // Try raw ULID value
        if (ulidRe.test(val)) {
          found = { cookie: c, sessionId: val };
          break;
        }
      }

      if (found) break;
      await page.waitForTimeout(200);
    }

    expect(found).toBeTruthy();
    if (!found) throw new Error('No cookie containing a valid sessionId ULID was found');

    const parsedSessionId = found.sessionId;
    expect(parsedSessionId).toMatch(ulidRe);
  });

  test('should have footer with copyright', async ({ page }) => {
    const footer = page.locator('footer');
    await expect(footer).toContainText('© 2025');
    await expect(footer).toContainText('Service de Devis Impression 3D');
  });
});

test.describe('File Uploader Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display drop zone text', async ({ page }) => {
    const fileUploader = page.locator('file-uploader');
    // Check that the shadow DOM content is rendered
    await expect(fileUploader).toBeVisible();

    // The component should have rendered its internal UI
    // Use internal:has-text to verify shadow DOM content without piercing
    const dropZoneText = page.getByText('Glissez vos fichiers ici');
    await expect(dropZoneText).toBeVisible();
  });

  test('should have file input element', async ({ page }) => {
    const fileUploader = page.locator('file-uploader');
    await expect(fileUploader).toBeVisible();
    // The file input is inside shadow DOM, verify via attributes
    await expect(fileUploader).toHaveAttribute('accepted-formats', 'stl,3mf');
  });

  test('should accept STL and 3MF formats', async ({ page }) => {
    const fileUploader = page.locator('file-uploader');
    await expect(fileUploader).toHaveAttribute('accepted-formats', 'stl,3mf');
  });

  test('should have max size of 50MB', async ({ page }) => {
    const fileUploader = page.locator('file-uploader');
    await expect(fileUploader).toHaveAttribute('max-size-mb', '50');
  });
});

test.describe('Page Structure and Semantics', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should have proper document title', async ({ page }) => {
    await expect(page).toHaveTitle('Devis Impression 3D');
  });

  test('should have meta description', async ({ page }) => {
    const metaDescription = page.locator('meta[name="description"]');
    await expect(metaDescription).toHaveAttribute(
      'content',
      /impression 3D.*STL.*3MF.*devis/i
    );
  });

  test('should have French language set', async ({ page }) => {
    const html = page.locator('html');
    await expect(html).toHaveAttribute('lang', 'fr');
  });

  test('should load Three.js import map', async ({ page }) => {
    const importMap = page.locator('script[type="importmap"]');
    await expect(importMap).toBeAttached();

    const content = await importMap.textContent();
    expect(content).toContain('three');
  });

  test('should have models list with proper role', async ({ page }) => {
    const modelsList = page.locator('#models-list');
    await expect(modelsList).toHaveAttribute('role', 'list');
  });
});
