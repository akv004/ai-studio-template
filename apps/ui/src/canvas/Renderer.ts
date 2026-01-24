// ============================================
// CANVAS RENDERER ABSTRACTION
// Abstract interface for rendering engines
// Allows swapping Canvas2D → WebGL → WebGPU
// ============================================

/**
 * Abstract Renderer Interface
 * Implementations must provide these methods for canvas-first UI
 */
export interface Renderer {
    /** Initialize the renderer with a canvas element */
    init(canvas: HTMLCanvasElement): void;

    /** Clear the entire canvas */
    clear(): void;

    /** Begin a new frame */
    beginFrame(): void;

    /** End the current frame and flush to screen */
    endFrame(): void;

    /** Draw a rectangle */
    drawRect(x: number, y: number, width: number, height: number, color: string): void;

    /** Draw a rounded rectangle */
    drawRoundedRect(x: number, y: number, width: number, height: number, radius: number, color: string): void;

    /** Draw a circle */
    drawCircle(x: number, y: number, radius: number, color: string): void;

    /** Draw a line */
    drawLine(x1: number, y1: number, x2: number, y2: number, color: string, width?: number): void;

    /** Draw text */
    drawText(text: string, x: number, y: number, color: string, font?: string): void;

    /** Draw a bezier curve (for node connections) */
    drawBezier(
        x1: number, y1: number,
        cx1: number, cy1: number,
        cx2: number, cy2: number,
        x2: number, y2: number,
        color: string,
        width?: number
    ): void;

    /** Resize the canvas */
    resize(width: number, height: number): void;

    /** Get the canvas dimensions */
    getSize(): { width: number; height: number };

    /** Dispose of resources */
    dispose(): void;
}

/**
 * Renderer factory function
 * Currently returns CanvasRenderer, will support WebGPU in future
 */
export type RendererType = 'canvas' | 'webgl' | 'webgpu';

export function createRenderer(type: RendererType = 'canvas'): Renderer {
    // Import CanvasRenderer at the top level for static analysis
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    switch (type) {
        case 'canvas':
            // Use dynamic import pattern - in practice, import CanvasRenderer directly
            // For now, we throw to encourage explicit instantiation
            throw new Error('Use new CanvasRenderer() directly instead of createRenderer()');
        case 'webgl':
            // TODO: Implement WebGL renderer
            throw new Error('WebGL renderer not yet implemented');
        case 'webgpu':
            // TODO: Implement WebGPU renderer
            throw new Error('WebGPU renderer not yet implemented');
        default:
            throw new Error(`Unknown renderer type: ${type}`);
    }
}
