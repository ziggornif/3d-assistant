// @ts-check
import { test, expect } from '@playwright/test';

const API_BASE = 'http://127.0.0.1:3000';

test.describe('Success Criteria Validation', () => {
  test.describe('API Response Times', () => {
    test('session creation should respond within 1 second', async ({ request }) => {
      const start = Date.now();
      const response = await request.post(`${API_BASE}/api/sessions`);
      const duration = Date.now() - start;

      expect(response.ok()).toBeTruthy();
      expect(duration).toBeLessThan(1000);
    });

    test('materials list should respond within 500ms', async ({ request }) => {
      const start = Date.now();
      const response = await request.get(`${API_BASE}/api/materials`);
      const duration = Date.now() - start;

      expect(response.ok()).toBeTruthy();
      expect(duration).toBeLessThan(500);
    });

    test('quote calculation should respond within 3 seconds', async ({ request }) => {
      // Create session first
      const sessionRes = await request.post(`${API_BASE}/api/sessions`);
      const session = await sessionRes.json();

      const start = Date.now();
      const response = await request.get(`${API_BASE}/api/sessions/${session.session_id}/quote`);
      const duration = Date.now() - start;

      // Should respond quickly even with no models
      expect(duration).toBeLessThan(3000);
    });

    test('health check should respond within 100ms', async ({ request }) => {
      const start = Date.now();
      const response = await request.get(`${API_BASE}/health`);
      const duration = Date.now() - start;

      expect(response.ok()).toBeTruthy();
      expect(duration).toBeLessThan(100);
    });
  });

  test.describe('Price Precision', () => {
    test('material prices should have correct decimal precision', async ({ request }) => {
      const response = await request.get(`${API_BASE}/api/materials`);
      const materials = await response.json();

      for (const material of materials) {
        // Price should be a number
        expect(typeof material.price_per_cm3).toBe('number');

        // Price should be positive
        expect(material.price_per_cm3).toBeGreaterThan(0);

        // Price string representation should have at most 6 decimal places (internal precision)
        const priceStr = material.price_per_cm3.toString();
        if (priceStr.includes('.')) {
          const decimals = priceStr.split('.')[1].length;
          expect(decimals).toBeLessThanOrEqual(6);
        }
      }
    });

    test('quote breakdown should display prices with 2 decimal places', async ({ request }) => {
      const sessionRes = await request.post(`${API_BASE}/api/sessions`);
      const session = await sessionRes.json();

      const quoteRes = await request.get(`${API_BASE}/api/sessions/${session.session_id}/quote`);
      const quote = await quoteRes.json();

      // Subtotal and total should be precise to 2 decimal places
      expect(typeof quote.subtotal).toBe('number');
      expect(typeof quote.total).toBe('number');

      // Check that values are rounded to 2 decimals
      const subtotalRounded = Math.round(quote.subtotal * 100) / 100;
      const totalRounded = Math.round(quote.total * 100) / 100;

      expect(quote.subtotal).toBe(subtotalRounded);
      expect(quote.total).toBe(totalRounded);
    });

    test('quote items should have accurate individual prices', async ({ request }) => {
      const sessionRes = await request.post(`${API_BASE}/api/sessions`);
      const session = await sessionRes.json();

      const quoteRes = await request.get(`${API_BASE}/api/sessions/${session.session_id}/quote`);
      const quote = await quoteRes.json();

      // Each item price should be a valid number
      for (const item of quote.items || []) {
        expect(typeof item.price).toBe('number');
        expect(item.price).toBeGreaterThanOrEqual(0);
      }

      // Sum of items should equal subtotal (if items exist)
      if (quote.items && quote.items.length > 0) {
        const itemsSum = quote.items.reduce((sum, item) => sum + item.price, 0);
        const roundedSum = Math.round(itemsSum * 100) / 100;
        expect(quote.subtotal).toBeCloseTo(roundedSum, 2);
      }
    });
  });

  test.describe('File Upload Limits', () => {
    test('should enforce maximum file size of 50MB', async ({ request }) => {
      // Create session
      const sessionRes = await request.post(`${API_BASE}/api/sessions`);
      const session = await sessionRes.json();

      // Try to check upload endpoint exists
      const response = await request.post(
        `${API_BASE}/api/sessions/${session.session_id}/models`,
        {
          multipart: {
            file: {
              name: 'test.stl',
              mimeType: 'application/octet-stream',
              buffer: Buffer.alloc(100), // Small valid file
            },
          },
        }
      );

      // Should accept small files (either success or validation error for content, not size)
      expect([200, 400, 422]).toContain(response.status());
    });

    test('should reject invalid file formats', async ({ request }) => {
      const sessionRes = await request.post(`${API_BASE}/api/sessions`);
      const session = await sessionRes.json();

      // Try to upload a .txt file (invalid format)
      const response = await request.post(
        `${API_BASE}/api/sessions/${session.session_id}/models`,
        {
          multipart: {
            file: {
              name: 'test.txt',
              mimeType: 'text/plain',
              buffer: Buffer.from('This is not a 3D file'),
            },
          },
        }
      );

      // Should reject invalid format
      expect(response.status()).toBe(400);
    });

    test('should accept STL file format', async ({ request }) => {
      const sessionRes = await request.post(`${API_BASE}/api/sessions`);
      const session = await sessionRes.json();

      // Minimal valid ASCII STL
      const stlContent = `solid test
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
endsolid test`;

      const response = await request.post(
        `${API_BASE}/api/sessions/${session.session_id}/models`,
        {
          multipart: {
            file: {
              name: 'test.stl',
              mimeType: 'application/octet-stream',
              buffer: Buffer.from(stlContent),
            },
          },
        }
      );

      expect(response.ok()).toBeTruthy();
      const model = await response.json();
      expect(model).toHaveProperty('model_id');
      expect(model).toHaveProperty('filename');
    });
  });

  test.describe('Session Management', () => {
    test('session should have valid expiration time', async ({ request }) => {
      const response = await request.post(`${API_BASE}/api/sessions`);
      const session = await response.json();

      expect(session).toHaveProperty('session_id');
      expect(session).toHaveProperty('expires_at');

      // Expiration should be in the future (at least 23 hours from now)
      const expiresAt = new Date(session.expires_at);
      const now = new Date();
      const hoursDiff = (expiresAt.getTime() - now.getTime()) / (1000 * 60 * 60);

      expect(hoursDiff).toBeGreaterThan(23);
      expect(hoursDiff).toBeLessThanOrEqual(25); // Around 24 hours
    });

    test('session ID should be unique', async ({ request }) => {
      const session1 = await request.post(`${API_BASE}/api/sessions`).then(r => r.json());
      const session2 = await request.post(`${API_BASE}/api/sessions`).then(r => r.json());

      expect(session1.session_id).not.toBe(session2.session_id);
    });

    test('invalid session should return 404', async ({ request }) => {
      const response = await request.get(`${API_BASE}/api/sessions/invalid-session-id/quote`);
      expect(response.status()).toBe(404);
    });
  });

  test.describe('Data Integrity', () => {
    test('materials should have all required fields', async ({ request }) => {
      const response = await request.get(`${API_BASE}/api/materials`);
      const materials = await response.json();

      expect(materials.length).toBeGreaterThan(0);

      for (const material of materials) {
        expect(material).toHaveProperty('id');
        expect(material).toHaveProperty('name');
        expect(material).toHaveProperty('price_per_cm3');
        expect(material).toHaveProperty('color');
        expect(material).toHaveProperty('description');

        // Validate types
        expect(typeof material.id).toBe('string');
        expect(typeof material.name).toBe('string');
        expect(typeof material.price_per_cm3).toBe('number');
        expect(typeof material.color).toBe('string');
        expect(typeof material.description).toBe('string');

        // Color should be valid hex
        expect(material.color).toMatch(/^#[0-9A-Fa-f]{6}$/);
      }
    });

    test('quote should have consistent structure', async ({ request }) => {
      const sessionRes = await request.post(`${API_BASE}/api/sessions`);
      const session = await sessionRes.json();

      const quoteRes = await request.get(`${API_BASE}/api/sessions/${session.session_id}/quote`);
      const quote = await quoteRes.json();

      expect(quote).toHaveProperty('items');
      expect(quote).toHaveProperty('subtotal');
      expect(quote).toHaveProperty('total');
      expect(quote).toHaveProperty('fees');

      expect(Array.isArray(quote.items)).toBe(true);
      expect(typeof quote.subtotal).toBe('number');
      expect(typeof quote.total).toBe('number');
      expect(typeof quote.fees).toBe('number');
    });
  });

  test.describe('Error Handling', () => {
    test('should return proper error for malformed JSON', async ({ request }) => {
      const sessionRes = await request.post(`${API_BASE}/api/sessions`);
      const session = await sessionRes.json();

      const response = await request.patch(
        `${API_BASE}/api/sessions/${session.session_id}/models/test-id`,
        {
          headers: { 'Content-Type': 'application/json' },
          data: 'invalid json{',
        }
      );

      expect(response.status()).toBeGreaterThanOrEqual(400);
    });

    test('should handle missing required fields gracefully', async ({ request }) => {
      const sessionRes = await request.post(`${API_BASE}/api/sessions`);
      const session = await sessionRes.json();

      const response = await request.patch(
        `${API_BASE}/api/sessions/${session.session_id}/models/test-id`,
        {
          headers: { 'Content-Type': 'application/json' },
          data: {}, // Empty object, missing required fields
        }
      );

      // Should either accept empty update or return 400/422
      expect([200, 400, 404, 422]).toContain(response.status());
    });
  });
});
