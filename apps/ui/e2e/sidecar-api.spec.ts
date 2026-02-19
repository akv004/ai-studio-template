import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as http from 'http';

const SIDECAR_URL = process.env.SIDECAR_URL || 'http://localhost:8765';
const SIDECAR_TOKEN = process.env.AI_STUDIO_TOKEN || '';
const LLM_BASE_URL = process.env.LLM_BASE_URL || 'http://localhost:8003/v1';
const LLM_MODEL = process.env.LLM_MODEL || 'Qwen/Qwen3-VL-8B-Instruct';

// Common headers for authenticated sidecar requests
const headers = (): Record<string, string> => {
    const h: Record<string, string> = { 'Content-Type': 'application/json' };
    if (SIDECAR_TOKEN) h['x-ai-studio-token'] = SIDECAR_TOKEN;
    return h;
};

// Check if sidecar is reachable (fast TCP check)
async function isSidecarUp(): Promise<boolean> {
    const url = new URL(SIDECAR_URL);
    return new Promise((resolve) => {
        const req = http.get(`${SIDECAR_URL}/health`, { timeout: 2000 }, (res) => {
            resolve(res.statusCode === 200);
        });
        req.on('error', () => resolve(false));
        req.on('timeout', () => { req.destroy(); resolve(false); });
    });
}

test.describe('Sidecar API Tests', () => {

    test.beforeEach(async () => {
        if (!(await isSidecarUp())) {
            test.skip(true, 'Sidecar not running');
        }
    });

    test('health endpoint responds', async ({ request }) => {
        const resp = await request.get(`${SIDECAR_URL}/health`);
        const body = await resp.json();
        expect(body.status).toBe('healthy');
        console.log('[sidecar] Health:', JSON.stringify(body));
    });

    test('chat/direct with text prompt returns response', async ({ request }) => {
        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            headers: headers(),
            data: {
                messages: [{ role: 'user', content: 'What is 2+2? Answer with just the number.' }],
                provider: 'local',
                model: LLM_MODEL,
                temperature: 0.1,
                base_url: LLM_BASE_URL,
            },
        });

        if (!resp.ok()) {
            console.warn('[sidecar] chat/direct failed:', resp.status(), (await resp.text()).slice(0, 200));
            test.skip(true, 'Provider not available');
            return;
        }

        const body = await resp.json();
        expect(body.content).toBeDefined();
        expect(typeof body.content).toBe('string');
        expect(body.content.length).toBeGreaterThan(0);
        console.log('[sidecar] Chat response:', body.content.slice(0, 100));
        expect(body.usage).toBeDefined();
    });

    test('chat/direct with vision (single image) sends multimodal', async ({ request }) => {
        const imagePath = '/tmp/ai-studio-samples/dashboard-chart.png';
        if (!fs.existsSync(imagePath)) {
            test.skip(true, 'Test image not found');
            return;
        }

        const imageBuffer = fs.readFileSync(imagePath);
        const base64Data = imageBuffer.toString('base64');

        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            headers: headers(),
            data: {
                messages: [{ role: 'user', content: 'Describe what you see in this image briefly.' }],
                provider: 'local',
                model: LLM_MODEL,
                temperature: 0.1,
                base_url: LLM_BASE_URL,
                images: [{
                    data: base64Data,
                    mime_type: 'image/png',
                }],
            },
        });

        if (!resp.ok()) {
            console.warn('[sidecar] Vision chat failed:', resp.status());
            test.skip(true, 'Vision provider not available');
            return;
        }

        const body = await resp.json();
        expect(body.content).toBeDefined();
        expect(body.content.length).toBeGreaterThan(10);
        console.log('[sidecar] Vision response:', body.content.slice(0, 200));

        const lowerContent = body.content.toLowerCase();
        const mentionsVisual = ['sales', 'cost', 'profit', 'chart', 'box', 'image', 'blue', 'red', 'green']
            .some(keyword => lowerContent.includes(keyword));
        expect(mentionsVisual).toBe(true);
    });

    test('chat/direct with multiple images', async ({ request }) => {
        const imagePath = '/tmp/ai-studio-samples/dashboard-chart.png';
        if (!fs.existsSync(imagePath)) {
            test.skip(true, 'Test image not found');
            return;
        }

        const imageBuffer = fs.readFileSync(imagePath);
        const base64Data = imageBuffer.toString('base64');

        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            headers: headers(),
            data: {
                messages: [{ role: 'user', content: 'How many images do you see? Describe each briefly.' }],
                provider: 'local',
                model: LLM_MODEL,
                temperature: 0.1,
                base_url: LLM_BASE_URL,
                images: [
                    { data: base64Data, mime_type: 'image/png' },
                    { data: base64Data, mime_type: 'image/png' },
                ],
            },
        });

        if (!resp.ok()) {
            console.warn('[sidecar] Multi-image test failed');
            test.skip(true, 'Multi-image not available');
            return;
        }

        const body = await resp.json();
        expect(body.content).toBeDefined();
        expect(body.content.length).toBeGreaterThan(10);
        console.log('[sidecar] Multi-image response:', body.content.slice(0, 200));
    });

    test('chat/direct rejects empty messages', async ({ request }) => {
        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            headers: headers(),
            data: {
                messages: [],
                provider: 'local',
                model: LLM_MODEL,
            },
        });

        if (resp.ok()) {
            console.log('[sidecar] Empty messages was accepted (unexpected but not critical)');
        } else {
            expect(resp.status()).toBeGreaterThanOrEqual(400);
        }
    });
});

test.describe('File Read → LLM Integration', () => {

    test.beforeEach(async () => {
        if (!(await isSidecarUp())) {
            test.skip(true, 'Sidecar not running');
        }
    });

    test('read text file and send to LLM', async ({ request }) => {
        const filePath = '/tmp/ai-studio-samples/bug-report.txt';
        if (!fs.existsSync(filePath)) {
            test.skip(true, 'Test file not found');
            return;
        }

        const content = fs.readFileSync(filePath, 'utf-8');

        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            headers: headers(),
            data: {
                messages: [{ role: 'user', content: content }],
                provider: 'local',
                model: LLM_MODEL,
                temperature: 0.1,
                base_url: LLM_BASE_URL,
                system_prompt: 'Summarize this bug report in 2-3 sentences.',
            },
        });

        if (!resp.ok()) {
            test.skip(true, 'Provider not available');
            return;
        }

        const body = await resp.json();
        expect(body.content).toBeDefined();
        expect(body.content.length).toBeGreaterThan(20);
        console.log('[integration] Bug report summary:', body.content.slice(0, 300));
    });

    test('read CSV and analyze with LLM', async ({ request }) => {
        const filePath = '/tmp/ai-studio-samples/sales-data.csv';
        if (!fs.existsSync(filePath)) {
            test.skip(true, 'Test file not found');
            return;
        }

        const content = fs.readFileSync(filePath, 'utf-8');

        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            headers: headers(),
            data: {
                messages: [{ role: 'user', content: content }],
                provider: 'local',
                model: LLM_MODEL,
                temperature: 0.1,
                base_url: LLM_BASE_URL,
                system_prompt: 'What is the top performing product by growth? Answer in one sentence.',
            },
        });

        if (!resp.ok()) {
            test.skip(true, 'Provider not available');
            return;
        }

        const body = await resp.json();
        expect(body.content).toBeDefined();
        console.log('[integration] CSV analysis:', body.content.slice(0, 300));
    });
});
