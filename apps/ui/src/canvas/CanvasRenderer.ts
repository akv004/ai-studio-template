import type { Renderer } from './Renderer';

// ============================================
// CANVAS 2D RENDERER
// Primary renderer implementation using HTML Canvas 2D API
// High-performance, widely supported
// ============================================

export class CanvasRenderer implements Renderer {
    private canvas: HTMLCanvasElement | null = null;
    private ctx: CanvasRenderingContext2D | null = null;
    private dpr: number = 1;

    init(canvas: HTMLCanvasElement): void {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');

        if (!this.ctx) {
            throw new Error('Failed to get 2D rendering context');
        }

        // Handle high-DPI displays
        this.dpr = window.devicePixelRatio || 1;
        this.resize(canvas.clientWidth, canvas.clientHeight);
    }

    clear(): void {
        if (!this.ctx || !this.canvas) return;
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }

    beginFrame(): void {
        this.clear();
        if (this.ctx) {
            this.ctx.save();
            this.ctx.scale(this.dpr, this.dpr);
        }
    }

    endFrame(): void {
        if (this.ctx) {
            this.ctx.restore();
        }
    }

    drawRect(x: number, y: number, width: number, height: number, color: string): void {
        if (!this.ctx) return;
        this.ctx.fillStyle = color;
        this.ctx.fillRect(x, y, width, height);
    }

    drawRoundedRect(x: number, y: number, width: number, height: number, radius: number, color: string): void {
        if (!this.ctx) return;
        this.ctx.fillStyle = color;
        this.ctx.beginPath();
        this.ctx.roundRect(x, y, width, height, radius);
        this.ctx.fill();
    }

    drawCircle(x: number, y: number, radius: number, color: string): void {
        if (!this.ctx) return;
        this.ctx.fillStyle = color;
        this.ctx.beginPath();
        this.ctx.arc(x, y, radius, 0, Math.PI * 2);
        this.ctx.fill();
    }

    drawLine(x1: number, y1: number, x2: number, y2: number, color: string, width: number = 1): void {
        if (!this.ctx) return;
        this.ctx.strokeStyle = color;
        this.ctx.lineWidth = width;
        this.ctx.beginPath();
        this.ctx.moveTo(x1, y1);
        this.ctx.lineTo(x2, y2);
        this.ctx.stroke();
    }

    drawText(text: string, x: number, y: number, color: string, font: string = '13px Inter, sans-serif'): void {
        if (!this.ctx) return;
        this.ctx.fillStyle = color;
        this.ctx.font = font;
        this.ctx.fillText(text, x, y);
    }

    drawBezier(
        x1: number, y1: number,
        cx1: number, cy1: number,
        cx2: number, cy2: number,
        x2: number, y2: number,
        color: string,
        width: number = 2
    ): void {
        if (!this.ctx) return;
        this.ctx.strokeStyle = color;
        this.ctx.lineWidth = width;
        this.ctx.lineCap = 'round';
        this.ctx.beginPath();
        this.ctx.moveTo(x1, y1);
        this.ctx.bezierCurveTo(cx1, cy1, cx2, cy2, x2, y2);
        this.ctx.stroke();
    }

    resize(width: number, height: number): void {
        if (!this.canvas) return;

        // Set display size
        this.canvas.style.width = `${width}px`;
        this.canvas.style.height = `${height}px`;

        // Set actual pixel size for high-DPI
        this.canvas.width = width * this.dpr;
        this.canvas.height = height * this.dpr;
    }

    getSize(): { width: number; height: number } {
        if (!this.canvas) return { width: 0, height: 0 };
        return {
            width: this.canvas.clientWidth,
            height: this.canvas.clientHeight,
        };
    }

    dispose(): void {
        this.canvas = null;
        this.ctx = null;
    }
}
