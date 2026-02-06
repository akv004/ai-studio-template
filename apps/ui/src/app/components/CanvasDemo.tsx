import { useEffect, useRef, useState } from 'react';
import { CanvasRenderer } from '../../canvas';
import { RefreshCw, ZoomIn, ZoomOut } from 'lucide-react';

/**
 * Canvas Demo Component
 * 
 * Demonstrates the Canvas Rendering Layer with:
 * - Node graph visualization
 * - Bezier curve connections
 * - Interactive canvas
 */

interface Node {
    id: string;
    x: number;
    y: number;
    width: number;
    height: number;
    label: string;
    type: 'input' | 'process' | 'output';
    inputs: string[];
    outputs: string[];
}

interface Connection {
    from: string;
    fromPort: number;
    to: string;
    toPort: number;
}

// Mock node graph data
const mockNodes: Node[] = [
    { id: 'input1', x: 50, y: 80, width: 140, height: 70, label: 'Image Input', type: 'input', inputs: [], outputs: ['out1'] },
    { id: 'input2', x: 50, y: 200, width: 140, height: 70, label: 'Model Config', type: 'input', inputs: [], outputs: ['out1'] },
    { id: 'process1', x: 280, y: 120, width: 160, height: 90, label: 'Preprocessing', type: 'process', inputs: ['in1', 'in2'], outputs: ['out1'] },
    { id: 'process2', x: 520, y: 100, width: 160, height: 90, label: 'YOLOv8 Detect', type: 'process', inputs: ['in1'], outputs: ['out1', 'out2'] },
    { id: 'output1', x: 760, y: 60, width: 140, height: 70, label: 'Detections', type: 'output', inputs: ['in1'], outputs: [] },
    { id: 'output2', x: 760, y: 180, width: 140, height: 70, label: 'Visualized', type: 'output', inputs: ['in1'], outputs: [] },
];

const mockConnections: Connection[] = [
    { from: 'input1', fromPort: 0, to: 'process1', toPort: 0 },
    { from: 'input2', fromPort: 0, to: 'process1', toPort: 1 },
    { from: 'process1', fromPort: 0, to: 'process2', toPort: 0 },
    { from: 'process2', fromPort: 0, to: 'output1', toPort: 0 },
    { from: 'process2', fromPort: 1, to: 'output2', toPort: 0 },
];

export function CanvasDemo() {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const rendererRef = useRef<CanvasRenderer | null>(null);
    const [selectedNode, setSelectedNode] = useState<string | null>(null);
    const animationRef = useRef<number>(0);

    // Node type colors
    const nodeColors = {
        input: { bg: '#1e3a5f', border: '#3b82f6', text: '#93c5fd' },
        process: { bg: '#3b2f4a', border: '#8b5cf6', text: '#c4b5fd' },
        output: { bg: '#1e3f3a', border: '#22c55e', text: '#86efac' },
    };

    // Get port position on a node
    const getPortPosition = (node: Node, isOutput: boolean, portIndex: number, totalPorts: number) => {
        const spacing = node.height / (totalPorts + 1);
        const x = isOutput ? node.x + node.width : node.x;
        const y = node.y + spacing * (portIndex + 1);
        return { x, y };
    };

    // Main render function
    const render = (time: number) => {
        const renderer = rendererRef.current;
        if (!renderer) return;

        renderer.beginFrame();

        // Draw connections first (behind nodes)
        mockConnections.forEach((conn) => {
            const fromNode = mockNodes.find(n => n.id === conn.from);
            const toNode = mockNodes.find(n => n.id === conn.to);
            if (!fromNode || !toNode) return;

            const start = getPortPosition(fromNode, true, conn.fromPort, fromNode.outputs.length);
            const end = getPortPosition(toNode, false, conn.toPort, toNode.inputs.length);

            // Animated flow effect
            const flowOffset = (time * 0.001) % 1;

            // Draw bezier curve
            const controlOffset = Math.abs(end.x - start.x) * 0.5;
            renderer.drawBezier(
                start.x, start.y,
                start.x + controlOffset, start.y,
                end.x - controlOffset, end.y,
                end.x, end.y,
                '#8b5cf680',
                2
            );

            // Draw animated dots along the curve
            for (let i = 0; i < 3; i++) {
                const t = (flowOffset + i * 0.33) % 1;
                const dotX = Math.pow(1 - t, 3) * start.x +
                    3 * Math.pow(1 - t, 2) * t * (start.x + controlOffset) +
                    3 * (1 - t) * Math.pow(t, 2) * (end.x - controlOffset) +
                    Math.pow(t, 3) * end.x;
                const dotY = Math.pow(1 - t, 3) * start.y +
                    3 * Math.pow(1 - t, 2) * t * start.y +
                    3 * (1 - t) * Math.pow(t, 2) * end.y +
                    Math.pow(t, 3) * end.y;
                renderer.drawCircle(dotX, dotY, 3, '#8b5cf6');
            }

            // Draw port circles
            renderer.drawCircle(start.x, start.y, 6, '#8b5cf6');
            renderer.drawCircle(end.x, end.y, 6, '#8b5cf6');
        });

        // Draw nodes
        mockNodes.forEach((node) => {
            const colors = nodeColors[node.type];
            const isSelected = selectedNode === node.id;

            // Node shadow
            renderer.drawRoundedRect(node.x + 4, node.y + 4, node.width, node.height, 8, 'rgba(0,0,0,0.3)');

            // Node background
            renderer.drawRoundedRect(node.x, node.y, node.width, node.height, 8, colors.bg);

            // Node border (using lines for selected highlight)
            if (isSelected) {
                renderer.drawRoundedRect(node.x - 2, node.y - 2, node.width + 4, node.height + 4, 10, '#ffffff40');
            }

            // Node label (centered manually)
            renderer.drawText(
                node.label,
                node.x + 10,
                node.y + node.height / 2 + 4,
                colors.text,
                '13px Inter, sans-serif'
            );

            // Node type badge
            renderer.drawRoundedRect(node.x + 8, node.y + 8, 50, 16, 4, colors.border);
            renderer.drawText(
                node.type.toUpperCase(),
                node.x + 12,
                node.y + 19,
                '#ffffff',
                '9px Inter, sans-serif'
            );
        });

        // Instructions
        renderer.drawText(
            'Node Graph Demo - Canvas Rendering Layer',
            10, 20,
            '#ffffff80',
            '12px Inter, sans-serif'
        );

        renderer.endFrame();
        animationRef.current = requestAnimationFrame(render);
    };

    useEffect(() => {
        if (!canvasRef.current) return;

        const renderer = new CanvasRenderer();
        renderer.init(canvasRef.current);
        rendererRef.current = renderer;

        // Start animation loop
        animationRef.current = requestAnimationFrame(render);

        // Handle resize
        const handleResize = () => {
            if (canvasRef.current && rendererRef.current) {
                rendererRef.current.resize(
                    canvasRef.current.clientWidth,
                    canvasRef.current.clientHeight
                );
            }
        };

        window.addEventListener('resize', handleResize);
        handleResize();

        return () => {
            cancelAnimationFrame(animationRef.current);
            window.removeEventListener('resize', handleResize);
        };
    }, []);

    // Handle canvas click to select nodes
    const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
        const rect = canvasRef.current?.getBoundingClientRect();
        if (!rect) return;

        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        // Check if clicked on a node
        const clickedNode = mockNodes.find(node =>
            x >= node.x && x <= node.x + node.width &&
            y >= node.y && y <= node.y + node.height
        );

        setSelectedNode(clickedNode?.id || null);
    };

    return (
        <div className="h-full flex flex-col">
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-[var(--border-subtle)]">
                <div>
                    <h2 className="font-semibold">Canvas Rendering Demo</h2>
                    <p className="text-sm text-[var(--text-muted)]">
                        Node graph using CanvasRenderer API
                    </p>
                </div>
                <div className="flex items-center gap-2">
                    <button className="btn btn-ghost btn-sm">
                        <ZoomOut className="w-4 h-4" />
                    </button>
                    <span className="text-sm text-[var(--text-muted)]">100%</span>
                    <button className="btn btn-ghost btn-sm">
                        <ZoomIn className="w-4 h-4" />
                    </button>
                    <button className="btn btn-secondary btn-sm ml-4">
                        <RefreshCw className="w-4 h-4" />
                        Reset View
                    </button>
                </div>
            </div>

            {/* Canvas Container */}
            <div className="flex-1 relative bg-[#0a0a0f]">
                <canvas
                    ref={canvasRef}
                    className="absolute inset-0 w-full h-full"
                    onClick={handleCanvasClick}
                />

                {/* Selected node info */}
                {selectedNode && (
                    <div className="absolute bottom-4 left-4 p-3 rounded-lg bg-[var(--bg-elevated)] border border-[var(--border-subtle)]">
                        <div className="text-sm font-medium">
                            Selected: {mockNodes.find(n => n.id === selectedNode)?.label}
                        </div>
                        <div className="text-xs text-[var(--text-muted)] mt-1">
                            Click elsewhere to deselect
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
