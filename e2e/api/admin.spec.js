// @ts-check
import { test, expect } from '@playwright/test';

const API_BASE = 'http://127.0.0.1:3000';
const ADMIN_TOKEN = 'admin-secret-token-2025';

test.describe('Admin Materials API', () => {
  test('should require authentication for admin endpoints', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/admin/materials`);
    expect(response.status()).toBe(401);
  });

  test('should reject invalid token', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/admin/materials`, {
      headers: { Authorization: 'Bearer invalid-token' },
    });
    expect(response.status()).toBe(401);
  });

  test('should list all materials including inactive', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/admin/materials`, {
      headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
    });
    expect(response.ok()).toBeTruthy();

    const materials = await response.json();
    expect(Array.isArray(materials)).toBeTruthy();
    expect(materials.length).toBeGreaterThan(0);

    // Verify admin-specific fields
    const material = materials[0];
    expect(material).toHaveProperty('id');
    expect(material).toHaveProperty('name');
    expect(material).toHaveProperty('price_per_cm3');
    expect(material).toHaveProperty('active');
    expect(material).toHaveProperty('created_at');
    expect(material).toHaveProperty('updated_at');
  });

  test('should create new material', async ({ request }) => {
    const newMaterial = {
      name: `Test Material ${Date.now()}`,
      service_type_id: '3d_printing',
      price_per_cm3: 0.123,
      color: '#FF5500',
      description: 'Test material for API testing',
    };

    const response = await request.post(`${API_BASE}/api/admin/materials`, {
      headers: {
        Authorization: `Bearer ${ADMIN_TOKEN}`,
        'Content-Type': 'application/json',
      },
      data: newMaterial,
    });

    expect(response.ok()).toBeTruthy();
    const created = await response.json();

    expect(created).toHaveProperty('id');
    expect(created.name).toBe(newMaterial.name);
    expect(created.price_per_cm3).toBe(newMaterial.price_per_cm3);
    expect(created.color).toBe(newMaterial.color);
    expect(created.active).toBe(true);
  });

  test('should update material price', async ({ request }) => {
    // Get existing material
    const listRes = await request.get(`${API_BASE}/api/admin/materials`, {
      headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
    });
    const materials = await listRes.json();
    const material = materials[0];
    const originalPrice = material.price_per_cm3;
    const newPrice = originalPrice + 0.001;

    const response = await request.patch(`${API_BASE}/api/admin/materials/${material.id}`, {
      headers: {
        Authorization: `Bearer ${ADMIN_TOKEN}`,
        'Content-Type': 'application/json',
      },
      data: { price_per_cm3: newPrice },
    });

    expect(response.ok()).toBeTruthy();
    const updated = await response.json();
    expect(updated.price_per_cm3).toBe(newPrice);

    // Restore original price
    await request.patch(`${API_BASE}/api/admin/materials/${material.id}`, {
      headers: {
        Authorization: `Bearer ${ADMIN_TOKEN}`,
        'Content-Type': 'application/json',
      },
      data: { price_per_cm3: originalPrice },
    });
  });

  test('should toggle material active status', async ({ request }) => {
    const listRes = await request.get(`${API_BASE}/api/admin/materials`, {
      headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
    });
    const materials = await listRes.json();
    const material = materials[0];
    const originalStatus = material.active;

    // Toggle off
    const response = await request.patch(`${API_BASE}/api/admin/materials/${material.id}`, {
      headers: {
        Authorization: `Bearer ${ADMIN_TOKEN}`,
        'Content-Type': 'application/json',
      },
      data: { active: !originalStatus },
    });

    expect(response.ok()).toBeTruthy();
    const updated = await response.json();
    expect(updated.active).toBe(!originalStatus);

    // Restore original status
    await request.patch(`${API_BASE}/api/admin/materials/${material.id}`, {
      headers: {
        Authorization: `Bearer ${ADMIN_TOKEN}`,
        'Content-Type': 'application/json',
      },
      data: { active: originalStatus },
    });
  });

  test('should return 404 for non-existent material', async ({ request }) => {
    const response = await request.patch(`${API_BASE}/api/admin/materials/non-existent-id`, {
      headers: {
        Authorization: `Bearer ${ADMIN_TOKEN}`,
        'Content-Type': 'application/json',
      },
      data: { price_per_cm3: 1.0 },
    });

    expect(response.status()).toBe(404);
  });
});

test.describe('Admin Pricing History API', () => {
  test('should get pricing history', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/admin/pricing-history`, {
      headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
    });

    expect(response.ok()).toBeTruthy();
    const history = await response.json();
    expect(Array.isArray(history)).toBeTruthy();

    if (history.length > 0) {
      const entry = history[0];
      expect(entry).toHaveProperty('id');
      expect(entry).toHaveProperty('material_id');
      expect(entry).toHaveProperty('material_name');
      expect(entry).toHaveProperty('new_price');
      expect(entry).toHaveProperty('changed_at');
    }
  });

  test('pricing history should track price changes', async ({ request }) => {
    // Get a material and change its price
    const listRes = await request.get(`${API_BASE}/api/admin/materials`, {
      headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
    });
    const materials = await listRes.json();
    const material = materials[0];
    const originalPrice = material.price_per_cm3;
    const newPrice = originalPrice + 0.005;

    // Update price
    await request.patch(`${API_BASE}/api/admin/materials/${material.id}`, {
      headers: {
        Authorization: `Bearer ${ADMIN_TOKEN}`,
        'Content-Type': 'application/json',
      },
      data: { price_per_cm3: newPrice },
    });

    // Check history
    const historyRes = await request.get(`${API_BASE}/api/admin/pricing-history`, {
      headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
    });
    const history = await historyRes.json();

    // Find our specific change entry (may not be first due to parallel tests)
    const ourEntry = history.find(
      entry =>
        entry.material_id === material.id &&
        Math.abs(entry.new_price - newPrice) < 0.0001
    );
    expect(ourEntry).toBeTruthy();
    expect(ourEntry.material_id).toBe(material.id);
    expect(ourEntry.new_price).toBeCloseTo(newPrice, 4);

    // Restore original price
    await request.patch(`${API_BASE}/api/admin/materials/${material.id}`, {
      headers: {
        Authorization: `Bearer ${ADMIN_TOKEN}`,
        'Content-Type': 'application/json',
      },
      data: { price_per_cm3: originalPrice },
    });
  });
});

test.describe('Admin Session Cleanup API', () => {
  test('should require authentication for cleanup endpoint', async ({ request }) => {
    const response = await request.post(`${API_BASE}/api/admin/cleanup`);
    expect(response.status()).toBe(401);
  });

  test('should cleanup expired sessions', async ({ request }) => {
    const response = await request.post(`${API_BASE}/api/admin/cleanup`, {
      headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
    });

    expect(response.ok()).toBeTruthy();
    const result = await response.json();

    // Verify cleanup result structure
    expect(result).toHaveProperty('sessions_deleted');
    expect(result).toHaveProperty('directories_deleted');
    expect(result).toHaveProperty('errors');
    expect(typeof result.sessions_deleted).toBe('number');
    expect(typeof result.directories_deleted).toBe('number');
    expect(Array.isArray(result.errors)).toBeTruthy();
  });

  test('cleanup should handle empty database gracefully', async ({ request }) => {
    // Multiple cleanups should be idempotent
    const response1 = await request.post(`${API_BASE}/api/admin/cleanup`, {
      headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
    });
    const response2 = await request.post(`${API_BASE}/api/admin/cleanup`, {
      headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
    });

    expect(response1.ok()).toBeTruthy();
    expect(response2.ok()).toBeTruthy();

    const result1 = await response1.json();
    const result2 = await response2.json();

    // Both should succeed with no errors
    expect(result1.errors).toHaveLength(0);
    expect(result2.errors).toHaveLength(0);
  });

  test('cleanup should only delete expired sessions', async ({ request }) => {
    // Create a new session (not expired)
    const sessionRes = await request.post(`${API_BASE}/api/sessions`);
    expect(sessionRes.ok()).toBeTruthy();
    const session = await sessionRes.json();

    // Run cleanup
    const cleanupRes = await request.post(`${API_BASE}/api/admin/cleanup`, {
      headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
    });
    const cleanupResult = await cleanupRes.json();

    // The new session should still exist (not expired)
    const checkRes = await request.get(`${API_BASE}/api/sessions/${session.session_id}/quote`);
    // Session should still be valid (returns 200 or specific status for no models)
    expect([200, 400].includes(checkRes.status())).toBeTruthy();
  });
});
