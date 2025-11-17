// @ts-check
import { test, expect } from '@playwright/test';

const API_BASE = 'http://127.0.0.1:3000';

test.describe('Session Management API', () => {
  test('should create a new session', async ({ request }) => {
    const response = await request.post(`${API_BASE}/api/sessions`);
    expect(response.ok()).toBeTruthy();

    const data = await response.json();
    expect(data).toHaveProperty('session_id');
    expect(data).toHaveProperty('expires_at');
    expect(data.session_id).toBeTruthy();
    // ULID format: 26 characters, uppercase alphanumeric
    expect(data.session_id).toMatch(/^[0-9A-Z]{26}$/);
  });

  test('should return 404 for non-existent session when getting quote', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/sessions/non-existent-id/quote`);
    expect(response.status()).toBe(404);
  });

  test('session should have valid expiration time', async ({ request }) => {
    const response = await request.post(`${API_BASE}/api/sessions`);
    const data = await response.json();

    const expiresAt = new Date(data.expires_at);
    const now = new Date();
    const hoursDiff = (expiresAt.getTime() - now.getTime()) / (1000 * 60 * 60);

    // Should expire in ~24 hours
    expect(hoursDiff).toBeGreaterThan(23);
    expect(hoursDiff).toBeLessThan(25);
  });
});

test.describe('Health Check API', () => {
  test('health check should return OK', async ({ request }) => {
    const response = await request.get(`${API_BASE}/health`);
    expect(response.ok()).toBeTruthy();
    const text = await response.text();
    expect(text).toBe('OK');
  });
});
