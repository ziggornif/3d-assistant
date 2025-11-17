// @ts-check
import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import AdmZip from 'adm-zip';

const API_BASE = 'http://127.0.0.1:3000';

test.describe('Quote Generation API', () => {
  test('should get quote with base fee for session without configured models', async ({ request }) => {
    // Create a new session
    const sessionRes = await request.post(`${API_BASE}/api/sessions`);
    const sessionData = await sessionRes.json();
    const sessionId = sessionData.session_id;

    const response = await request.get(`${API_BASE}/api/sessions/${sessionId}/quote`);
    expect(response.ok()).toBeTruthy();

    const quote = await response.json();
    expect(quote.items).toHaveLength(0);
    expect(quote.subtotal).toBe(0);
    // Base fee is applied even with no items
    expect(quote.fees).toBeGreaterThan(0);
    // Total includes fees and minimum order value if applicable
    expect(quote.total).toBeGreaterThanOrEqual(quote.fees);
  });

  test('should upload STL file successfully', async ({ request }) => {
    // Create session
    const sessionRes = await request.post(`${API_BASE}/api/sessions`);
    const { session_id: sessionId } = await sessionRes.json();

    // Create a minimal valid binary STL file
    // Header (80 bytes) + triangle count (4 bytes) = 84 bytes minimum
    const stlHeader = Buffer.alloc(80, 0);
    const triangleCount = Buffer.alloc(4, 0);
    const stlContent = Buffer.concat([stlHeader, triangleCount]);

    const response = await request.post(`${API_BASE}/api/sessions/${sessionId}/models`, {
      multipart: {
        file: {
          name: 'test_empty.stl',
          mimeType: 'application/octet-stream',
          buffer: stlContent,
        },
      },
    });

    expect(response.ok()).toBeTruthy();
    const data = await response.json();

    expect(data).toHaveProperty('model_id');
    expect(data.model_id).toMatch(/^[0-9A-Z]{26}$/); // ULID format
    expect(data).toHaveProperty('filename', 'test_empty.stl');
    expect(data).toHaveProperty('volume_cm3');
    expect(data).toHaveProperty('dimensions_mm');
    expect(data).toHaveProperty('triangle_count', 0);
    expect(data).toHaveProperty('preview_url');
    expect(data.preview_url).toContain(sessionId);
  });

  test('should upload 3MF file successfully', async ({ request }) => {
    // Create session
    const sessionRes = await request.post(`${API_BASE}/api/sessions`);
    const { session_id: sessionId } = await sessionRes.json();

    // Create a valid 3MF file (ZIP with model XML)
    const zip = new AdmZip();

    const modelXml = `<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="10" y="10" z="0"/>
          <vertex x="0" y="10" z="0"/>
          <vertex x="0" y="0" z="10"/>
          <vertex x="10" y="0" z="10"/>
          <vertex x="10" y="10" z="10"/>
          <vertex x="0" y="10" z="10"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="2" v3="1"/>
          <triangle v1="0" v2="3" v3="2"/>
          <triangle v1="4" v2="5" v3="6"/>
          <triangle v1="4" v2="6" v3="7"/>
          <triangle v1="0" v2="1" v3="5"/>
          <triangle v1="0" v2="5" v3="4"/>
          <triangle v1="2" v2="3" v3="7"/>
          <triangle v1="2" v2="7" v3="6"/>
          <triangle v1="0" v2="4" v3="7"/>
          <triangle v1="0" v2="7" v3="3"/>
          <triangle v1="1" v2="2" v3="6"/>
          <triangle v1="1" v2="6" v3="5"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build><item objectid="1"/></build>
</model>`;

    zip.addFile('3D/3dmodel.model', Buffer.from(modelXml));
    const zipBuffer = zip.toBuffer();

    const response = await request.post(`${API_BASE}/api/sessions/${sessionId}/models`, {
      multipart: {
        file: {
          name: 'test_cube.3mf',
          mimeType: 'application/octet-stream',
          buffer: zipBuffer,
        },
      },
    });

    expect(response.ok()).toBeTruthy();
    const data = await response.json();

    expect(data).toHaveProperty('model_id');
    expect(data.model_id).toMatch(/^[0-9A-Z]{26}$/);
    expect(data).toHaveProperty('filename', 'test_cube.3mf');
    expect(data).toHaveProperty('volume_cm3');
    expect(data.volume_cm3).toBeCloseTo(1.0, 1); // 10x10x10mm cube = 1cm³
    expect(data).toHaveProperty('dimensions_mm');
    expect(data.dimensions_mm.x).toBeCloseTo(10, 1);
    expect(data.dimensions_mm.y).toBeCloseTo(10, 1);
    expect(data.dimensions_mm.z).toBeCloseTo(10, 1);
    expect(data).toHaveProperty('triangle_count', 12);
    expect(data).toHaveProperty('preview_url');
    expect(data.preview_url).toContain('.3mf');
  });

  test('should configure model with material and generate quote', async ({ request }) => {
    // Create session
    const sessionRes = await request.post(`${API_BASE}/api/sessions`);
    const { session_id: sessionId } = await sessionRes.json();

    // Get materials
    const materialsRes = await request.get(`${API_BASE}/api/materials`);
    const materials = await materialsRes.json();
    const materialId = materials[0].id;

    // Upload model
    const stlContent = Buffer.concat([Buffer.alloc(80, 0), Buffer.alloc(4, 0)]);
    const uploadRes = await request.post(`${API_BASE}/api/sessions/${sessionId}/models`, {
      multipart: {
        file: {
          name: 'test.stl',
          mimeType: 'application/octet-stream',
          buffer: stlContent,
        },
      },
    });
    const uploadData = await uploadRes.json();
    const modelId = uploadData.model_id;

    // Configure model with material
    const configRes = await request.patch(
      `${API_BASE}/api/sessions/${sessionId}/models/${modelId}`,
      {
        data: { material_id: materialId },
      }
    );

    expect(configRes.ok()).toBeTruthy();
    const configData = await configRes.json();
    expect(configData.model_id).toBe(modelId);
    expect(configData.material_id).toBe(materialId);
    expect(configData).toHaveProperty('estimated_price');

    // Generate quote
    const quoteRes = await request.post(`${API_BASE}/api/sessions/${sessionId}/quote`);
    expect(quoteRes.ok()).toBeTruthy();

    const quote = await quoteRes.json();
    expect(quote).toHaveProperty('quote_id');
    expect(quote.quote_id).toMatch(/^[0-9A-Z]{26}$/);
    expect(quote.items.length).toBeGreaterThan(0);
    expect(quote).toHaveProperty('subtotal');
    expect(quote).toHaveProperty('fees');
    expect(quote).toHaveProperty('total');
    expect(quote.total).toBeGreaterThanOrEqual(quote.subtotal);
  });

  test('should return 404 for quote with invalid session', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/sessions/invalid-session-id/quote`);
    expect(response.status()).toBe(404);
  });

  test('quote should apply minimum order value', async ({ request }) => {
    // Create session
    const sessionRes = await request.post(`${API_BASE}/api/sessions`);
    const { session_id: sessionId } = await sessionRes.json();

    const response = await request.get(`${API_BASE}/api/sessions/${sessionId}/quote`);
    const quote = await response.json();

    // Check minimum_applied flag
    expect(quote).toHaveProperty('minimum_applied');
    expect(typeof quote.minimum_applied).toBe('boolean');

    // If minimum is applied, total should be at least the minimum
    if (quote.minimum_applied) {
      expect(quote.total).toBeGreaterThanOrEqual(10); // Assuming 10€ minimum
    }
  });
});
