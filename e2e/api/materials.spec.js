// @ts-check
import { test, expect } from '@playwright/test';

const API_BASE = 'http://127.0.0.1:3000';

test.describe('Materials API', () => {
  test('should list available materials', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/materials`);
    expect(response.ok()).toBeTruthy();

    const materials = await response.json();
    expect(Array.isArray(materials)).toBeTruthy();
    expect(materials.length).toBeGreaterThan(0);

    // Check material structure (public endpoint)
    const material = materials[0];
    expect(material).toHaveProperty('id');
    expect(material).toHaveProperty('name');
    expect(material).toHaveProperty('price_per_cm3');
    expect(material).toHaveProperty('color');
    expect(material).toHaveProperty('properties');
  });

  test('materials should have valid prices', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/materials`);
    const materials = await response.json();

    for (const material of materials) {
      expect(material.price_per_cm3).toBeGreaterThan(0);
      expect(typeof material.price_per_cm3).toBe('number');
    }
  });

  test('should have PLA material available', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/materials`);
    const materials = await response.json();

    const pla = materials.find((m) => m.name.includes('PLA'));
    expect(pla).toBeTruthy();
    expect(pla.id).toContain('pla');
  });

  test('should have multiple material colors', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/materials`);
    const materials = await response.json();

    const colors = new Set(materials.map((m) => m.color).filter(Boolean));
    expect(colors.size).toBeGreaterThan(1);
  });
});
