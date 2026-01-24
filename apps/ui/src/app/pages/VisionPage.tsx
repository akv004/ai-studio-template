import { useState } from 'react';
import { Camera, Upload, Sparkles, Square, Crosshair, GitBranch } from 'lucide-react';
import { CanvasDemo } from '../components/CanvasDemo';

/**
 * Vision Page
 * 
 * Features:
 * - Image preview with detection overlays
 * - Canvas-based node graph demo (heavy visuals)
 */
export function VisionPage() {
    const [isDetecting, setIsDetecting] = useState(false);
    const [hasImage] = useState(true);
    const [viewMode, setViewMode] = useState<'detection' | 'canvas'>('detection');

    // Mock detection boxes
    const mockDetections = [
        { label: 'Person', confidence: 0.94, x: 15, y: 10, w: 25, h: 60, color: '#22c55e' },
        { label: 'Car', confidence: 0.87, x: 55, y: 35, w: 35, h: 30, color: '#3b82f6' },
        { label: 'Dog', confidence: 0.76, x: 5, y: 55, w: 15, h: 20, color: '#f59e0b' },
    ];

    const handleCapture = () => {
        setIsDetecting(true);
        setTimeout(() => setIsDetecting(false), 1500);
    };

    return (
        <div className="animate-fade-in h-full flex flex-col">
            {/* Page Header */}
            <div className="page-header">
                <div>
                    <h1 className="page-title">Vision</h1>
                    <p className="page-description">Object detection and image analysis</p>
                </div>
                <div className="flex items-center gap-2">
                    {/* View Mode Toggle */}
                    <div className="flex rounded-lg overflow-hidden border border-[var(--border-subtle)] mr-4">
                        <button
                            className={`px-3 py-1.5 text-sm ${viewMode === 'detection' ? 'bg-[var(--accent-primary)] text-white' : 'bg-[var(--bg-tertiary)]'}`}
                            onClick={() => setViewMode('detection')}
                        >
                            Detection
                        </button>
                        <button
                            className={`px-3 py-1.5 text-sm flex items-center gap-1.5 ${viewMode === 'canvas' ? 'bg-[var(--accent-primary)] text-white' : 'bg-[var(--bg-tertiary)]'}`}
                            onClick={() => setViewMode('canvas')}
                        >
                            <GitBranch className="w-3.5 h-3.5" />
                            Node Graph
                        </button>
                    </div>

                    {viewMode === 'detection' && (
                        <>
                            <button className="btn btn-secondary">
                                <Upload className="w-4 h-4" />
                                Upload Image
                            </button>
                            <button className="btn btn-primary" onClick={handleCapture}>
                                <Camera className="w-4 h-4" />
                                Capture
                            </button>
                        </>
                    )}
                </div>
            </div>

            {/* Canvas Node Graph Demo */}
            {viewMode === 'canvas' && (
                <div className="flex-1 mt-4 panel overflow-hidden">
                    <CanvasDemo />
                </div>
            )}

            {/* Detection View */}
            {viewMode === 'detection' && (
                <div className="flex-1 flex gap-4 mt-4">
                    {/* Image Preview */}
                    <div className="flex-1 panel">
                        <div className="panel-header">
                            <span className="panel-title">Preview</span>
                            <div className="flex items-center gap-2">
                                <button className="btn btn-ghost btn-sm">
                                    <Square className="w-4 h-4" />
                                </button>
                                <button className="btn btn-ghost btn-sm">
                                    <Crosshair className="w-4 h-4" />
                                </button>
                            </div>
                        </div>
                        <div className="relative aspect-video bg-[var(--bg-primary)] m-4 rounded-lg overflow-hidden">
                            {/* Placeholder Image */}
                            <div
                                className="absolute inset-0 flex items-center justify-center"
                                style={{
                                    background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #1a1a2e 100%)'
                                }}
                            >
                                {!hasImage ? (
                                    <div className="text-center">
                                        <Camera className="w-12 h-12 mx-auto text-[var(--text-muted)] mb-2" />
                                        <span className="text-[var(--text-muted)]">No image loaded</span>
                                    </div>
                                ) : (
                                    <div className="w-full h-full relative">
                                        {/* Simulated image background */}
                                        <div className="absolute inset-0 opacity-20 bg-gradient-to-br from-blue-500 via-purple-500 to-pink-500" />

                                        {/* Detection Overlays */}
                                        {mockDetections.map((det, i) => (
                                            <div
                                                key={i}
                                                className="absolute border-2 rounded transition-all"
                                                style={{
                                                    left: `${det.x}%`,
                                                    top: `${det.y}%`,
                                                    width: `${det.w}%`,
                                                    height: `${det.h}%`,
                                                    borderColor: det.color,
                                                    boxShadow: `0 0 10px ${det.color}40`
                                                }}
                                            >
                                                <div
                                                    className="absolute -top-6 left-0 px-2 py-0.5 rounded text-xs font-medium"
                                                    style={{ background: det.color }}
                                                >
                                                    {det.label} {Math.round(det.confidence * 100)}%
                                                </div>
                                            </div>
                                        ))}

                                        {/* Processing Overlay */}
                                        {isDetecting && (
                                            <div className="absolute inset-0 bg-black/50 flex items-center justify-center">
                                                <div className="flex items-center gap-3 text-white">
                                                    <Sparkles className="w-5 h-5 animate-pulse" />
                                                    <span>Running detection...</span>
                                                </div>
                                            </div>
                                        )}
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* Detections Panel */}
                    <div className="w-80 panel">
                        <div className="panel-header">
                            <span className="panel-title">Detections</span>
                            <span className="text-xs text-[var(--text-muted)]">{mockDetections.length} found</span>
                        </div>
                        <div className="panel-content space-y-2">
                            {mockDetections.map((det, i) => (
                                <div
                                    key={i}
                                    className="flex items-center gap-3 p-3 rounded-md bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)] cursor-pointer"
                                >
                                    <div
                                        className="w-3 h-3 rounded-full"
                                        style={{ background: det.color }}
                                    />
                                    <div className="flex-1">
                                        <div className="font-medium text-sm">{det.label}</div>
                                        <div className="text-xs text-[var(--text-muted)]">
                                            Confidence: {Math.round(det.confidence * 100)}%
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
