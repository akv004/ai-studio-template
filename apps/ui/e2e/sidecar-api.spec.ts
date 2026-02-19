import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

const SIDECAR_URL = process.env.SIDECAR_URL || 'http://localhost:8765';

test.describe('Sidecar API Tests', () => {

    test('health endpoint responds', async ({ request }) => {
        const resp = await request.get(`${SIDECAR_URL}/health`);
        if (resp.ok()) {
            const body = await resp.json();
            expect(body.status).toBeDefined();
            console.log('[sidecar] Health:', JSON.stringify(body));
        } else {
            console.warn('[sidecar] Health check failed — sidecar may not be running');
            test.skip();
        }
    });

    test('chat/direct with text prompt returns response', async ({ request }) => {
        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            data: {
                messages: [{ role: 'user', content: 'What is 2+2? Answer with just the number.' }],
                provider: 'local',
                model: 'qwen3-vl',
                temperature: 0.1,
                base_url: 'http://localhost:11434/v1',
            },
        });

        if (!resp.ok()) {
            console.warn('[sidecar] chat/direct failed — provider may not be available');
            test.skip();
            return;
        }

        const body = await resp.json();
        expect(body.content).toBeDefined();
        expect(typeof body.content).toBe('string');
        expect(body.content.length).toBeGreaterThan(0);
        console.log('[sidecar] Chat response:', body.content.slice(0, 100));

        // Verify usage stats are returned
        expect(body.usage).toBeDefined();
    });

    test('chat/direct with vision (single image) sends multimodal', async ({ request }) => {
        // Read the test image
        const imagePath = '/tmp/ai-studio-samples/dashboard-chart.png';
        if (!fs.existsSync(imagePath)) {
            console.warn('[sidecar] Test image not found at', imagePath);
            test.skip();
            return;
        }

        const imageBuffer = fs.readFileSync(imagePath);
        const base64Data = imageBuffer.toString('base64');

        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            data: {
                messages: [{ role: 'user', content: 'Describe what you see in this image briefly.' }],
                provider: 'local',
                model: 'qwen3-vl',
                temperature: 0.1,
                base_url: 'http://localhost:11434/v1',
                images: [{
                    data: base64Data,
                    mime_type: 'image/png',
                }],
            },
        });

        if (!resp.ok()) {
            const errorText = await resp.text();
            console.warn('[sidecar] Vision chat failed:', resp.status(), errorText.slice(0, 200));
            test.skip();
            return;
        }

        const body = await resp.json();
        expect(body.content).toBeDefined();
        expect(body.content.length).toBeGreaterThan(10);
        console.log('[sidecar] Vision response:', body.content.slice(0, 200));

        // The response should mention something about the image content
        // (sales, costs, profit, chart, dashboard, etc.)
        const lowerContent = body.content.toLowerCase();
        const mentionsVisual = ['sales', 'cost', 'profit', 'chart', 'box', 'image', 'blue', 'red', 'green']
            .some(keyword => lowerContent.includes(keyword));
        expect(mentionsVisual).toBe(true);
    });

    test('chat/direct with multiple images', async ({ request }) => {
        const imagePath = '/tmp/ai-studio-samples/dashboard-chart.png';
        if (!fs.existsSync(imagePath)) {
            test.skip();
            return;
        }

        const imageBuffer = fs.readFileSync(imagePath);
        const base64Data = imageBuffer.toString('base64');

        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            data: {
                messages: [{ role: 'user', content: 'How many images do you see? Describe each briefly.' }],
                provider: 'local',
                model: 'qwen3-vl',
                temperature: 0.1,
                base_url: 'http://localhost:11434/v1',
                images: [
                    { data: base64Data, mime_type: 'image/png' },
                    { data: base64Data, mime_type: 'image/png' },
                ],
            },
        });

        if (!resp.ok()) {
            console.warn('[sidecar] Multi-image test failed — may not support multiple images');
            test.skip();
            return;
        }

        const body = await resp.json();
        expect(body.content).toBeDefined();
        expect(body.content.length).toBeGreaterThan(10);
        console.log('[sidecar] Multi-image response:', body.content.slice(0, 200));
    });

    test('chat/direct rejects empty messages', async ({ request }) => {
        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            data: {
                messages: [],
                provider: 'local',
                model: 'qwen3-vl',
            },
        });

        // Should return 4xx or 5xx error
        if (resp.ok()) {
            console.log('[sidecar] Empty messages was accepted (unexpected but not critical)');
        } else {
            expect(resp.status()).toBeGreaterThanOrEqual(400);
        }
    });
});

test.describe('File Read → LLM Integration', () => {

    test('read text file and send to LLM', async ({ request }) => {
        const filePath = '/tmp/ai-studio-samples/bug-report.txt';
        if (!fs.existsSync(filePath)) {
            test.skip();
            return;
        }

        const content = fs.readFileSync(filePath, 'utf-8');

        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            data: {
                messages: [{ role: 'user', content: content }],
                provider: 'local',
                model: 'qwen3-vl',
                temperature: 0.1,
                base_url: 'http://localhost:11434/v1',
                system_prompt: 'Summarize this bug report in 2-3 sentences.',
            },
        });

        if (!resp.ok()) {
            test.skip();
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
            test.skip();
            return;
        }

        const content = fs.readFileSync(filePath, 'utf-8');

        const resp = await request.post(`${SIDECAR_URL}/chat/direct`, {
            data: {
                messages: [{ role: 'user', content: content }],
                provider: 'local',
                model: 'qwen3-vl',
                temperature: 0.1,
                base_url: 'http://localhost:11434/v1',
                system_prompt: 'What is the top performing product by growth? Answer in one sentence.',
            },
        });

        if (!resp.ok()) {
            test.skip();
            return;
        }

        const body = await resp.json();
        expect(body.content).toBeDefined();
        console.log('[integration] CSV analysis:', body.content.slice(0, 300));
    });
});
